//!
//! Public state-variable getter synthesis.
//!
//! Solidity synthesises an external accessor for every `public` state variable.
//! The state variable emits its own accessor: a `constant` folds to a pure
//! literal getter, a scalar reads its slot, a mapping/array chains a
//! `sol.map`/bounds-checked `sol.gep` per key/index, and a struct expands to its
//! flattened returnable members.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use num_bigint::BigInt;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::Function;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::RequireOperation;
use solx_mlir::ods::sol::ReturnOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Pointer;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;

/// The field-layout plan for a struct's `public` accessor return tuple.
pub trait StructGetterLayout {
    /// The destructured returnable members of a `public` struct, as its accessor
    /// returns them: for each member the accessor yields — every scalar, plus
    /// `string` / `bytes` returned as a memory copy — its field index, MLIR
    /// member type, and ABI result type. Nested mappings, arrays, and structs are
    /// skipped (Solidity omits them from the accessor tuple). `None` when no
    /// member is returnable or a member is untyped. Shared by the struct getter
    /// and a struct external-call return.
    fn struct_getter_layout<'context>(
        &self,
        struct_mlir_type: Type<'context>,
        builder: &Builder<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>>;
}

impl StructGetterLayout for StructDefinition {
    fn struct_getter_layout<'context>(
        &self,
        struct_mlir_type: Type<'context>,
        builder: &Builder<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>> {
        let mut plan = Vec::new();
        for (member_index, member) in self.members().iter().enumerate() {
            let is_string_or_bytes = match member.get_type() {
                Some(
                    SlangType::Mapping(_)
                    | SlangType::Array(_)
                    | SlangType::FixedSizeArray(_)
                    | SlangType::Struct(_),
                ) => continue,
                Some(SlangType::String(_) | SlangType::Bytes(_)) => true,
                Some(_) => false,
                None => return None,
            };
            let member_type = AstType::new(struct_mlir_type)
                .element_type(member_index)
                .into_mlir();
            // A `string`/`bytes` member returns a memory copy; every other member
            // reaching here is a value type (mapping/array/struct are skipped
            // above).
            let result_member_type = if is_string_or_bytes {
                AstType::string(builder.context, DataLocation::Memory).into_mlir()
            } else {
                member_type
            };
            plan.push((member_index as u64, member_type, result_member_type));
        }
        if plan.is_empty() {
            return None;
        }
        Some(plan)
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for StateVariableDefinition {
    type Output = ();

    /// Emits the auto-generated external accessor for this `public` state variable
    /// into the contract body. A variable with no accessor, or whose accessor is
    /// not yet supported (a non-struct reference terminal), is left ungenerated —
    /// the rest of the contract still compiles.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) {
        /// Folds a constant integer expression to a [`BigInt`] when it is one of
        /// the closed set of integer-foldable forms.
        fn fold_constant_int(expression: &Expression) -> Option<BigInt> {
            match expression {
                Expression::DecimalNumberExpression(decimal) => decimal.integer_value(),
                Expression::HexNumberExpression(hex) => hex.integer_value(),
                Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        fold_constant_int(&state_variable.value()?)
                    }
                    Some(Definition::Constant(constant)) => fold_constant_int(&constant.value()?),
                    _ => None,
                },
                Expression::MemberAccessExpression(access) => {
                    match access.member().resolve_to_definition() {
                        Some(Definition::StateVariable(state_variable)) => {
                            fold_constant_int(&state_variable.value()?)
                        }
                        Some(Definition::Constant(constant)) => {
                            fold_constant_int(&constant.value()?)
                        }
                        _ => None,
                    }
                }
                Expression::FunctionCallExpression(call) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = call.arguments()
                    else {
                        return None;
                    };
                    let mut arguments = positional.iter();
                    let argument = arguments.next()?;
                    if arguments.next().is_some() {
                        return None;
                    }
                    let is_wrap_unwrap = matches!(
                        &call.operand(),
                        Expression::MemberAccessExpression(member)
                            if matches!(
                                member.member().resolve_to_built_in(),
                                Some(BuiltIn::Wrap | BuiltIn::Unwrap)
                            )
                    );
                    if is_wrap_unwrap || call.is_type_conversion() {
                        fold_constant_int(&argument)
                    } else {
                        None
                    }
                }
                _ => expression
                    .get_type()
                    .and_then(|slang_type| match slang_type {
                        SlangType::Literal(literal) => match literal.kind() {
                            LiteralKind::Integer { value } => Some(value),
                            LiteralKind::HexInteger { value, .. } => Some(BigInt::from(value)),
                            _ => None,
                        },
                        _ => None,
                    }),
            }
        }

        /// Emits the terminal `sol.return` from the addressed `base`: a struct
        /// expands into its flattened returnable members (each loaded and coerced
        /// to its ABI result type), a scalar loads the single value.
        fn return_loaded<'context, 'block>(
            base: Value<'context, 'block>,
            struct_plan: &Option<Vec<(u64, Type<'context>, Type<'context>)>>,
            result_type: Type<'context>,
            builder: &Builder<'context>,
            entry: &BlockRef<'context, 'block>,
        ) {
            match struct_plan {
                Some(plan) => {
                    let mut values = Vec::new();
                    for (member_index, member_type, result_member_type) in plan {
                        let index_value = AstValue::constant(
                            *member_index as i64,
                            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
                            builder,
                            entry,
                        );
                        let address = Pointer::new(base)
                            .gep(index_value, AstType::new(*member_type), builder, entry)
                            .into_mlir();
                        let value = if member_type == result_member_type {
                            Pointer::new(address)
                                .load(AstType::new(*result_member_type), builder, entry)
                                .into_mlir()
                        } else {
                            AstValue::new(address)
                                .cast(AstType::new(*result_member_type), builder, entry)
                                .into_mlir()
                        };
                        values.push(value);
                    }
                    mlir_op_void!(builder, entry, ReturnOperation.operands(&values));
                }
                None => {
                    let value = Pointer::new(base)
                        .load(AstType::new(result_type), builder, entry)
                        .into_mlir();
                    mlir_op_void!(builder, entry, ReturnOperation.operands(&[value]));
                }
            }
        }

        let state_variable = self;
        let builder = &context.state.builder;

        // A variable with no ABI accessor has no getter.
        let abi = match state_variable.compute_abi_entry() {
            Some(AbiEntry::Function(abi)) => abi,
            _ => return,
        };

        // A `constant` getter folds its compile-time initializer (no slot, no
        // inputs), exactly as a file-level `constant`.
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            if !abi.inputs().is_empty() {
                return;
            }
            let Some(initializer) = state_variable.value() else {
                return;
            };
            let signature = state_variable
                .compute_canonical_signature()
                .expect("slang validated");
            let selector = state_variable.compute_selector().expect("slang validated");
            // Reference types (`string` / `bytes`) return in `Memory` — what an
            // external getter call hands back — not their declared storage location.
            let slang_type = state_variable.get_type().expect("slang validated");
            let element_type = AstType::resolve(&slang_type, LocationPolicy::ForceMemory, builder);
            let entry = Function::new(signature, Vec::new(), vec![element_type]).define(
                Some(selector),
                StateMutability::Pure,
                None,
                None,
                builder,
                &block,
            );
            if let Some(value) = fold_constant_int(&initializer) {
                let constant = AstValue::constant_from_bigint(
                    &value,
                    AstType::new(element_type),
                    builder,
                    &entry,
                )
                .into_mlir();
                mlir_op_void!(builder, &entry, ReturnOperation.operands(&[constant]));
                return;
            }
            // A non-integer constant (`string` / `bytesN` literal) materialises its
            // initializer toward the return type, as an explicit `return <const>`
            // would; the getter's empty scope is the threaded context itself.
            let BlockAnd {
                value,
                block: entry,
            } = if let Expression::StringExpression(string_literal) = &initializer {
                string_literal.emit_as(element_type, context, entry)
            } else {
                initializer.emit(context, entry)
            };
            let value = value
                .cast(AstType::new(element_type), builder, &entry)
                .into_mlir();
            mlir_op_void!(builder, &entry, ReturnOperation.operands(&[value]));
            return;
        }

        let Some(slot) = context.storage_layout.get(&state_variable.node_id()) else {
            return;
        };
        let location = slot.location;
        let declared_type = state_variable.get_type().expect("slang validated");

        // An indexed (mapping/array) getter: each key/index is a parameter. Walk
        // the nesting once to collect the parameter types and reach the terminal
        // type — needed to define the function before its entry block exists.
        if !abi.inputs().is_empty() {
            let signature = state_variable
                .compute_canonical_signature()
                .expect("slang validated");
            let selector = state_variable.compute_selector().expect("slang validated");

            let mut input_types: Vec<Type<'context>> = Vec::new();
            let mut terminal = declared_type.clone();
            loop {
                match &terminal {
                    SlangType::Mapping(mapping_type) => {
                        let key_slang = mapping_type.key_type();
                        // A reference-typed key (`string` / `bytes`) is an ABI input
                        // decoded into memory; slang reports it with the mapping's
                        // storage location, so build the memory type directly.
                        let key_type = if key_slang.is_reference_type() {
                            AstType::string(builder.context, DataLocation::Memory).into_mlir()
                        } else {
                            AstType::resolve(
                                &key_slang,
                                LocationPolicy::Declared(Some(location)),
                                builder,
                            )
                        };
                        input_types.push(key_type);
                        terminal = mapping_type.value_type();
                    }
                    SlangType::Array(array_type) => {
                        input_types.push(
                            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                                .into_mlir(),
                        );
                        terminal = array_type.element_type();
                    }
                    SlangType::FixedSizeArray(array_type) => {
                        input_types.push(
                            AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                                .into_mlir(),
                        );
                        terminal = array_type.element_type();
                    }
                    _ => break,
                }
            }
            if input_types.is_empty() || input_types.len() != abi.inputs().len() {
                return;
            }

            let container_type = AstType::resolve_state_variable(
                &state_variable.get_type().expect("slang validated"),
                builder,
            );
            let result_type =
                AstType::resolve(&terminal, LocationPolicy::Declared(Some(location)), builder);
            // A struct terminal expands into its flattened returnable members;
            // another reference terminal is not yet handled (left ungenerated).
            let struct_plan = match &terminal {
                SlangType::Struct(struct_type) => {
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        return;
                    };
                    match struct_definition.struct_getter_layout(result_type, builder) {
                        Some(plan) => Some(plan),
                        None => return,
                    }
                }
                _ if terminal.is_reference_type() => return,
                _ => None,
            };
            let result_types: Vec<Type<'context>> = match &struct_plan {
                Some(plan) => plan.iter().map(|(_, _, result)| *result).collect(),
                None => vec![result_type],
            };
            let entry = Function::new(signature, input_types, result_types).define(
                Some(selector),
                StateMutability::View,
                None,
                None,
                builder,
                &block,
            );
            let mut base =
                Pointer::addr_of(&slot.name, AstType::new(container_type), builder, &entry)
                    .into_mlir();
            // Re-walk the nesting, stepping the access over each parameter: a
            // `sol.map` for a mapping key, a bounds-checked `sol.gep` for an array
            // index (out-of-bounds bare-revert via a no-message `sol.require`,
            // matching solc's accessor — not `sol.gep`'s `Panic(0x32)`).
            let mut current = declared_type.clone();
            let mut index = 0usize;
            loop {
                match &current {
                    SlangType::Mapping(mapping_type) => {
                        let arg: Value<'context, 'block> = entry
                            .argument(index)
                            .expect("argument index is within the block signature")
                            .into();
                        let value_slang = mapping_type.value_type();
                        let resolved_value = AstType::resolve(
                            &value_slang,
                            LocationPolicy::Declared(Some(location)),
                            builder,
                        );
                        // Intermediate containers are addressed by their reference;
                        // a value terminal by a `!sol.ptr<V>`.
                        let level_type = if value_slang.is_reference_type() {
                            resolved_value
                        } else {
                            AstType::pointer(builder.context, resolved_value, location).into_mlir()
                        };
                        base = Pointer::new(base)
                            .entry(
                                AstValue::new(arg),
                                AstType::new(level_type),
                                builder,
                                &entry,
                            )
                            .into_mlir();
                        index += 1;
                        current = value_slang;
                    }
                    SlangType::Array(array_type) => {
                        let arg: Value<'context, 'block> = entry
                            .argument(index)
                            .expect("argument index is within the block signature")
                            .into();
                        let element_type = AstType::resolve(
                            &array_type.element_type(),
                            LocationPolicy::Declared(Some(location)),
                            builder,
                        );
                        let length = AstValue::new(base).length(builder, &entry);
                        let in_bounds = AstValue::new(arg)
                            .compare(length, CmpPredicate::Lt, builder, &entry)
                            .into_mlir();
                        mlir_op_void!(builder, &entry, RequireOperation.cond(in_bounds).args(&[]));
                        base = Pointer::new(base)
                            .gep(
                                AstValue::new(arg),
                                AstType::new(element_type),
                                builder,
                                &entry,
                            )
                            .into_mlir();
                        index += 1;
                        current = array_type.element_type();
                    }
                    SlangType::FixedSizeArray(array_type) => {
                        let arg: Value<'context, 'block> = entry
                            .argument(index)
                            .expect("argument index is within the block signature")
                            .into();
                        let element_type = AstType::resolve(
                            &array_type.element_type(),
                            LocationPolicy::Declared(Some(location)),
                            builder,
                        );
                        let length = AstValue::uint256(array_type.size() as i64, builder, &entry)
                            .into_mlir();
                        let in_bounds = AstValue::new(arg)
                            .compare(AstValue::new(length), CmpPredicate::Lt, builder, &entry)
                            .into_mlir();
                        mlir_op_void!(builder, &entry, RequireOperation.cond(in_bounds).args(&[]));
                        base = Pointer::new(base)
                            .gep(
                                AstValue::new(arg),
                                AstType::new(element_type),
                                builder,
                                &entry,
                            )
                            .into_mlir();
                        index += 1;
                        current = array_type.element_type();
                    }
                    _ => break,
                }
            }
            return_loaded(base, &struct_plan, result_type, builder, &entry);
            return;
        }

        // A no-argument struct getter expands the struct's returnable members.
        if let SlangType::Struct(struct_type) = &declared_type
            && let Definition::Struct(struct_definition) = struct_type.definition()
        {
            let struct_mlir_type = AstType::resolve(
                &declared_type,
                LocationPolicy::Declared(Some(location)),
                builder,
            );
            if let Some(plan) = struct_definition.struct_getter_layout(struct_mlir_type, builder) {
                let result_types: Vec<Type<'context>> =
                    plan.iter().map(|(_, _, result)| *result).collect();
                let container_type = AstType::resolve_state_variable(
                    &state_variable.get_type().expect("slang validated"),
                    builder,
                );
                let signature = state_variable
                    .compute_canonical_signature()
                    .expect("slang validated");
                let selector = state_variable.compute_selector().expect("slang validated");
                let entry = Function::new(signature, Vec::new(), result_types).define(
                    Some(selector),
                    StateMutability::View,
                    None,
                    None,
                    builder,
                    &block,
                );
                let base =
                    Pointer::addr_of(&slot.name, AstType::new(container_type), builder, &entry)
                        .into_mlir();
                return_loaded(base, &Some(plan), struct_mlir_type, builder, &entry);
                return;
            }
        }

        // A scalar / reference getter reads the variable's slot directly.
        let signature = state_variable
            .compute_canonical_signature()
            .expect("slang validated");
        let selector = state_variable.compute_selector().expect("slang validated");
        let element_type = AstType::resolve_state_variable(
            &state_variable.get_type().expect("slang validated"),
            builder,
        );
        // A reference-typed variable is addressed by the reference type itself in
        // storage; a value type by a `!sol.ptr<T, _>`.
        let address_type = if declared_type.is_reference_type() {
            element_type
        } else {
            AstType::pointer(builder.context, element_type, location).into_mlir()
        };
        let entry = Function::new(signature, Vec::new(), vec![element_type]).define(
            Some(selector),
            StateMutability::View,
            None,
            None,
            builder,
            &block,
        );
        let storage_ref =
            Pointer::addr_of(&slot.name, AstType::new(address_type), builder, &entry).into_mlir();
        let value = if declared_type.is_reference_type() {
            storage_ref
        } else {
            Pointer::new(storage_ref)
                .load(AstType::new(element_type), builder, &entry)
                .into_mlir()
        };
        mlir_op_void!(builder, &entry, ReturnOperation.operands(&[value]));
    }
}
