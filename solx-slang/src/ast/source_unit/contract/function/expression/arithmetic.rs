//!
//! Arithmetic expression lowering: binary ops, prefix, postfix.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::ir::ast::Expression;

use solx_mlir::ICmpPredicate;

use crate::ast::source_unit::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Emits a binary arithmetic LLVM operation.
    ///
    /// Uses signed LLVM operations (`sdiv`, `srem`, `ashr`) when either
    /// operand has a signed integer type.
    pub fn emit_binary_op(
        &self,
        left: &Expression,
        right: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let signed = Self::is_signed(left) || Self::is_signed(right);
        let (lhs, block) = self.emit(left, block)?;
        let (rhs, block) = self.emit(right, block)?;
        // TODO: use sol.cadd/csub/cmul for checked arithmetic (Solidity 0.8+ default)
        // TODO: change to a nice enum with FromStr
        let operation_name = match operator {
            "+" => solx_mlir::Builder::ADD,
            "-" => solx_mlir::Builder::SUB,
            "*" => solx_mlir::Builder::MUL,
            "/" if signed => solx_mlir::Builder::SDIV,
            "/" => solx_mlir::Builder::UDIV,
            "%" if signed => solx_mlir::Builder::SREM,
            "%" => solx_mlir::Builder::UREM,
            "&" => solx_mlir::Builder::AND,
            "|" => solx_mlir::Builder::OR,
            "^" => solx_mlir::Builder::XOR,
            "<<" => solx_mlir::Builder::SHL,
            ">>" if signed => solx_mlir::Builder::ASHR,
            ">>" => solx_mlir::Builder::LSHR,
            _ => anyhow::bail!("unsupported binary operator: {operator}"),
        };
        let value = self.emit_llvm_operation(operation_name, lhs, rhs, &block)?;
        Ok((value, block))
    }

    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: &str,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // TODO: support postfix ++/-- on state variables (storage)
        let Expression::Identifier(identifier) = operand else {
            anyhow::bail!("unsupported postfix operand");
        };
        let name = identifier.name();
        let pointer = self
            .environment
            .variable(&name)
            .ok_or_else(|| anyhow::anyhow!("undefined variable: {name}"))?;
        let old = self.emit_load(pointer, &block)?;
        let one = self.state.builder().emit_sol_constant(1, &block);
        // TODO: change to a nice enum with FromStr
        let operation_name = match operator {
            "++" => solx_mlir::Builder::ADD,
            "--" => solx_mlir::Builder::SUB,
            _ => anyhow::bail!("unsupported postfix operator: {operator}"),
        };
        let new = self.emit_llvm_operation(operation_name, old, one, &block)?;
        self.emit_store(new, pointer, &block);
        Ok((old, block))
    }

    /// Emits prefix `!` or `-`.
    pub fn emit_prefix(
        &self,
        operator: &str,
        operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(operand, block)?;
        // TODO: support prefix ++, --, and ~ (bitwise NOT)
        // TODO: change to a nice enum with FromStr
        match operator {
            "!" => {
                let zero = self.state.builder().emit_sol_constant(0, &block);
                let cmp = self
                    .state
                    .builder()
                    .emit_icmp(value, zero, ICmpPredicate::Eq, &block);
                let result = self.state.builder().emit_zext_to_i256(cmp, &block);
                Ok((result, block))
            }
            "-" => {
                let zero = self.state.builder().emit_sol_constant(0, &block);
                let result =
                    self.emit_llvm_operation(solx_mlir::Builder::SUB, zero, value, &block)?;
                Ok((result, block))
            }
            _ => anyhow::bail!("unsupported prefix operator: {operator}"),
        }
    }
}
