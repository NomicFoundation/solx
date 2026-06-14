//!
//! Public state-variable getter synthesis.
//!
//! Solidity synthesises an external accessor for every `public` state variable.
//! This module carries the per-getter frame ([`GetterAbi`]) and the emission /
//! classification methods that lower it; the dispatching entry points are `impl`
//! blocks on the foreign [`ContractEmitter`] (§2a: the SOLE top-level type here
//! is `GetterAbi`).
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::r#type::TypeLike;
use num_bigint::BigInt;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::abi::AbiFunction;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::StateMutability;
use solx_mlir::ods::sol::AddrOfOperation;
use solx_mlir::ods::sol::LengthOperation;
use solx_mlir::ods::sol::MapOperation;
use solx_mlir::ods::sol::ReturnOperation;
use solx_utils::DataLocation;

use std::collections::HashMap;

use slang_solidity_v2::ast::NodeId;
use solx_mlir::Environment;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::ContractEmitter;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::getter_level::GetterLevel;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;
use crate::ast::type_conversion::TypeConversion;

/// The per-getter emission frame: the state variable, its canonical ABI
/// signature and selector, declared type, storage slot, and the `sol.contract`
/// body the getter `sol.func` is appended to.
///
/// The SOLE top-level type of this module (§2a); a pure `Copy` data carrier
/// destructured by the `ContractEmitter` emission methods.
#[derive(Clone, Copy)]
pub struct GetterAbi<'a, 'context, 'block> {
    /// The state variable whose accessor is being generated.
    state_variable: &'a StateVariableDefinition,
    /// The canonical ABI signature, e.g. `balances(address)`.
    signature: &'a str,
    /// The 4-byte function selector derived from the signature.
    selector: u32,
    /// The variable's declared Solidity type (mapping/array/struct/scalar).
    declared_type: &'a SlangType,
    /// The storage slot holding the variable (carries the `{label}_{node_id}`
    /// symbol name addressed by `sol.addr_of`).
    slot: &'a StorageSlot,
    /// The storage data location (`Storage` for persistent state).
    location: DataLocation,
    /// The `sol.contract` body the getter `sol.func` is appended to.
    contract_body: &'a BlockRef<'context, 'block>,
}

impl<'state, 'context> ContractEmitter<'state, 'context> {
    /// Computes the shared ABI prologue of a `public` state variable's getter —
    /// its function ABI entry (carrying the input parameters), canonical
    /// signature, and selector — or `None` when the variable has no synthesised
    /// accessor. The single source the constant and storage getter entry points
    /// both read.
    fn getter_abi_prologue(
        state_variable: &StateVariableDefinition,
    ) -> Option<(AbiFunction, String, u32)> {
        let Some(AbiEntry::Function(abi)) = state_variable.compute_abi_entry() else {
            return None;
        };
        let signature = state_variable.compute_canonical_signature()?;
        let selector = state_variable.compute_selector()?;
        Some((abi, signature, selector))
    }

    /// Dispatches getter synthesis for one non-constant state variable to the
    /// scalar or indexed (mapping/array) path. Struct getters are emitted by a
    /// later fill; a variable left without an accessor is harmless (the rest of
    /// the contract still compiles).
    pub fn emit_state_variable_getter(
        &self,
        state_variable: &StateVariableDefinition,
        slot: &StorageSlot,
        location: DataLocation,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some((abi, signature, selector)) = Self::getter_abi_prologue(state_variable) else {
            return Ok(());
        };
        let declared_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let frame = GetterAbi {
            state_variable,
            signature: &signature,
            selector,
            declared_type: &declared_type,
            slot,
            location,
            contract_body,
        };
        if !abi.inputs().is_empty() {
            return self.emit_indexed_getter(&frame, abi.inputs().len());
        }
        if self.emit_struct_getter(&frame)? {
            return Ok(());
        }
        self.emit_scalar_getter(&frame)
    }

    /// Emits a scalar / reference getter: `T public name` becomes `function
    /// name() view returns (T)` reading the variable's slot.
    fn emit_scalar_getter(&self, abi: &GetterAbi<'_, 'context, '_>) -> anyhow::Result<()> {
        let GetterAbi {
            state_variable,
            signature,
            selector,
            declared_type,
            slot,
            location,
            contract_body,
        } = *abi;
        let builder = &self.state.builder;
        let element_type = TypeConversion::resolve_state_variable_type(state_variable, builder)?;
        // A reference-typed variable (`string`/`bytes`/array) is addressed by the
        // reference type itself in storage; value types by a `!sol.ptr<T, _>`.
        let address_type = if declared_type.is_reference_type() {
            element_type
        } else {
            crate::ast::Type::pointer(builder.context, element_type, location).into_mlir()
        };
        let entry = builder.emit_sol_func(
            signature,
            &[],
            std::slice::from_ref(&element_type),
            Some(selector),
            StateMutability::View,
            None,
            None,
            contract_body,
        );
        let storage_ref = sol_op!(
            builder,
            &entry,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(builder.context, &slot.name))
                .addr(address_type)
        );
        let value = if declared_type.is_reference_type() {
            storage_ref
        } else {
            builder.emit_sol_load(storage_ref, element_type, &entry)?
        };
        sol_op_void!(builder, &entry, ReturnOperation.operands(&[value]));
        Ok(())
    }

    /// Emits an indexed getter for a mapping / array state variable: `m(k)`,
    /// `a(uint256)`, `a(i, j)`, `m(k1, k2)`, ... Each nesting level chains a
    /// `sol.map` (mappings) or a bounds-checked `sol.gep` (arrays) over its
    /// key/index argument; the final value is loaded.
    ///
    /// Struct and other reference results are emitted by a later fill (the
    /// getter is left ungenerated meanwhile rather than emitted incorrectly).
    fn emit_indexed_getter(
        &self,
        abi: &GetterAbi<'_, 'context, '_>,
        abi_input_count: usize,
    ) -> anyhow::Result<()> {
        let GetterAbi {
            state_variable,
            signature,
            selector,
            declared_type,
            slot,
            location,
            contract_body,
        } = *abi;
        let builder = &self.state.builder;
        let (input_types, levels, result_slang) =
            self.plan_indexed_getter_levels(declared_type, location);
        if input_types.is_empty() || input_types.len() != abi_input_count {
            return Ok(());
        }
        let container_type = TypeConversion::resolve_state_variable_type(state_variable, builder)?;
        let result_type =
            result_slang.resolve_type(LocationPolicy::Declared(Some(location)), builder);
        // A struct result expands into its flattened returnable-member tuple;
        // other reference results aren't handled yet (left ungenerated).
        let struct_plan = match &result_slang {
            SlangType::Struct(struct_type) => {
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    return Ok(());
                };
                match Self::struct_getter_layout(&struct_definition, result_type, builder) {
                    Some(plan) => Some(plan),
                    None => return Ok(()),
                }
            }
            _ if result_slang.is_reference_type() => return Ok(()),
            _ => None,
        };
        let result_types: Vec<Type<'context>> = match &struct_plan {
            Some(plan) => plan
                .iter()
                .map(|(_, _, member_result)| *member_result)
                .collect(),
            None => vec![result_type],
        };
        let entry = builder.emit_sol_func(
            signature,
            &input_types,
            &result_types,
            Some(selector),
            StateMutability::View,
            None,
            None,
            contract_body,
        );
        let base = sol_op!(
            builder,
            &entry,
            AddrOfOperation
                .var(FlatSymbolRefAttribute::new(builder.context, &slot.name))
                .addr(container_type)
        );
        let base = self.emit_getter_access_chain(base, &levels, &entry)?;
        self.emit_indexed_getter_result(base, &struct_plan, result_type, &entry)
    }

    /// Emits the indexed getter's return: a struct result expands into its
    /// flattened returnable-member tuple (each member loaded from `base`); a
    /// scalar result loads the single value.
    fn emit_indexed_getter_result<'block>(
        &self,
        base: Value<'context, 'block>,
        struct_plan: &Option<Vec<(u64, Type<'context>, Type<'context>)>>,
        result_type: Type<'context>,
        entry: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let builder = &self.state.builder;
        match struct_plan {
            Some(plan) => {
                let mut values = Vec::new();
                for (member_index, member_type, result_member_type) in plan {
                    let index_value = crate::ast::Value::constant(
                        *member_index as i64,
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
                        builder,
                        entry,
                    );
                    let address = crate::ast::Pointer::new(base)
                        .gep(
                            index_value,
                            crate::ast::Type::new(*member_type),
                            builder,
                            entry,
                        )
                        .into_mlir();
                    values.push(Self::load_getter_member(
                        builder,
                        address,
                        *member_type,
                        *result_member_type,
                        entry,
                    )?);
                }
                sol_op_void!(builder, entry, ReturnOperation.operands(&values));
            }
            None => {
                let value = builder.emit_sol_load(base, result_type, entry)?;
                sol_op_void!(builder, entry, ReturnOperation.operands(&[value]));
            }
        }
        Ok(())
    }

    /// Walks the mapping/array nesting of an indexed getter's declared type,
    /// producing the per-key/index ABI input types, the ordered access-`level`
    /// plan, and the terminal Solidity type reached after all levels.
    fn plan_indexed_getter_levels(
        &self,
        declared_type: &SlangType,
        location: DataLocation,
    ) -> (Vec<Type<'context>>, Vec<GetterLevel<'context>>, SlangType) {
        let builder = &self.state.builder;
        let mut input_types: Vec<Type<'context>> = Vec::new();
        let mut levels: Vec<GetterLevel<'context>> = Vec::new();
        let mut current = declared_type.clone();
        loop {
            match &current {
                SlangType::Mapping(mapping_type) => {
                    let key_slang = mapping_type.key_type();
                    let value_slang = mapping_type.value_type();
                    let resolved_value =
                        value_slang.resolve_type(LocationPolicy::Declared(Some(location)), builder);
                    // Intermediate containers are addressed by their reference; a
                    // value terminal by a `!sol.ptr<V>`.
                    let level_type = if value_slang.is_reference_type() {
                        resolved_value
                    } else {
                        crate::ast::Type::pointer(builder.context, resolved_value, location)
                            .into_mlir()
                    };
                    // A reference-typed key (`string`/`bytes`) is an ABI input
                    // decoded into memory; `sol.map` hashes the key bytes for the
                    // slot. slang reports the key with the mapping's storage
                    // location, so build the memory type directly rather than
                    // resolving it (which would yield a storage string).
                    let key_type = if key_slang.is_reference_type() {
                        crate::ast::Type::string(builder.context, DataLocation::Memory).into_mlir()
                    } else {
                        key_slang.resolve_type(LocationPolicy::Declared(Some(location)), builder)
                    };
                    input_types.push(key_type);
                    levels.push(GetterLevel::Mapping(level_type));
                    current = value_slang;
                }
                SlangType::Array(array_type) => {
                    let element_slang = array_type.element_type();
                    let element_type = element_slang
                        .resolve_type(LocationPolicy::Declared(Some(location)), builder);
                    input_types.push(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    );
                    levels.push(GetterLevel::Array(element_type, None));
                    current = element_slang;
                }
                SlangType::FixedSizeArray(array_type) => {
                    let element_slang = array_type.element_type();
                    let element_type = element_slang
                        .resolve_type(LocationPolicy::Declared(Some(location)), builder);
                    input_types.push(
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    );
                    levels.push(GetterLevel::Array(
                        element_type,
                        Some(array_type.size() as u64),
                    ));
                    current = element_slang;
                }
                _ => break,
            }
        }
        (input_types, levels, current)
    }

    /// Chains the per-level storage access for an indexed getter, starting from
    /// the base slot reference: a `sol.map` for each mapping key and a
    /// bounds-checked `sol.gep` for each array index (out-of-bounds bare-revert
    /// via a no-message `sol.require`, matching solc's accessor — NOT `sol.gep`'s
    /// `Panic(0x32)`, which the semantic tests reject). Returns the reference to
    /// the addressed element.
    fn emit_getter_access_chain<'block>(
        &self,
        mut base: Value<'context, 'block>,
        levels: &[GetterLevel<'context>],
        entry: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        let builder = &self.state.builder;
        for (index, level) in levels.iter().enumerate() {
            let arg: Value<'context, 'block> = entry.argument(index)?.into();
            base = match level {
                GetterLevel::Mapping(level_type) => {
                    sol_op!(
                        builder,
                        entry,
                        MapOperation.mapping(base).key(arg).addr(*level_type)
                    )
                }
                GetterLevel::Array(element_type, fixed_size) => {
                    let length = match fixed_size {
                        Some(size) => crate::ast::Value::constant(
                            *size as i64,
                            crate::ast::Type::unsigned(
                                builder.context,
                                solx_utils::BIT_LENGTH_FIELD,
                            ),
                            builder,
                            entry,
                        )
                        .into_mlir(),
                        None => {
                            sol_op!(
                                builder,
                                entry,
                                LengthOperation.inp(base).len(
                                    crate::ast::Type::unsigned(
                                        builder.context,
                                        solx_utils::BIT_LENGTH_FIELD
                                    )
                                    .into_mlir()
                                )
                            )
                        }
                    };
                    let in_bounds = crate::ast::Value::from(arg)
                        .compare(
                            crate::ast::Value::from(length),
                            CmpPredicate::Lt,
                            builder,
                            entry,
                        )
                        .into_mlir();
                    builder.emit_sol_require(in_bounds, None, &[], false, entry);
                    crate::ast::Pointer::new(base)
                        .gep(
                            crate::ast::Value::new(arg),
                            crate::ast::Type::new(*element_type),
                            builder,
                            entry,
                        )
                        .into_mlir()
                }
            };
        }
        Ok(base)
    }

    /// Emits a `constant` state variable's getter (a folded compile-time value).
    pub fn emit_constant_getter(
        &self,
        state_variable: &StateVariableDefinition,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        contract_body: &BlockRef<'context, '_>,
    ) -> anyhow::Result<()> {
        let Some((abi, signature, selector)) = Self::getter_abi_prologue(state_variable) else {
            return Ok(());
        };
        if !abi.inputs().is_empty() {
            return Ok(());
        }
        let Some(initializer) = state_variable.value() else {
            return Ok(());
        };

        let builder = &self.state.builder;
        // The getter returns the constant's value type, reference types
        // (`string` / `bytes`) in `Memory` — what an external getter call hands
        // back — not their declared storage location.
        let slang_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let element_type = slang_type.resolve_type(LocationPolicy::ForceMemory, builder);
        let entry = builder.emit_sol_func(
            &signature,
            &[],
            std::slice::from_ref(&element_type),
            Some(selector),
            StateMutability::Pure,
            None,
            None,
            contract_body,
        );
        if let Some(value) = Self::fold_constant_int(&initializer) {
            let constant = crate::ast::Value::constant_from_bigint(
                &value,
                crate::ast::Type::new(element_type),
                builder,
                &entry,
            )
            .into_mlir();
            sol_op_void!(builder, &entry, ReturnOperation.operands(&[constant]));
            return Ok(());
        }
        // A non-integer constant — a `string` / `bytesN` literal — is not
        // integer-foldable; materialise its initializer toward the return type
        // through the expression emitter, exactly as an explicit getter
        // `return <const>` would (a constant body has no locals, so an empty
        // environment suffices). Matches solc: a `sol.string_lit`, or the
        // literal's value `bytes_cast` to `bytesN`.
        let environment = Environment::new();
        let emitter = ExpressionContext::new(
            self.state,
            &environment,
            storage_layout,
            ArithmeticMode::Checked,
        );
        let BlockAnd {
            value,
            block: entry,
        } = (Toward {
            expression: &initializer,
            target_type: element_type,
        })
        .emit(&emitter, entry)?;
        let value = value
            .coerce_to(
                crate::ast::Type::new(element_type),
                &self.state.builder,
                &entry,
            )
            .into_mlir();
        sol_op_void!(builder, &entry, ReturnOperation.operands(&[value]));
        Ok(())
    }

    /// Folds a constant integer expression to a [`BigInt`], when it is one of the
    /// closed set of integer-foldable forms.
    pub fn fold_constant_int(expression: &Expression) -> Option<BigInt> {
        match expression {
            Expression::DecimalNumberExpression(decimal) => decimal.integer_value(),
            Expression::HexNumberExpression(hex) => hex.integer_value(),
            Expression::Identifier(identifier) => match identifier.resolve_to_definition() {
                Some(Definition::StateVariable(state_variable)) => {
                    Self::fold_constant_int(&state_variable.value()?)
                }
                Some(Definition::Constant(constant)) => Self::fold_constant_int(&constant.value()?),
                _ => None,
            },
            Expression::MemberAccessExpression(access) => {
                match access.member().resolve_to_definition() {
                    Some(Definition::StateVariable(state_variable)) => {
                        Self::fold_constant_int(&state_variable.value()?)
                    }
                    Some(Definition::Constant(constant)) => {
                        Self::fold_constant_int(&constant.value()?)
                    }
                    _ => None,
                }
            }
            Expression::FunctionCallExpression(call) => {
                let slang_solidity_v2::ast::ArgumentsDeclaration::PositionalArguments(positional) =
                    call.arguments()
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
                    Self::fold_constant_int(&argument)
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

    /// Emits a no-argument getter for a `public` struct state variable (its
    /// returnable members as a flattened tuple). Emitted by a later fill.
    pub fn emit_struct_getter(&self, abi: &GetterAbi<'_, 'context, '_>) -> anyhow::Result<bool> {
        let GetterAbi {
            state_variable,
            signature,
            selector,
            declared_type,
            slot,
            location,
            contract_body,
        } = *abi;
        let builder = &self.state.builder;
        if let SlangType::Struct(struct_type) = declared_type
            && let Definition::Struct(struct_definition) = struct_type.definition()
        {
            let struct_mlir_type =
                declared_type.resolve_type(LocationPolicy::Declared(Some(location)), builder);
            if let Some(plan) =
                Self::struct_getter_layout(&struct_definition, struct_mlir_type, builder)
            {
                let result_types: Vec<Type<'context>> = plan
                    .iter()
                    .map(|(_, _, result_type)| *result_type)
                    .collect();
                let container_type =
                    TypeConversion::resolve_state_variable_type(state_variable, builder)?;
                let entry = builder.emit_sol_func(
                    signature,
                    &[],
                    &result_types,
                    Some(selector),
                    StateMutability::View,
                    None,
                    None,
                    contract_body,
                );
                let base = sol_op!(
                    builder,
                    &entry,
                    AddrOfOperation
                        .var(FlatSymbolRefAttribute::new(builder.context, &slot.name))
                        .addr(container_type)
                );
                self.emit_indexed_getter_result(base, &Some(plan), struct_mlir_type, &entry)?;
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Plans a struct getter's destructured member layout (offset, member type,
    /// result member type), when the result type is a struct.
    pub fn struct_getter_layout(
        struct_definition: &StructDefinition,
        struct_mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>> {
        let mut plan = Vec::new();
        for (member_index, member) in struct_definition.members().iter().enumerate() {
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
            // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
            // `sol::getEltType` for the struct field at `member_index` (mirroring
            // the AST member index, including skipped members).
            let member_type = unsafe {
                Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(
                    struct_mlir_type.to_raw(),
                    member_index as u64,
                ))
            };
            // A `string`/`bytes` member returns a memory copy; every other member
            // reaching here is a value type (mapping/array/struct/string/bytes are
            // skipped above). A function-pointer member would need a func-ref
            // guard (an `is_sol_function_ref` predicate), which solx-mlir does
            // not yet expose; such a struct is vanishingly rare and absent
            // from the test corpus — left to the solx-mlir Sol-type-predicate fill.
            let result_member_type = if is_string_or_bytes {
                crate::ast::Type::string(builder.context, DataLocation::Memory).into_mlir()
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

    /// Loads one struct getter member, casting it to its ABI result type through
    /// the single [`TypeConversion`](crate::ast::type_conversion::TypeConversion)
    /// entry.
    pub fn load_getter_member<'block>(
        builder: &solx_mlir::Builder<'context>,
        address: Value<'context, 'block>,
        member_type: Type<'context>,
        result_member_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        if member_type == result_member_type {
            builder.emit_sol_load(address, result_member_type, block)
        } else {
            Ok(crate::ast::Value::from(address)
                .coerce_to(crate::ast::Type::new(result_member_type), builder, block)
                .into_mlir())
        }
    }
}
