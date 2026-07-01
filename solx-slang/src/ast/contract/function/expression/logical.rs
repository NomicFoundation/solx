//!
//! Comparison and short-circuit logical expression lowering.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::YieldOperation;

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
            AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
        };
        let lhs = TypeConversion::from_target_type(common_type, self.state).emit(
            lhs,
            self.state,
            &block,
        );
        let rhs = TypeConversion::from_target_type(common_type, self.state).emit(
            rhs,
            self.state,
            &block,
        );
        let comparison =
            AstValue::new(lhs).compare(AstValue::new(rhs), predicate, self.state, &block).into_mlir();
        Ok((comparison, block))
    }

    /// Emits short-circuit `&&` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `false`, and only evaluate the RHS when the LHS is true.
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

        let i1_type =
            AstType::signless(self.state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let result_ptr = Pointer::stack(AstType::new(i1_type), self.state, &block).into_mlir();
        let false_value = AstValue::boolean(false, self.state, &block).into_mlir();
        Pointer::new(result_ptr).store(AstValue::new(false_value), self.state, &block);

        let (then_block, else_block) =
            mlir_region_op!(self.state, &block, IfOperation.cond(lhs_bool); then_region, else_region);

        let (rhs, then_end) = self.emit_value(right, then_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        Pointer::new(result_ptr).store(AstValue::new(rhs_bool), self.state, &then_end);
        mlir_op_void!(self.state, &then_end, YieldOperation.ins(&[]));

        mlir_op_void!(self.state, &else_block, YieldOperation.ins(&[]));

        let result =
            Pointer::new(result_ptr).load(AstType::new(i1_type), self.state, &block).into_mlir();
        Ok((result, block))
    }

    /// Emits short-circuit `||` using `sol.if` with an `i1` alloca.
    ///
    /// Matches solc's pattern: allocate a boolean result variable, default to
    /// `true`, and only evaluate the RHS when the LHS is false.
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

        let i1_type =
            AstType::signless(self.state.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let result_ptr = Pointer::stack(AstType::new(i1_type), self.state, &block).into_mlir();
        let true_value = AstValue::boolean(true, self.state, &block).into_mlir();
        Pointer::new(result_ptr).store(AstValue::new(true_value), self.state, &block);

        let (then_block, else_block) =
            mlir_region_op!(self.state, &block, IfOperation.cond(lhs_bool); then_region, else_region);

        mlir_op_void!(self.state, &then_block, YieldOperation.ins(&[]));

        let (rhs, else_end) = self.emit_value(right, else_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        Pointer::new(result_ptr).store(AstValue::new(rhs_bool), self.state, &else_end);
        mlir_op_void!(self.state, &else_end, YieldOperation.ins(&[]));

        let result =
            Pointer::new(result_ptr).load(AstType::new(i1_type), self.state, &block).into_mlir();
        Ok((result, block))
    }
}
