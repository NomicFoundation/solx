//!
//! Literal expression lowering: numbers, booleans, strings, arrays, `this`.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ods::sol::ThisOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers a decimal number literal to a `sol.constant`, using the integer
    /// value the binder computes after applying any unit suffix.
    pub fn emit_decimal(
        &self,
        decimal: &DecimalNumberExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let value = decimal
            .integer_value()
            .expect("a decimal literal evaluates to an integer after applying units");
        let result_type = self
            .resolve_slang_type(decimal.get_type())
            .expect("the binder types every decimal literal node");
        self.state.builder.emit_constant(&value, result_type, block)
    }

    /// Lowers a hexadecimal number literal to a `sol.constant`.
    pub fn emit_hex(
        &self,
        hex: &HexNumberExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let value = hex
            .integer_value()
            .expect("a hex literal always evaluates to an integer");
        let result_type = self
            .resolve_slang_type(hex.get_type())
            .expect("the binder types every hex literal node");
        self.state.builder.emit_constant(&value, result_type, block)
    }

    /// Lowers a boolean keyword (`true` / `false`) to an `i1` `sol.constant`.
    pub fn emit_boolean(
        &self,
        value: bool,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        self.state.builder.emit_bool(value, block)
    }

    /// Lowers a string literal to a freshly allocated memory string.
    pub fn emit_string(
        &self,
        string: &StringExpression,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let bytes = string.value();
        let text = std::str::from_utf8(&bytes).expect("a string literal is valid UTF-8");
        self.state.builder.emit_sol_string_lit(text, block)
    }

    /// Lowers `this` to the current contract address via `sol.this`.
    pub fn emit_this(&self, block: &BlockRef<'context, 'block>) -> Value<'context, 'block> {
        let contract_type = self
            .state
            .current_contract_type
            .expect("`this` is only valid inside a contract");
        let operation = ThisOperation::builder(
            self.state.builder.context,
            self.state.builder.unknown_location,
        )
        .addr(contract_type)
        .build();
        block
            .append_operation(operation.into())
            .result(0)
            .expect("sol.this always produces one result")
            .into()
    }

    /// Lowers an array literal `[a, b, c]` to `sol.array_lit`, casting each
    /// element to the array's element type.
    pub fn emit_array(
        &self,
        array: &ArrayExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let result_slang_type = array
            .get_type()
            .expect("the binder types every array literal");
        let element_slang_type = match &result_slang_type {
            SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
            SlangType::Array(array_type) => array_type.element_type(),
            _ => unreachable!("an array literal is always typed as an array"),
        };
        let builder = &self.state.builder;
        let array_type = TypeConversion::resolve_slang_type(&result_slang_type, None, builder);
        let element_type = TypeConversion::resolve_slang_type(&element_slang_type, None, builder);
        let mut element_values = Vec::new();
        let mut current = block;
        for item in array.items().iter() {
            let (value, next) = self.emit_value(&item, current)?;
            let cast_value =
                TypeConversion::from_target_type(element_type, builder).emit(value, builder, &next);
            element_values.push(cast_value);
            current = next;
        }
        let value = builder.emit_sol_array_lit(&element_values, array_type, &current);
        Ok((value, current))
    }
}
