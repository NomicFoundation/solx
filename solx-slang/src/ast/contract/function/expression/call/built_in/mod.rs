//!
//! Built-in function call lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;
use solx_utils::DataLocation;

use super::CallEmitter;
use super::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to lower `callee(arguments)` as a Solidity built-in.
    ///
    /// Returns `Ok(Some((value, block)))` on a recognized built-in — `value` is
    /// `None` for statement-style built-ins (`assert`, `require`) — or
    /// `Ok(None)` when the callee is not a built-in, so the caller falls
    /// through. Built-ins beyond `assert`/`require` defer to later domains.
    pub(super) fn try_emit_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::Identifier(identifier) = callee else {
            return Ok(None);
        };
        let Some(built_in) = identifier.resolve_to_built_in() else {
            return Ok(None);
        };
        match built_in {
            BuiltIn::Assert => {
                let condition = arguments.iter().next().expect("assert takes one argument");
                Ok(Some((None, self.emit_assert(&condition, block)?)))
            }
            BuiltIn::Require => {
                let mut arguments = arguments.iter();
                let condition = arguments
                    .next()
                    .expect("require takes a condition argument");
                let message = arguments.next();
                Ok(Some((
                    None,
                    self.emit_require(&condition, message.as_ref(), block)?,
                )))
            }
            _ => Ok(None),
        }
    }

    /// Lowers `assert(condition)` to `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition = self.expression_emitter.emit_is_nonzero(value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition, &block);
        Ok(block)
    }

    /// Lowers `require(condition[, message])` to `sol.require`.
    ///
    /// A string-literal message is carried verbatim; a runtime string is
    /// wrapped as `Error(string)`. A custom-error message (`require(c, E(a))`)
    /// defers to the reverts domain.
    fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition = self.expression_emitter.emit_is_nonzero(value, &block);
        let builder = &self.expression_emitter.state.builder;
        match message {
            None => {
                builder.emit_sol_require(condition, None, &[], false, &block);
                Ok(block)
            }
            Some(Expression::StringExpression(string)) => {
                let literal = String::from_utf8(string.value())
                    .map_err(|_| anyhow::anyhow!("require message is not valid UTF-8"))?;
                builder.emit_sol_require(condition, Some(&literal), &[], false, &block);
                Ok(block)
            }
            Some(expression) => {
                let (message_value, block) =
                    self.expression_emitter.emit_value(expression, block)?;
                let string_type = builder.types.string(DataLocation::Memory);
                let message_value = TypeConversion::from_target_type(string_type, builder).emit(
                    message_value,
                    builder,
                    &block,
                );
                builder.emit_sol_require(
                    condition,
                    Some("Error(string)"),
                    &[message_value],
                    true,
                    &block,
                );
                Ok(block)
            }
        }
    }
}
