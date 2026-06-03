//!
//! Literal expression lowering to `sol.constant` values.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use num_bigint::BigInt;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::Type as SlangType;

use super::ExpressionEmitter;
use super::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a decimal number literal to a `sol.constant`.
    ///
    /// The value is taken after applying any unit suffix (`wei`, `ether`,
    /// `seconds`, …); the type is the smallest byte-aligned integer type the
    /// binder assigned to the literal node.
    pub(super) fn emit_decimal(
        &self,
        decimal: &DecimalNumberExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let value = decimal
            .integer_value()
            .expect("decimal literal evaluates to an integer after applying units");
        let slang_type = decimal
            .get_type()
            .expect("binder types every decimal literal node");
        self.emit_integer_constant(&value, &slang_type, block)
    }

    /// Lowers a hexadecimal number literal to a `sol.constant`.
    pub(super) fn emit_hex(
        &self,
        hex: &HexNumberExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let value = hex
            .integer_value()
            .expect("hex literal always evaluates to an integer");
        let slang_type = hex.get_type().expect("binder types every hex literal node");
        self.emit_integer_constant(&value, &slang_type, block)
    }

    /// Lowers a boolean keyword (`true` / `false`) to an `i1` `sol.constant`.
    pub(super) fn emit_boolean(
        &self,
        value: bool,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.builder.emit_bool(value, block)
    }

    /// Emits a `sol.constant` of the integer type the binder assigned to a
    /// numeric literal node.
    fn emit_integer_constant(
        &self,
        value: &BigInt,
        slang_type: &SlangType,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let result_type = TypeConversion::resolve_slang_type(slang_type, None, &self.state.builder);
        self.state.builder.emit_constant(value, result_type, block)
    }
}
