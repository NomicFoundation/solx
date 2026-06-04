//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a simple assignment (`=`) to a local variable or parameter.
    ///
    /// The right-hand side is evaluated, cast to the target's declared type,
    /// and stored to the target's stack slot; the stored value is the
    /// expression's result. Compound assignments and non-local targets (state
    /// variables, index elements, struct fields) are lowered by later domains.
    pub fn emit_assignment(
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

        let left = assignment.left_operand();
        let Expression::Identifier(identifier) = &left else {
            unimplemented!("assignment target: {:?}", std::mem::discriminant(&left));
        };
        let (pointer, element_type) = match identifier.resolve_to_definition() {
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                self.environment.variable_with_type(&identifier.name())
            }
            Some(_) => unimplemented!("assignment to binding kind: {}", identifier.name()),
            None => unreachable!("slang resolves every identifier reference"),
        };

        let (value, block) = self.emit_value(&assignment.right_operand(), block)?;
        let stored = TypeConversion::from_target_type(element_type, &self.state.builder).emit(
            value,
            &self.state.builder,
            &block,
        );
        self.state.builder.emit_sol_store(stored, pointer, &block);
        Ok((stored, block))
    }
}
