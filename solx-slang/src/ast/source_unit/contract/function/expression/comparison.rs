//!
//! Comparison and short-circuit logical expression lowering.
//!

use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::RegionLike;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::ICmpPredicate;

use crate::ast::source_unit::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits an `llvm.icmp` comparison, zero-extended to `i256`.
    ///
    /// Uses signed predicates when either operand is a signed integer type.
    pub(super) fn emit_icmp(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let signed = self.is_signed_expression(left) || self.is_signed_expression(right);
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;

        let predicate = match (operator, signed) {
            ("==", _) => ICmpPredicate::Eq,
            ("!=", _) => ICmpPredicate::Ne,
            (">", false) => ICmpPredicate::Ugt,
            (">", true) => ICmpPredicate::Sgt,
            (">=", false) => ICmpPredicate::Uge,
            (">=", true) => ICmpPredicate::Sge,
            ("<", false) => ICmpPredicate::Ult,
            ("<", true) => ICmpPredicate::Slt,
            ("<=", false) => ICmpPredicate::Ule,
            ("<=", true) => ICmpPredicate::Sle,
            _ => anyhow::bail!("unsupported comparison operator: {operator}"),
        };

        let cmp = self.state.emit_icmp(lhs, rhs, predicate, &block);
        let value = self.state.emit_zext_to_i256(cmp, &block);
        Ok((value, block))
    }

    /// Emits short-circuit `&&` using control flow.
    ///
    /// Result is always a canonical boolean (0 or 1).
    pub(super) fn emit_and(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        let zero = self.state.emit_sol_constant(0, &block);
        block.append_operation(self.state.llvm_cond_br(
            lhs_bool,
            &rhs_block,
            &merge_block,
            &[],
            &[zero],
        ));

        let (rhs, rhs_block) = self.emit(right, rhs_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &rhs_block);
        let rhs_normalized = self.state.emit_zext_to_i256(rhs_bool, &rhs_block);
        rhs_block.append_operation(self.state.llvm_br(&merge_block, &[rhs_normalized]));

        let result = merge_block.argument(0)?.into();
        Ok((result, merge_block))
    }

    /// Emits short-circuit `||` using control flow.
    ///
    /// Result is always a canonical boolean (0 or 1).
    pub(super) fn emit_or(
        &self,
        left: &Expression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (lhs, block) = self.emit(left, block)?;
        let i256 = self.state.i256();
        let location = self.state.location();

        let lhs_bool = self.emit_is_nonzero(lhs, &block);
        let one = self.state.emit_sol_constant(1, &block);

        let rhs_block = self.region.append_block(Block::new(&[]));
        let merge_block = self.region.append_block(Block::new(&[(i256, location)]));

        block.append_operation(self.state.llvm_cond_br(
            lhs_bool,
            &merge_block,
            &rhs_block,
            &[one],
            &[],
        ));

        let (rhs, rhs_block) = self.emit(right, rhs_block)?;
        let rhs_bool = self.emit_is_nonzero(rhs, &rhs_block);
        let rhs_normalized = self.state.emit_zext_to_i256(rhs_bool, &rhs_block);
        rhs_block.append_operation(self.state.llvm_br(&merge_block, &[rhs_normalized]));

        let result = merge_block.argument(0)?.into();
        Ok((result, merge_block))
    }
}
