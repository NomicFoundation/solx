//!
//! Comparison and short-circuit logical expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;

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
        // A string literal compared with a `bytesN` / `byte` operand (`b == "d"`)
        // materializes as a fixedbytes/byte constant rather than a memory string.
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, block)?;
        if lhs.r#type() == rhs.r#type() {
            let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
            return Ok((comparison, block));
        }
        // Mixed-signedness comparison: widen each operand to 256 bits while
        // preserving its own signedness (avoid sign-extending an unsigned
        // value, which `sol.cast ui8 → si256` would do). After widening,
        // emit `sol.cmp` at a common signed/unsigned type.
        let signed_lhs = melior::ir::r#type::IntegerType::try_from(lhs.r#type())
            .map(|t| t.is_signed())
            .unwrap_or(false);
        let signed_rhs = melior::ir::r#type::IntegerType::try_from(rhs.r#type())
            .map(|t| t.is_signed())
            .unwrap_or(false);
        let context = self.state.builder.context;
        let signed_256 =
            melior::ir::Type::from(melior::ir::r#type::IntegerType::signed(context, 256));
        let unsigned_256 = self.state.builder.types.ui256;
        // First widen each operand to its own signedness at 256 bits.
        let lhs_wide_type = if signed_lhs { signed_256 } else { unsigned_256 };
        let rhs_wide_type = if signed_rhs { signed_256 } else { unsigned_256 };
        let lhs_wide = TypeConversion::from_target_type(lhs_wide_type, &self.state.builder).emit(
            lhs,
            &self.state.builder,
            &block,
        );
        let rhs_wide = TypeConversion::from_target_type(rhs_wide_type, &self.state.builder).emit(
            rhs,
            &self.state.builder,
            &block,
        );
        // Both are now 256 bits. If either is signed, retype the other to
        // signed (a bit-preserving cast); else both are unsigned.
        let common = if signed_lhs || signed_rhs { signed_256 } else { unsigned_256 };
        let lhs_common = if lhs_wide.r#type() == common {
            lhs_wide
        } else {
            self.state
                .builder
                .emit_sol_cast(lhs_wide, common, &block)
        };
        let rhs_common = if rhs_wide.r#type() == common {
            rhs_wide
        } else {
            self.state
                .builder
                .emit_sol_cast(rhs_wide, common, &block)
        };
        let comparison =
            self.state
                .builder
                .emit_sol_cmp(lhs_common, rhs_common, predicate, &block);
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

        let i1_type = self.state.builder.types.i1;
        let result_ptr = self.state.builder.emit_sol_alloca(i1_type, &block);
        let false_value = self.state.builder.emit_bool(false, &block);
        self.state
            .builder
            .emit_sol_store(false_value, result_ptr, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(lhs_bool, &block);

        // Then: LHS was true — evaluate RHS and store result.
        let (rhs, then_end) = self.emit_value(right, then_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &then_end);
        self.state
            .builder
            .emit_sol_store(rhs_bool, result_ptr, &then_end);
        self.state.builder.emit_sol_yield(&then_end);

        // Else: LHS was false — result stays false.
        self.state.builder.emit_sol_yield(&else_block);

        let result = self
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
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

        let i1_type = self.state.builder.types.i1;
        let result_ptr = self.state.builder.emit_sol_alloca(i1_type, &block);
        let true_value = self.state.builder.emit_bool(true, &block);
        self.state
            .builder
            .emit_sol_store(true_value, result_ptr, &block);

        let (then_block, else_block) = self.state.builder.emit_sol_if(lhs_bool, &block);

        // Then: LHS was true — result stays true.
        self.state.builder.emit_sol_yield(&then_block);

        // Else: LHS was false — evaluate RHS and store result.
        let (rhs, else_end) = self.emit_value(right, else_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &else_end);
        self.state
            .builder
            .emit_sol_store(rhs_bool, result_ptr, &else_end);
        self.state.builder.emit_sol_yield(&else_end);

        let result = self
            .state
            .builder
            .emit_sol_load(result_ptr, i1_type, &block)?;
        Ok((result, block))
    }
}
