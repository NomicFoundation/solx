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
use solx_mlir::Pointer;
use solx_mlir::StateMutability;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::ReturnOperation;
use solx_utils::DataLocation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::getter::keyed_signature::KeyedSignature;
use crate::ast::contract::getter::member::Member;
use crate::ast::contract::getter::signature::Signature;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for StateVariableDefinition {
    type Output = ();

    /// Emits the auto-generated external accessor for this `public` state variable into the contract
    /// body. A non-`public` variable, an `immutable` variable, or one carrying no returnable ABI
    /// entry emits nothing.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) {
        let state = context.state;

        let Some(AbiEntry::Function(abi)) = self.compute_abi_entry() else {
            return;
        };
        let signature = self
            .compute_canonical_signature()
            .expect("a public accessor has a canonical signature");
        let selector = self
            .compute_selector()
            .expect("a public accessor has a selector");

        if matches!(self.mutability(), StateVariableMutability::Constant) {
            let initializer = self
                .value()
                .expect("a constant state variable is initialised");
            let (_, result_types) = self
                .getter_signature(state)
                .expect("a constant public accessor returns a value");
            let element_type = result_types[0];
            let entry = Function::new(signature, Vec::new(), result_types).define(
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
            mlir_op_void!(state, &entry, ReturnOperation.operands(&[value]));
            return;
        }

        if matches!(self.mutability(), StateVariableMutability::Immutable) {
            return;
        }

        let Some(slot) = context.storage_layout.get(&self.node_id()) else {
            return;
        };
        let declared_type = self.get_type().expect("slang types every state variable");

        if !abi.inputs().is_empty() {
            let KeyedSignature {
                input_types,
                result_types,
                members,
                terminal_is_reference,
            } = self
                .keyed_signature(state)
                .expect("a keyed public accessor has a returnable keyed signature");
            let container_type = TypeConversion::resolve_state_variable_type(self, state);
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
                        let resolved_value = TypeConversion::resolve_slang_type(
                            &value_slang,
                            Some(DataLocation::Storage),
                            state,
                        );
                        let entry_type = ExpressionContext::address_type(
                            state,
                            resolved_value,
                            DataLocation::Storage,
                            &value_slang,
                        );
                        base = Pointer::new(base)
                            .map(AstValue::new(argument), AstType::new(entry_type), state, &entry)
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
                let element_type = TypeConversion::resolve_slang_type(
                    &element_slang,
                    Some(DataLocation::Storage),
                    state,
                );
                base = Pointer::new(base)
                    .gep(AstValue::new(argument), AstType::new(element_type), true, state, &entry)
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
                            .data_loc_cast(AstType::new(result_type), state, &entry)
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
            let struct_mlir_type = TypeConversion::resolve_slang_type(
                &declared_type,
                Some(DataLocation::Storage),
                state,
            );
            if let Some(members) = Member::layout(&struct_definition, struct_mlir_type, state) {
                let result_types: Vec<Type<'context>> =
                    members.iter().map(|member| member.result_type).collect();
                let container_type = TypeConversion::resolve_state_variable_type(self, state);
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

        let element_type = TypeConversion::resolve_state_variable_type(self, state);
        let is_reference = declared_type.is_reference_type();
        let (_, result_types) = self
            .getter_signature(state)
            .expect("a scalar public accessor returns a value");
        let return_type = result_types[0];
        let entry = Function::new(signature, Vec::new(), result_types).define(
            Some(selector),
            StateMutability::View,
            None,
            None,
            state,
            &block,
        );
        let value = if is_reference {
            let storage_reference =
                Pointer::addr_of(&slot.name, AstType::new(element_type), state, &entry).into_mlir();
            AstValue::new(storage_reference)
                .data_loc_cast(AstType::new(return_type), state, &entry)
                .into_mlir()
        } else {
            context.emit_storage_load(slot, element_type, &entry)
        };
        mlir_op_void!(state, &entry, ReturnOperation.operands(&[value]));
    }
}
