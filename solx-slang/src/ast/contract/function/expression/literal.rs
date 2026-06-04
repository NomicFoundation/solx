//!
//! Literal expression lowering to `sol.constant` values.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;

use crate::ast::contract::function::expression::ExpressionEmitter;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a decimal number literal to a `sol.constant`.
    pub fn emit_decimal(
        &self,
        _decimal: &DecimalNumberExpression,
        _block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        unimplemented!("literal: decimal number")
    }

    /// Lowers a hexadecimal number literal to a `sol.constant`.
    pub fn emit_hex(
        &self,
        _hex: &HexNumberExpression,
        _block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        unimplemented!("literal: hex number")
    }

    /// Lowers a boolean literal to a `sol.constant`.
    pub fn emit_boolean(
        &self,
        _value: bool,
        _block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        unimplemented!("literal: boolean")
    }

    /// Lowers a string literal to a memory `sol.string` value.
    pub fn emit_string(
        &self,
        _string: &StringExpression,
        _block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        unimplemented!("literal: string")
    }
}
