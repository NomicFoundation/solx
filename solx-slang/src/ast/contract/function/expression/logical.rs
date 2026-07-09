//!
//! Comparison and short-circuit logical expression lowering.
//!

use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context> ExpressionEmitter<'state, 'context> {
    /// Emits a `sol.cmp` comparison.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_comparison(
        &self,
        left: &Expression,
        right: &Expression,
        predicate: CmpPredicate,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let lhs = self.emit_value(left, context)?;
        let rhs = self.emit_value(right, context)?;
        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            Type::unsigned(context.melior, solx_utils::BIT_LENGTH_FIELD)
        };
        let lhs = TypeConversion::from_target_type(common_type, context).emit(lhs, context);
        let rhs = TypeConversion::from_target_type(common_type, context).emit(rhs, context);
        let comparison = lhs.compare(rhs, predicate, context);
        Ok(comparison)
    }

    /// Emits short-circuit `&&` using `sol.if` with an `i1` alloca.
    ///
    /// Allocates a boolean result variable, defaults it to `false`, and only
    /// evaluates the RHS when the LHS is true.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let lhs = self.emit_value(left, context)?;
        let lhs_bool = self.emit_is_nonzero(lhs, context);

        let i1_type = Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN);
        let result_ptr = Place::stack(i1_type, context);
        let false_value = Value::boolean(false, context);
        result_ptr.store(false_value, context);

        let parent = context.current_block();
        let (then_block, else_block) = parent.branch(lhs_bool, true, context);
        let else_block = else_block.expect("`&&` emits an else arm");

        context.current_block = Some(then_block);
        let rhs = self.emit_value(right, context)?;
        let rhs_bool = self.emit_is_nonzero(rhs, context);
        result_ptr.store(rhs_bool, context);
        let then_end = context.current_block();
        then_end.r#yield(&[], context);

        else_block.r#yield(&[], context);

        context.current_block = Some(parent);
        let result = result_ptr.load(i1_type, context);
        Ok(result)
    }

    /// Emits short-circuit `||` using `sol.if` with an `i1` alloca.
    ///
    /// Allocates a boolean result variable, defaults it to `true`, and only
    /// evaluates the RHS when the LHS is false.
    ///
    /// # Errors
    ///
    /// Returns an error if either operand contains unsupported constructs.
    pub fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        context: &mut Context<'context>,
    ) -> anyhow::Result<Value<'context>> {
        let lhs = self.emit_value(left, context)?;
        let lhs_bool = self.emit_is_nonzero(lhs, context);

        let i1_type = Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN);
        let result_ptr = Place::stack(i1_type, context);
        let true_value = Value::boolean(true, context);
        result_ptr.store(true_value, context);

        let parent = context.current_block();
        let (then_block, else_block) = parent.branch(lhs_bool, true, context);
        let else_block = else_block.expect("`||` emits an else arm");

        then_block.r#yield(&[], context);

        context.current_block = Some(else_block);
        let rhs = self.emit_value(right, context)?;
        let rhs_bool = self.emit_is_nonzero(rhs, context);
        result_ptr.store(rhs_bool, context);
        let else_end = context.current_block();
        else_end.r#yield(&[], context);

        context.current_block = Some(parent);
        let result = result_ptr.load(i1_type, context);
        Ok(result)
    }
}
