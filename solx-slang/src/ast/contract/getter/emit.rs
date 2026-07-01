//!
//! Emits the external accessor body for a `public` state variable.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Function;
use solx_mlir::LocationPolicy;
use solx_mlir::Pointer;
use solx_mlir::StateMutability;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ReturnOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::getter::keyed_signature::KeyedSignature;
use crate::ast::contract::getter::member::Member;
use crate::ast::contract::getter::signature::Signature;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for StateVariableDefinition {
    type Output = ();

    /// Emits the auto-generated external accessor for this `public` state variable into the contract body.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) {
        let state = context.state;

        let Some(AbiEntry::Function(abi)) = self.compute_abi_entry() else {
            return;
        };
        let signature = self.compute_canonical_signature().expect("slang validated");
        let selector = self.compute_selector().expect("slang validated");

        if matches!(self.mutability(), StateVariableMutability::Constant) {
            let initializer = self
                .value()
                .expect("a constant state variable is initialized");
            let slang_type = self.get_type().expect("slang validated");
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

        let slot = context
            .storage_layout
            .get(&self.node_id())
            .expect("a public non-constant state variable has a storage slot");
        let location = slot.location;
        let declared_type = self.get_type().expect("slang validated");

        if !abi.inputs().is_empty() {
            let Some(KeyedSignature {
                input_types,
                result_types,
                members,
                terminal_is_reference,
            }) = self.keyed_signature(location, state)
            else {
                unreachable!("a keyed public accessor has a returnable keyed signature");
            };
            let container_type = AstType::resolve_state_variable(&declared_type, state);
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
            let mut current = declared_type.clone();
            let mut index = 0usize;
            loop {
                let element_slang = match &current {
                    SlangType::Mapping(mapping_type) => {
                        let argument: Value<'context, 'block> = entry
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
                            .address_type(location, state.mlir_context)
                            .into_mlir();
                        base = Pointer::new(base)
                            .map(
                                AstValue::new(argument),
                                AstType::new(level_type),
                                state,
                                &entry,
                            )
                            .into_mlir();
                        index += 1;
                        current = value_slang;
                        continue;
                    }
                    SlangType::Array(array_type) => array_type.element_type(),
                    SlangType::FixedSizeArray(array_type) => array_type.element_type(),
                    _ => break,
                };
                let argument: Value<'context, 'block> = entry
                    .argument(index)
                    .expect("argument index is within the block signature")
                    .into();
                let element_type = AstType::resolve(
                    &element_slang,
                    LocationPolicy::Declared(Some(location)),
                    state,
                );
                // An array index passes `no_panic_bounds` so an out-of-bounds access plain-reverts
                // rather than `Panic(0x32)`.
                base = Pointer::new(base)
                    .gep(
                        AstValue::new(argument),
                        AstType::new(element_type),
                        true,
                        state,
                        &entry,
                    )
                    .into_mlir();
                index += 1;
                current = element_slang;
            }
            let return_values: Vec<_> = match &members {
                Some(members) => members
                    .iter()
                    .map(|member| member.load_from(base, state, &entry))
                    .collect(),
                None => {
                    let value = if terminal_is_reference {
                        AstValue::new(base)
                            .cast(AstType::new(result_type), state, &entry)
                            .into_mlir()
                    } else {
                        Pointer::new(base)
                            .load(AstType::new(result_type), state, &entry)
                            .into_mlir()
                    };
                    vec![value]
                }
            };
            mlir_op_void!(state, &entry, ReturnOperation.operands(&return_values));
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
            if let Some(members) = Member::layout(&struct_definition, struct_mlir_type, state) {
                let result_types: Vec<Type<'context>> =
                    members.iter().map(|member| member.result_type).collect();
                let container_type = AstType::resolve_state_variable(&declared_type, state);
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
                let return_values: Vec<_> = members
                    .iter()
                    .map(|member| member.load_from(base, state, &entry))
                    .collect();
                mlir_op_void!(state, &entry, ReturnOperation.operands(&return_values));
                return;
            }
        }

        let element_type = AstType::resolve_state_variable(&declared_type, state);
        let address_type = AstType::new(element_type)
            .address_type(location, state.mlir_context)
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
