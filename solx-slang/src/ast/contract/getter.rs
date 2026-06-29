//!
//! Public state-variable getter synthesis.
//!
//! Solidity synthesises an external accessor for every `public` state variable; the variable emits its own.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::Function;
use solx_mlir::StateMutability;
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
    /// The struct's returnable members `(field index, member type, ABI result type)`: mappings and
    /// arrays are skipped, a nested struct is returned whole as its memory ABI tuple. `None` if no
    /// member is returnable.
    fn struct_getter_layout<'context>(
        &self,
        struct_mlir_type: Type<'context>,
        context: &Context<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>>;
}

impl StructGetterLayout for StructDefinition {
    fn struct_getter_layout<'context>(
        &self,
        struct_mlir_type: Type<'context>,
        context: &Context<'context>,
    ) -> Option<Vec<(u64, Type<'context>, Type<'context>)>> {
        let mut plan = Vec::new();
        for (member_index, member) in self.members().iter().enumerate() {
            let member_slang = member.get_type()?;
            let member_type = AstType::new(struct_mlir_type)
                .element_type(member_index)
                .into_mlir();
            let result_member_type = match &member_slang {
                SlangType::Mapping(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => {
                    continue;
                }
                SlangType::String(_) | SlangType::Bytes(_) => {
                    AstType::string(context.mlir(), DataLocation::Memory).into_mlir()
                }
                SlangType::Struct(_) => {
                    AstType::resolve(&member_slang, LocationPolicy::ForceMemory, context)
                }
                _ => member_type,
            };
            plan.push((member_index as u64, member_type, result_member_type));
        }
        if plan.is_empty() {
            return None;
        }
        Some(plan)
    }
}

/// The resolved signature of a keyed getter: `(input_types, result_types, struct_plan,
/// terminal_is_reference)`.
type KeyedGetterSignature<'context> = (
    Vec<Type<'context>>,
    Vec<Type<'context>>,
    Option<Vec<(u64, Type<'context>, Type<'context>)>>,
    bool,
);

/// The external ABI signature of a `public` state variable's synthesised getter,
/// shared by the call-position getter (`this.m(key)`), the getter-as-function-pointer
/// value (`fp = this.m`), and the published method identifiers.
pub trait GetterSignature {
    /// Returns `(parameter_types, return_types)`: scalar `() -> (T)`, mapping
    /// `(K) -> (V)`, array `(uint256) -> (element)`, struct `() -> (flattened
    /// members)`. `None` for a getter with no flattenable result.
    fn getter_signature<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)>;

    /// The multi-level signature of a keyed (`mapping`/array) getter: the re-walk's
    /// `(input_types, result_types)`, plus the terminal's `struct_plan` and whether it
    /// is a reference (Memory) leaf. Shared by [`GetterSignature::getter_signature`]
    /// and the getter body, so the two can never disagree. `None` for a
    /// non-keyed type or a terminal with no flattenable getter.
    fn keyed_getter_signature<'context>(
        &self,
        location: DataLocation,
        context: &Context<'context>,
    ) -> Option<KeyedGetterSignature<'context>>;
}

impl GetterSignature for StateVariableDefinition {
    fn getter_signature<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)> {
        self.get_type()
            .and_then(|declared_type| match &declared_type {
                SlangType::Mapping(_) | SlangType::Array(_) | SlangType::FixedSizeArray(_) => self
                    .keyed_getter_signature(DataLocation::Storage, context)
                    .map(|(input_types, result_types, _, _)| (input_types, result_types)),
                SlangType::Struct(struct_type) => {
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        return None;
                    };
                    let struct_mlir_type = AstType::resolve(
                        &declared_type,
                        LocationPolicy::Declared(Some(DataLocation::Storage)),
                        context,
                    );
                    let plan = struct_definition.struct_getter_layout(struct_mlir_type, context)?;
                    let return_types = plan
                        .iter()
                        .map(|(_, _, result_type)| *result_type)
                        .collect();
                    Some((Vec::new(), return_types))
                }
                other if other.is_reference_type() => Some((
                    Vec::new(),
                    vec![AstType::resolve(
                        other,
                        LocationPolicy::ForceMemory,
                        context,
                    )],
                )),
                other => Some((
                    Vec::new(),
                    vec![AstType::resolve(
                        other,
                        LocationPolicy::Declared(None),
                        context,
                    )],
                )),
            })
    }

    fn keyed_getter_signature<'context>(
        &self,
        location: DataLocation,
        context: &Context<'context>,
    ) -> Option<KeyedGetterSignature<'context>> {
        let mut input_types: Vec<Type<'context>> = Vec::new();
        let mut terminal = self.get_type()?;
        loop {
            match &terminal {
                SlangType::Mapping(mapping_type) => {
                    let key = mapping_type.key_type();
                    let key_type = if key.is_reference_type() {
                        AstType::string(context.mlir(), DataLocation::Memory).into_mlir()
                    } else {
                        AstType::resolve(&key, LocationPolicy::Declared(Some(location)), context)
                    };
                    input_types.push(key_type);
                    terminal = mapping_type.value_type();
                }
                SlangType::Array(array_type) => {
                    input_types.push(
                        AstType::unsigned(context.mlir(), solx_utils::BIT_LENGTH_FIELD).into_mlir(),
                    );
                    terminal = array_type.element_type();
                }
                SlangType::FixedSizeArray(array_type) => {
                    input_types.push(
                        AstType::unsigned(context.mlir(), solx_utils::BIT_LENGTH_FIELD).into_mlir(),
                    );
                    terminal = array_type.element_type();
                }
                _ => break,
            }
        }
        if input_types.is_empty() {
            return None;
        }
        let terminal_is_reference = matches!(&terminal, SlangType::String(_) | SlangType::Bytes(_));
        let result_type = if terminal_is_reference {
            AstType::resolve(&terminal, LocationPolicy::ForceMemory, context)
        } else {
            AstType::resolve(&terminal, LocationPolicy::Declared(Some(location)), context)
        };
        let struct_plan = match &terminal {
            SlangType::Struct(struct_type) => {
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    return None;
                };
                Some(struct_definition.struct_getter_layout(result_type, context)?)
            }
            SlangType::String(_) | SlangType::Bytes(_) => None,
            _ if terminal.is_reference_type() => return None,
            _ => None,
        };
        let result_types: Vec<Type<'context>> = match &struct_plan {
            Some(plan) => plan.iter().map(|(_, _, result)| *result).collect(),
            None => vec![result_type],
        };
        Some((
            input_types,
            result_types,
            struct_plan,
            terminal_is_reference,
        ))
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for StateVariableDefinition {
    type Output = ();

    /// Emits the auto-generated external accessor for this `public` state variable into the contract body.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) {
        /// Emits the terminal `sol.return` from `base` (a struct expands to its members, a scalar loads one value).
        fn return_loaded<'context, 'block>(
            base: Value<'context, 'block>,
            struct_plan: &Option<Vec<(u64, Type<'context>, Type<'context>)>>,
            result_type: Type<'context>,
            is_reference: bool,
            context: &Context<'context>,
            entry: &BlockRef<'context, 'block>,
        ) {
            match struct_plan {
                Some(plan) => {
                    let mut values = Vec::new();
                    for (member_index, member_type, result_member_type) in plan {
                        let index_value = AstValue::constant(
                            *member_index as i64,
                            AstType::unsigned(context.mlir(), solx_utils::BIT_LENGTH_X64),
                            context,
                            entry,
                        );
                        let address = Pointer::new(base)
                            .gep(
                                index_value,
                                AstType::new(*member_type),
                                false,
                                context,
                                entry,
                            )
                            .into_mlir();
                        let value = if member_type == result_member_type {
                            Pointer::new(address)
                                .load(AstType::new(*result_member_type), context, entry)
                                .into_mlir()
                        } else {
                            AstValue::new(address)
                                .cast(AstType::new(*result_member_type), context, entry)
                                .into_mlir()
                        };
                        values.push(value);
                    }
                    mlir_op_void!(context, entry, ReturnOperation.operands(&values));
                }
                None => {
                    let value = if is_reference {
                        AstValue::new(base)
                            .cast(AstType::new(result_type), context, entry)
                            .into_mlir()
                    } else {
                        Pointer::new(base)
                            .load(AstType::new(result_type), context, entry)
                            .into_mlir()
                    };
                    mlir_op_void!(context, entry, ReturnOperation.operands(&[value]));
                }
            }
        }

        let state_variable = self;
        let state = context.state;

        let abi = match state_variable.compute_abi_entry() {
            Some(AbiEntry::Function(abi)) => abi,
            _ => return,
        };

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
            let slang_type = state_variable.get_type().expect("slang validated");
            let element_type = AstType::resolve(&slang_type, LocationPolicy::ForceMemory, state);
            let entry = Function::new(signature, Vec::new(), vec![element_type]).define(
                Some(selector),
                StateMutability::Pure,
                None,
                None,
                state,
                &block,
            );
            let BlockAnd {
                value,
                block: entry,
            } = initializer.emit_as(element_type, context, entry);
            mlir_op_void!(
                state,
                &entry,
                ReturnOperation.operands(&[value.into_mlir()])
            );
            return;
        }

        let Some(slot) = context.storage_layout.get(&state_variable.node_id()) else {
            return;
        };
        let location = slot.location;
        let declared_type = state_variable.get_type().expect("slang validated");

        if !abi.inputs().is_empty() {
            let signature = state_variable
                .compute_canonical_signature()
                .expect("slang validated");
            let selector = state_variable.compute_selector().expect("slang validated");

            let Some((input_types, result_types, struct_plan, terminal_is_reference)) =
                state_variable.keyed_getter_signature(location, state)
            else {
                return;
            };
            let container_type = AstType::resolve_state_variable(
                &state_variable.get_type().expect("slang validated"),
                state,
            );
            let result_type = result_types[0];
            let entry = Function::new(signature, input_types, result_types).define(
                Some(selector),
                StateMutability::View,
                None,
                None,
                state,
                &block,
            );
            let mut base =
                Pointer::addr_of(&slot.name, AstType::new(container_type), state, &entry)
                    .into_mlir();
            // Re-walk the nesting; an array index passes `no_panic_bounds` so an out-of-bounds access
            // plain-reverts rather than `Panic(0x32)`.
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
                            state,
                        );
                        let level_type = AstType::new(resolved_value)
                            .address_type(location, state.mlir())
                            .into_mlir();
                        base = Pointer::new(base)
                            .entry(AstValue::new(arg), AstType::new(level_type), state, &entry)
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
                            state,
                        );
                        base = Pointer::new(base)
                            .gep(
                                AstValue::new(arg),
                                AstType::new(element_type),
                                true,
                                state,
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
                            state,
                        );
                        base = Pointer::new(base)
                            .gep(
                                AstValue::new(arg),
                                AstType::new(element_type),
                                true,
                                state,
                                &entry,
                            )
                            .into_mlir();
                        index += 1;
                        current = array_type.element_type();
                    }
                    _ => break,
                }
            }
            return_loaded(
                base,
                &struct_plan,
                result_type,
                terminal_is_reference,
                state,
                &entry,
            );
            return;
        }

        if let SlangType::Struct(struct_type) = &declared_type
            && let Definition::Struct(struct_definition) = struct_type.definition()
        {
            let struct_mlir_type = AstType::resolve(
                &declared_type,
                LocationPolicy::Declared(Some(location)),
                state,
            );
            if let Some(plan) = struct_definition.struct_getter_layout(struct_mlir_type, state) {
                let result_types: Vec<Type<'context>> =
                    plan.iter().map(|(_, _, result)| *result).collect();
                let container_type = AstType::resolve_state_variable(
                    &state_variable.get_type().expect("slang validated"),
                    state,
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
                    state,
                    &block,
                );
                let base =
                    Pointer::addr_of(&slot.name, AstType::new(container_type), state, &entry)
                        .into_mlir();
                return_loaded(base, &Some(plan), struct_mlir_type, false, state, &entry);
                return;
            }
        }

        let signature = state_variable
            .compute_canonical_signature()
            .expect("slang validated");
        let selector = state_variable.compute_selector().expect("slang validated");
        let element_type = AstType::resolve_state_variable(
            &state_variable.get_type().expect("slang validated"),
            state,
        );
        let address_type = AstType::new(element_type)
            .address_type(location, state.mlir())
            .into_mlir();
        let is_reference = declared_type.is_reference_type();
        let return_type = if is_reference {
            AstType::resolve(&declared_type, LocationPolicy::ForceMemory, state)
        } else {
            element_type
        };
        let entry = Function::new(signature, Vec::new(), vec![return_type]).define(
            Some(selector),
            StateMutability::View,
            None,
            None,
            state,
            &block,
        );
        let value = if is_reference {
            let storage_reference =
                Pointer::addr_of(&slot.name, AstType::new(address_type), state, &entry).into_mlir();
            AstValue::new(storage_reference)
                .cast(AstType::new(return_type), state, &entry)
                .into_mlir()
        } else {
            slot.load(state, element_type, &entry)
        };
        mlir_op_void!(state, &entry, ReturnOperation.operands(&[value]));
    }
}
