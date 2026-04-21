//!
//! Function call and member access expression lowering.
//!

pub mod type_conversion;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity::backend::built_ins::BuiltIn;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionCallExpression;
use solx_mlir::ods::sol::BaseFeeOperation;
use solx_mlir::ods::sol::BlockNumberOperation;
use solx_mlir::ods::sol::CallValueOperation;
use solx_mlir::ods::sol::CallerOperation;
use solx_mlir::ods::sol::ChainIdOperation;
use solx_mlir::ods::sol::CoinbaseOperation;
use solx_mlir::ods::sol::GasLimitOperation;
use solx_mlir::ods::sol::GasPriceOperation;
use solx_mlir::ods::sol::OriginOperation;
use solx_mlir::ods::sol::TimestampOperation;

use crate::ast::contract::function::expression::ExpressionEmitter;

use self::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Solidity built-in `assert()` function name.
    const ASSERT: &'static str = "assert";

    /// Solidity built-in `require()` function name.
    const REQUIRE: &'static str = "require";

    /// Maximum number of arguments for `require()` (condition + optional message).
    const MAX_REQUIRE_ARGUMENTS: usize = 2;

    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Resolves the callee by name and argument count, handling type
    /// conversions, built-in functions, and user-defined calls.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is unsupported, arguments contain
    /// unsupported constructs, or the function is undefined.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            anyhow::bail!("only positional arguments supported");
        };

        // Resolve callee name for Identifier/PayableKeyword callees; None for
        // ElementaryType and MemberAccessExpression (handled as identity casts).
        let callee = call.operand();
        let callee_name: Option<String> = match &callee {
            Expression::Identifier(identifier) => Some(identifier.name()),
            Expression::PayableKeyword => Some("payable".to_owned()),
            _ => None,
        };

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("len checked to be 1 above");
            let (value, block) = self.expression_emitter.emit_value(&first, block)?;
            let builder = &self.expression_emitter.state.builder;

            let target_type = self
                .expression_emitter
                .resolve_expression_type(call.node_id())
                .ok_or_else(|| anyhow::anyhow!("unresolved type conversion target"))?;

            let result =
                TypeConversion::from_target_type(target_type, builder).emit(value, builder, &block);
            return Ok((Some(result), block));
        }

        // Non-conversion path: require named callee.
        let callee_name =
            callee_name.ok_or_else(|| anyhow::anyhow!("unsupported callee expression"))?;

        // Handle require() built-in.
        if callee_name == Self::REQUIRE
            && (positional_arguments.len() == 1
                || positional_arguments.len() == Self::MAX_REQUIRE_ARGUMENTS)
        {
            // TODO: encode revert reason from second argument
            let first = positional_arguments
                .iter()
                .next()
                .expect("length checked above");
            let block = self.emit_require(&first, block)?;
            return Ok((None, block));
        }

        // Handle assert() built-in.
        if callee_name == Self::ASSERT && positional_arguments.len() == 1 {
            let first = positional_arguments
                .iter()
                .next()
                .expect("length checked above");
            let block = self.emit_assert(&first, block)?;
            return Ok((None, block));
        }

        let mut argument_values = Vec::new();
        let mut current_block = block;

        for argument in positional_arguments.iter() {
            let (value, next_block) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let argument_types: Vec<melior::ir::Type<'context>> =
            argument_values.iter().map(|value| value.r#type()).collect();
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(&callee_name, &argument_types)?;

        // Cast arguments to match the callee's declared parameter types.
        let builder = &self.expression_emitter.state.builder;
        for (value, &param_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(param_type, builder);
            *value = conversion.emit(*value, builder, &current_block);
        }

        if return_types.is_empty() {
            self.expression_emitter.state.builder.emit_sol_call(
                mlir_name,
                &argument_values,
                &[],
                &current_block,
            )?;
            Ok((None, current_block))
        } else {
            let result = self
                .expression_emitter
                .state
                .builder
                .emit_sol_call(mlir_name, &argument_values, return_types, &current_block)?
                .expect("function call always produces at least one result");
            Ok((Some(result), current_block))
        }
    }

    /// Emits a member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    /// Resolves the member via slang's binder to a specific `BuiltIn` variant
    /// rather than string-matching the member name.
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        member_identifier: &slang_solidity::backend::ir::ast::Identifier,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let context = builder.context;
        let location = builder.unknown_location;
        let address_type = builder.types.sol_address;
        let ui256_type = builder.types.ui256;
        let operation = match member_identifier.resolved_built_in() {
            Some(BuiltIn::TxOrigin) => OriginOperation::builder(context, location)
                .addr(address_type)
                .build()
                .into(),
            Some(BuiltIn::TxGasPrice) => GasPriceOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::MsgSender) => CallerOperation::builder(context, location)
                .addr(address_type)
                .build()
                .into(),
            Some(BuiltIn::MsgValue) => CallValueOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::BlockTimestamp) => TimestampOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::BlockNumber) => BlockNumberOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::BlockCoinbase) => CoinbaseOperation::builder(context, location)
                .addr(address_type)
                .build()
                .into(),
            Some(BuiltIn::BlockChainid) => ChainIdOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::BlockBasefee) => BaseFeeOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            Some(BuiltIn::BlockGaslimit) => GasLimitOperation::builder(context, location)
                .val(ui256_type)
                .build()
                .into(),
            _ => anyhow::bail!("unsupported member access: {}", member_identifier.name()),
        };
        let value = block
            .append_operation(operation)
            .result(0)
            .expect("intrinsic always produces one result")
            .into();
        Ok((value, block))
    }

    /// Emits an `assert(condition)` built-in via `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);
        self.expression_emitter
            .state
            .builder
            .emit_sol_assert(condition_boolean, &block);
        Ok(block)
    }

    /// Emits a `require(condition)` built-in via `sol.require`.
    fn emit_require(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let (condition_value, block) = self.expression_emitter.emit_value(condition, block)?;
        let condition_boolean = self
            .expression_emitter
            .emit_is_nonzero(condition_value, &block);

        self.expression_emitter
            .state
            .builder
            .emit_sol_require(condition_boolean, &block);

        Ok(block)
    }
}
