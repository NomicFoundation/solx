//!
//! Assignable locations (lvalues) and their loads and stores.
//!
//! Shared by assignment, compound assignment, and the increment/decrement
//! operators — every read-modify-write of a named location resolves through
//! here.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::storage_slot::StorageSlot;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

/// A resolved assignable location.
pub(super) enum Lvalue<'context, 'block> {
    /// A stack slot — a local variable or parameter.
    Stack(Value<'context, 'block>, Type<'context>),
    /// A value-typed state variable storage slot.
    Storage(StorageSlot, Type<'context>),
}

impl<'context, 'block> Lvalue<'context, 'block> {
    /// The declared element type of the location.
    pub(super) fn element_type(&self) -> Type<'context> {
        match self {
            Self::Stack(_, element_type) | Self::Storage(_, element_type) => *element_type,
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Resolves an expression to the location it denotes for assignment.
    ///
    /// Only identifier lvalues — locals, parameters, value-typed state
    /// variables — are supported; index, member, and reference-typed targets
    /// are lowered by later domains.
    pub(super) fn resolve_lvalue(&self, expression: &Expression) -> Lvalue<'context, 'block> {
        let Expression::Identifier(identifier) = expression else {
            unimplemented!("lvalue: {:?}", std::mem::discriminant(expression));
        };
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) =
                    self.environment.variable_with_type(&identifier.name());
                Lvalue::Stack(pointer, element_type)
            }
            Some(Definition::StateVariable(state_variable)) => {
                let declared_type = state_variable
                    .get_type()
                    .expect("binder types every state variable");
                if declared_type.is_reference_type() {
                    unimplemented!("lvalue: reference-typed state variable");
                }
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("every value-typed state variable has a storage slot")
                    .clone();
                let element_type =
                    TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
                Lvalue::Storage(slot, element_type)
            }
            Some(_) => unimplemented!("lvalue binding kind: {}", identifier.name()),
            None => unreachable!("slang resolves every identifier reference"),
        }
    }

    /// Loads the value currently held at an lvalue.
    pub(super) fn emit_lvalue_load(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<Value<'context, 'block>> {
        match lvalue {
            Lvalue::Stack(pointer, element_type) => {
                self.state
                    .builder
                    .emit_sol_load(*pointer, *element_type, block)
            }
            Lvalue::Storage(slot, element_type) => {
                self.emit_storage_load(slot, *element_type, block)
            }
        }
    }

    /// Stores a value to an lvalue.
    pub(super) fn emit_lvalue_store(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) {
        match lvalue {
            Lvalue::Stack(pointer, _) => self.state.builder.emit_sol_store(value, *pointer, block),
            Lvalue::Storage(slot, element_type) => {
                self.emit_storage_store(slot, value, *element_type, block);
            }
        }
    }
}
