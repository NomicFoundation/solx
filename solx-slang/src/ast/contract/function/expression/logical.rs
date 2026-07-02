//!
//! Comparison and short-circuit logical expression lowering.
//!

use melior::ir::BlockRef;
use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;
use solx_mlir::Effect;
use solx_mlir::Place;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let (rhs, block) = self.emit_value(right, block)?;
        let common_type = if lhs.r#type() == rhs.r#type() {
            lhs.r#type()
        } else {
            Type::unsigned(self.state.melior, solx_utils::BIT_LENGTH_FIELD)
        };
        let lhs =
            TypeConversion::from_target_type(common_type, self.state).emit(lhs, self.state, &block);
        let rhs =
            TypeConversion::from_target_type(common_type, self.state).emit(rhs, self.state, &block);
        let comparison = lhs.compare(rhs, predicate, self.state, &block);
        Ok((comparison, block))
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type = Type::signless(self.state.melior, solx_utils::BIT_LENGTH_BOOLEAN);
        let result_ptr = Place::stack(i1_type, self.state, &block);
        let false_value = Value::boolean(false, self.state, &block);
        result_ptr.store(false_value, self.state, &block);

        let (then_block, else_block) = Effect::new(self.state, block).branch(lhs_bool);

        let (rhs, then_end) = self.emit_value(right, then_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        result_ptr.store(rhs_bool, self.state, &then_end);
        Effect::new(self.state, then_end).r#yield(&[]);

        Effect::new(self.state, else_block).r#yield(&[]);

        let result = result_ptr.load(i1_type, self.state, &block);
        Ok((result, block))
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
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit_value(left, block)?;
        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let i1_type = Type::signless(self.state.melior, solx_utils::BIT_LENGTH_BOOLEAN);
        let result_ptr = Place::stack(i1_type, self.state, &block);
        let true_value = Value::boolean(true, self.state, &block);
        result_ptr.store(true_value, self.state, &block);

        let (then_block, else_block) = Effect::new(self.state, block).branch(lhs_bool);

        Effect::new(self.state, then_block).r#yield(&[]);

        let (rhs, else_end) = self.emit_value(right, else_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        result_ptr.store(rhs_bool, self.state, &else_end);
        Effect::new(self.state, else_end).r#yield(&[]);

        let result = result_ptr.load(i1_type, self.state, &block);
        Ok((result, block))
    }
}
