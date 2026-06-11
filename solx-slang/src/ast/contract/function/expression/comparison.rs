//!
//! Comparison expression lowering: equality and inequality (`sol.cmp`).
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Expression;
use solx_mlir::CmpPredicate;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::type_conversion::TypeConversion;

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
        let (lhs, rhs, block) = self.emit_binary_operands(left, right, block)?;
        if lhs.r#type() == rhs.r#type() {
            let comparison = self.state.builder.emit_sol_cmp(lhs, rhs, predicate, &block);
            return Ok((comparison, block));
        }
        // Mixed-type comparison (`i < 10` with `i : int8`, `10 : uint8`): widen
        // each operand to 256 bits preserving ITS OWN signedness — a signed
        // operand sign-extends, an unsigned one zero-extends — so a signed
        // negative value is not reinterpreted as a huge unsigned one. Then pick
        // the common type: signed if either operand is signed, mirroring solc's
        // promoted comparison type; a plain `ui256` default would make
        // `(-10) < 10` an unsigned comparison (false), skipping the loop.
        let signed_lhs =
            IntegerType::try_from(lhs.r#type()).is_ok_and(|integer| integer.is_signed());
        let signed_rhs =
            IntegerType::try_from(rhs.r#type()).is_ok_and(|integer| integer.is_signed());
        let context = self.state.builder.context;
        let signed_256 = Type::from(IntegerType::signed(context, 256));
        let unsigned_256 = self.state.builder.types.ui256;
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
        // Both are now 256 bits. Retype each to the common signedness with a
        // bit-preserving `sol.cast` (same width), then compare.
        let common = if signed_lhs || signed_rhs {
            signed_256
        } else {
            unsigned_256
        };
        let lhs_common = if lhs_wide.r#type() == common {
            lhs_wide
        } else {
            self.state.builder.emit_sol_cast(lhs_wide, common, &block)
        };
        let rhs_common = if rhs_wide.r#type() == common {
            rhs_wide
        } else {
            self.state.builder.emit_sol_cast(rhs_wide, common, &block)
        };
        let comparison = self
            .state
            .builder
            .emit_sol_cmp(lhs_common, rhs_common, predicate, &block);
        Ok((comparison, block))
    }
}
