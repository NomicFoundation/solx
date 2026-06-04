//!
//! Built-in function call lowering.
//!

/// Dynamic-array and `bytes` member built-ins (`push`/`pop`).
pub mod array;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Tries to lower `callee(arguments)` as a Solidity built-in.
    ///
    /// Returns `Ok(Some((value, block)))` on a recognized built-in — the value
    /// is `None` for statement-style built-ins (`assert`, `require`) — or
    /// `Ok(None)` when the callee is not a built-in, so the caller falls
    /// through. The binder fixes each built-in's arity, so the handlers take
    /// the arguments as given. Member-access built-ins (`msg.sender`,
    /// `abi.encode`, …) and the remaining globals defer to later domains.
    pub fn try_emit_built_in_call(
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
            BuiltIn::Assert => self.emit_assert(arguments, block).map(Some),
            BuiltIn::Require => self.emit_require(arguments, block).map(Some),
            BuiltIn::Gasleft => self.emit_gasleft(block).map(Some),
            BuiltIn::Keccak256 => self.emit_keccak256(arguments, block).map(Some),
            BuiltIn::Sha256 => self.emit_sha256(arguments, block).map(Some),
            BuiltIn::Ripemd160 => self.emit_ripemd160(arguments, block).map(Some),
            BuiltIn::Ecrecover => self.emit_ecrecover(arguments, block).map(Some),
            BuiltIn::Addmod => self.emit_addmod(arguments, block).map(Some),
            BuiltIn::Mulmod => self.emit_mulmod(arguments, block).map(Some),
            _ => Ok(None),
        }
    }

    /// Tries to lower a member-access call `base.method(arguments)` whose method
    /// is a Solidity built-in handled here (`arr.push(x)`, `arr.push()`,
    /// `arr.pop()`).
    ///
    /// Returns `Ok(None)` when the callee is not a member access or its member
    /// is not such a built-in, so the caller falls through to the remaining
    /// call kinds.
    pub fn try_emit_member_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        let Expression::MemberAccessExpression(access) = callee else {
            return Ok(None);
        };
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::ArrayPop) => self.emit_array_pop(access, block).map(Some),
            Some(BuiltIn::ArrayPush) => self.emit_array_push(access, arguments, block).map(Some),
            _ => Ok(None),
        }
    }

    /// Lowers `assert(condition)` to `sol.assert`.
    fn emit_assert(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let condition = arguments.iter().next().expect("assert takes one argument");
        let (value, block) = self.expression_emitter.emit_value(&condition, block)?;
        let condition = self.expression_emitter.emit_is_nonzero(value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition, &block);
        Ok((None, block))
    }

    /// Lowers `require(condition[, message])` to `sol.require`.
    ///
    /// A string-literal message is carried verbatim; a runtime string is
    /// wrapped as `Error(string)`. A custom-error message (`require(c, E(a))`)
    /// defers to the reverts domain.
    fn emit_require(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut arguments = arguments.iter();
        let condition = arguments
            .next()
            .expect("require takes a condition argument");
        let message = arguments.next();

        let (value, block) = self.expression_emitter.emit_value(&condition, block)?;
        let condition = self.expression_emitter.emit_is_nonzero(value, &block);
        let builder = &self.expression_emitter.state.builder;
        match message {
            None => {
                builder.emit_sol_require(condition, None, &[], false, &block);
                Ok((None, block))
            }
            Some(Expression::StringExpression(string)) => {
                let literal = String::from_utf8(string.value())
                    .map_err(|_| anyhow::anyhow!("require message is not valid UTF-8"))?;
                builder.emit_sol_require(condition, Some(&literal), &[], false, &block);
                Ok((None, block))
            }
            Some(expression) => {
                let (message, block) = self.expression_emitter.emit_value(&expression, block)?;
                self.expression_emitter.state.builder.emit_sol_require(
                    condition,
                    Some("Error(string)"),
                    &[message],
                    true,
                    &block,
                );
                Ok((None, block))
            }
        }
    }

    /// Lowers `gasleft()` to `sol.gasleft`.
    fn emit_gasleft(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_gas_left(&block);
        Ok((Some(value), block))
    }

    /// Lowers `keccak256(bytes memory)` to `sol.keccak256`.
    fn emit_keccak256(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_keccak256(values[0], &block);
        Ok((Some(value), block))
    }

    /// Lowers `sha256(bytes memory)` to the `sol.sha256` precompile.
    fn emit_sha256(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_sha256(values[0], &block);
        Ok((Some(value), block))
    }

    /// Lowers `ripemd160(bytes memory)` to the `sol.ripemd160` precompile.
    fn emit_ripemd160(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_ripemd160(values[0], &block);
        Ok((Some(value), block))
    }

    /// Lowers `ecrecover(hash, v, r, s)` to the `sol.ecrecover` precompile.
    fn emit_ecrecover(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_ecrecover(values[0], values[1], values[2], values[3], &block);
        Ok((Some(value), block))
    }

    /// Lowers `addmod(x, y, m)` to `sol.addmod`.
    fn emit_addmod(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let x = self.widen_to_word(values[0], &block);
        let y = self.widen_to_word(values[1], &block);
        let modulus = self.widen_to_word(values[2], &block);
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_addmod(x, y, modulus, &block);
        Ok((Some(value), block))
    }

    /// Lowers `mulmod(x, y, m)` to `sol.mulmod`.
    fn emit_mulmod(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (values, block) = self.emit_argument_values(arguments, block)?;
        let x = self.widen_to_word(values[0], &block);
        let y = self.widen_to_word(values[1], &block);
        let modulus = self.widen_to_word(values[2], &block);
        let value = self
            .expression_emitter
            .state
            .builder
            .emit_sol_mulmod(x, y, modulus, &block);
        Ok((Some(value), block))
    }

    /// Evaluates each argument expression in order.
    fn emit_argument_values(
        &self,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut values = Vec::with_capacity(arguments.len());
        let mut block = block;
        for argument in arguments.iter() {
            let (value, next_block) = self.expression_emitter.emit_value(&argument, block)?;
            values.push(value);
            block = next_block;
        }
        Ok((values, block))
    }

    /// Widens an integer value to `ui256`, the type `sol.addmod`/`sol.mulmod`
    /// require for their uniform operands (a narrow literal keeps its type).
    fn widen_to_word(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.expression_emitter.state.builder;
        builder.emit_sol_cast(value, builder.types.ui256, block)
    }
}
