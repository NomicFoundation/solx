//!
//! Solidity built-in function and EVM intrinsic emission.
//!

use crate::ast::Type as AstType;
pub mod abi;
pub mod array;
pub mod global;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::PositionalArguments;
use solx_mlir::ods::sol::ConcatOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::ExpressionContext;

/// ABI encoding mode for `abi.encode` / `abi.encodePacked`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeMode {
    /// Standard ABI encoding with per-element padding (`abi.encode`,
    /// `abi.encodeWithSelector`, `abi.encodeWithSignature`).
    Standard,
    /// Packed ABI encoding with no per-element padding (`abi.encodePacked`).
    Packed,
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Lowers `string.concat(...)` / `bytes.concat(...)` to `sol.concat`, which
    /// takes a variadic list of string / `bytesN` values and yields a freshly
    /// allocated memory string. An empty argument list is valid
    /// (`string.concat()` → `""`).
    pub fn emit_concat(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let BlockAnd {
            value: values,
            block,
        } = arguments.emit(self, block);
        let builder = &self.state.builder;
        let result_type =
            AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
        let value = sol_op!(
            builder,
            block,
            ConcatOperation.args(&values).result(result_type)
        );
        (Some(value), block)
    }
}
