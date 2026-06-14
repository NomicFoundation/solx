//!
//! Comparison expression lowering: equality and inequality (`sol.cmp`). Each
//! node bridges to its [`CmpPredicate`] via [`ComparisonPredicateExt`]; the
//! shared `sol.cmp` lowering stays on the emitter because `CmpPredicate` is a
//! foreign type (cf. the owned [`Operator`] for arithmetic).
//!
//! [`Operator`]: crate::ast::contract::function::expression::operator::Operator
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::EqualityExpression;
use slang_solidity_v2::ast::EqualityExpressionOperator;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::InequalityExpression;
use slang_solidity_v2::ast::InequalityExpressionOperator;
use solx_mlir::CmpPredicate;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::contract::function::expression::ExpressionContext;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
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
        // A string literal compared with a `bytesN` / `byte` sibling (`b == "d"`)
        // materialises toward the sibling's fixed-bytes type rather than emitting a
        // runtime `sol.string`; the non-string operand is emitted first to learn
        // that type. With neither (or both) a string literal, both emit naturally.
        let left_is_string = matches!(left, Expression::StringExpression(_));
        let right_is_string = matches!(right, Expression::StringExpression(_));
        let is_bytes_like = |r#type: crate::ast::Type| r#type.fixed_bytes_or_byte_width().is_some();
        let (lhs, rhs, block) = if right_is_string && !left_is_string {
            let BlockAnd { value: lhs, block } = left.emit(self, block)?;
            let BlockAnd { value: rhs, block } = if is_bytes_like(lhs.r#type()) {
                (Toward {
                    expression: right,
                    target_type: lhs.r#type().into_mlir(),
                })
                .emit(self, block)?
            } else {
                right.emit(self, block)?
            };
            (lhs, rhs, block)
        } else if left_is_string && !right_is_string {
            let BlockAnd { value: rhs, block } = right.emit(self, block)?;
            let BlockAnd { value: lhs, block } = if is_bytes_like(rhs.r#type()) {
                (Toward {
                    expression: left,
                    target_type: rhs.r#type().into_mlir(),
                })
                .emit(self, block)?
            } else {
                left.emit(self, block)?
            };
            (lhs, rhs, block)
        } else {
            let BlockAnd { value: lhs, block } = left.emit(self, block)?;
            let BlockAnd { value: rhs, block } = right.emit(self, block)?;
            (lhs, rhs, block)
        };
        if lhs.r#type() == rhs.r#type() {
            let comparison = lhs
                .compare(rhs, predicate, &self.state.builder, &block)
                .into_mlir();
            return Ok((comparison, block));
        }
        // Two fixed-bytes operands of different widths (`bytes3("abc") ==
        // bytes4("abc")`): `bytesN` are LEFT-aligned, so the operands share a
        // word once the narrower is zero-extended on the right. Widen the
        // smaller to the larger fixed-bytes width with a `sol.bytes_cast` and
        // compare AS fixed-bytes, matching solc. Bridging each through its own
        // width integer (the mixed-integer path below) right-aligns the values
        // — `bytes3("abc")` as `ui24` (0x616263) differs from `bytes4("abc")` as
        // `ui32` (0x61626300) — yielding the wrong result.
        if let (Some(lhs_width), Some(rhs_width)) = (
            lhs.r#type().fixed_bytes_or_byte_width(),
            rhs.r#type().fixed_bytes_or_byte_width(),
        ) {
            let builder = &self.state.builder;
            let common_width = lhs_width.max(rhs_width);
            let common = crate::ast::Type::fixed_bytes(builder.context, common_width).into_mlir();
            let lhs_common = if lhs_width == common_width {
                lhs
            } else {
                lhs.cast(crate::ast::Type::new(common), builder, &block)
            };
            let rhs_common = if rhs_width == common_width {
                rhs
            } else {
                rhs.cast(crate::ast::Type::new(common), builder, &block)
            };
            let comparison = lhs_common
                .compare(rhs_common, predicate, builder, &block)
                .into_mlir();
            return Ok((comparison, block));
        }
        // Mixed-type comparison (`i < 10` with `i : int8`, `10 : uint8`): widen
        // each operand to 256 bits preserving ITS OWN signedness — a signed
        // operand sign-extends, an unsigned one zero-extends — so a signed
        // negative value is not reinterpreted as a huge unsigned one. Then pick
        // the common type: signed if either operand is signed, mirroring solc's
        // promoted comparison type; a plain `ui256` default would make
        // `(-10) < 10` an unsigned comparison (false), skipping the loop.
        let signed_lhs = IntegerType::try_from(lhs.r#type().into_mlir())
            .is_ok_and(|integer| integer.is_signed());
        let signed_rhs = IntegerType::try_from(rhs.r#type().into_mlir())
            .is_ok_and(|integer| integer.is_signed());
        let context = self.state.builder.context;
        let signed_256 = Type::from(IntegerType::signed(context, 256));
        let unsigned_256 =
            crate::ast::Type::unsigned(self.state.builder.context, solx_utils::BIT_LENGTH_FIELD)
                .into_mlir();
        let lhs_wide_type = if signed_lhs { signed_256 } else { unsigned_256 };
        let rhs_wide_type = if signed_rhs { signed_256 } else { unsigned_256 };
        let lhs_wide = lhs.coerce_to(
            crate::ast::Type::new(lhs_wide_type),
            &self.state.builder,
            &block,
        );
        let rhs_wide = rhs.coerce_to(
            crate::ast::Type::new(rhs_wide_type),
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
        let lhs_common = if lhs_wide.r#type().into_mlir() == common {
            lhs_wide
        } else {
            lhs_wide.cast(crate::ast::Type::new(common), &self.state.builder, &block)
        };
        let rhs_common = if rhs_wide.r#type().into_mlir() == common {
            rhs_wide
        } else {
            rhs_wide.cast(crate::ast::Type::new(common), &self.state.builder, &block)
        };
        let comparison = lhs_common
            .compare(rhs_common, predicate, &self.state.builder, &block)
            .into_mlir();
        Ok((comparison, block))
    }
}

/// Bridges a slang comparison node to the [`CmpPredicate`] it applies, so the
/// shared comparison body lowers equality and inequality uniformly. Each node
/// maps its typed slang operator enum — never source text.
trait ComparisonPredicateExt {
    /// The [`CmpPredicate`] this comparison applies.
    fn bridged_predicate(&self) -> CmpPredicate;
}

impl ComparisonPredicateExt for EqualityExpression {
    fn bridged_predicate(&self) -> CmpPredicate {
        match self.operator() {
            EqualityExpressionOperator::BangEqual(_) => CmpPredicate::Ne,
            EqualityExpressionOperator::EqualEqual(_) => CmpPredicate::Eq,
        }
    }
}

impl ComparisonPredicateExt for InequalityExpression {
    fn bridged_predicate(&self) -> CmpPredicate {
        match self.operator() {
            InequalityExpressionOperator::GreaterThan(_) => CmpPredicate::Gt,
            InequalityExpressionOperator::GreaterThanEqual(_) => CmpPredicate::Ge,
            InequalityExpressionOperator::LessThan(_) => CmpPredicate::Lt,
            InequalityExpressionOperator::LessThanEqual(_) => CmpPredicate::Le,
        }
    }
}

expression_emit!(EqualityExpression, InequalityExpression; |node, context, block| {
    let (value, block) = context.emit_comparison(
        &node.left_operand(),
        &node.right_operand(),
        node.bridged_predicate(),
        block,
    )?;
    Ok(BlockAnd { block, value: value.into() })
});
