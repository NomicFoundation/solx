//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::storage_slot::StorageSlot;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

/// The resolved left-hand side of an assignment.
enum AssignmentTarget<'context, 'block> {
    /// A stack slot — a local variable or parameter — written via `sol.store`.
    Stack(Value<'context, 'block>, Type<'context>),
    /// A value-typed state variable, written to its storage slot.
    Storage(StorageSlot, Type<'context>),
}

impl<'context, 'block> AssignmentTarget<'context, 'block> {
    /// The declared element type the assigned value is cast to.
    fn element_type(&self) -> Type<'context> {
        match self {
            Self::Stack(_, element_type) | Self::Storage(_, element_type) => *element_type,
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a simple assignment (`=`); the cast right-hand side is stored to
    /// the target and is also the expression's result.
    ///
    /// Compound assignments are lowered by a later domain.
    pub(super) fn emit_assignment(
        &self,
        assignment: &AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        if !matches!(
            assignment.operator(),
            AssignmentExpressionOperator::Equal(_)
        ) {
            unimplemented!("compound assignment lowering");
        }

        let target = self.resolve_assignment_target(&assignment.left_operand());
        let (value, block) = self.emit_value(&assignment.right_operand(), block)?;
        let stored = TypeConversion::from_target_type(target.element_type(), &self.state.builder)
            .emit(value, &self.state.builder, &block);

        match target {
            AssignmentTarget::Stack(pointer, _) => {
                self.state.builder.emit_sol_store(stored, pointer, &block);
            }
            AssignmentTarget::Storage(slot, element_type) => {
                self.emit_storage_store(&slot, stored, element_type, &block);
            }
        }
        Ok((stored, block))
    }

    /// Resolves an assignment's left-hand side to its storage location.
    ///
    /// Only identifier targets — locals, parameters, value-typed state
    /// variables — are supported; index, member, and reference-typed targets
    /// are lowered by later domains.
    fn resolve_assignment_target(&self, left: &Expression) -> AssignmentTarget<'context, 'block> {
        let Expression::Identifier(identifier) = left else {
            unimplemented!("assignment target: {:?}", std::mem::discriminant(left));
        };
        match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) =
                    self.environment.variable_with_type(&identifier.name());
                AssignmentTarget::Stack(pointer, element_type)
            }
            Some(Definition::StateVariable(state_variable)) => {
                let declared_type = state_variable
                    .get_type()
                    .expect("binder types every state variable");
                if declared_type.is_reference_type() {
                    unimplemented!("assignment to a reference-typed state variable");
                }
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("every value-typed state variable has a storage slot")
                    .clone();
                let element_type =
                    TypeConversion::resolve_slang_type(&declared_type, None, &self.state.builder);
                AssignmentTarget::Storage(slot, element_type)
            }
            Some(_) => unimplemented!("assignment to binding kind: {}", identifier.name()),
            None => unreachable!("slang resolves every identifier reference"),
        }
    }
}
