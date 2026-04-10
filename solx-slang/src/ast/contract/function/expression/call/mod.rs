//!
//! Function call and member access expression lowering.
//!

pub mod type_conversion;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity::backend::built_ins::BuiltIn;
use slang_solidity::backend::ir::ast::ArgumentsDeclaration;
use slang_solidity::backend::ir::ast::Expression;
use slang_solidity::backend::ir::ast::FunctionCallExpression;

use self::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::ExpressionEmitter;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
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

        let mut argument_values = Vec::new();
        let mut current_block = block;

        for argument in positional_arguments.iter() {
            let (value, next_block) = self
                .expression_emitter
                .emit_value(&argument, current_block)?;
            argument_values.push(value);
            current_block = next_block;
        }

        let (mlir_name, return_types) = self
            .expression_emitter
            .state
            .resolve_function(&callee_name, argument_values.len())?;

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
        let address_type = builder.get_type(solx_mlir::Builder::SOL_ADDRESS);
        let ui256_type = builder.get_type(solx_mlir::Builder::UI256);
        let (intrinsic, result_type) = match member_identifier.resolved_built_in() {
            Some(BuiltIn::TxOrigin) => (solx_mlir::Builder::SOL_ORIGIN, address_type),
            Some(BuiltIn::TxGasPrice) => (solx_mlir::Builder::SOL_GASPRICE, ui256_type),
            Some(BuiltIn::MsgSender) => (solx_mlir::Builder::SOL_CALLER, address_type),
            Some(BuiltIn::MsgValue) => (solx_mlir::Builder::SOL_CALLVALUE, ui256_type),
            Some(BuiltIn::BlockTimestamp) => (solx_mlir::Builder::SOL_TIMESTAMP, ui256_type),
            Some(BuiltIn::BlockNumber) => (solx_mlir::Builder::SOL_BLOCKNUMBER, ui256_type),
            Some(BuiltIn::BlockCoinbase) => (solx_mlir::Builder::SOL_COINBASE, address_type),
            Some(BuiltIn::BlockChainid) => (solx_mlir::Builder::SOL_CHAINID, ui256_type),
            Some(BuiltIn::BlockBasefee) => (solx_mlir::Builder::SOL_BASEFEE, ui256_type),
            Some(BuiltIn::BlockGaslimit) => (solx_mlir::Builder::SOL_GASLIMIT, ui256_type),
            _ => anyhow::bail!("unsupported member access: {}", member_identifier.name()),
        };
        let value = builder.emit_sol_intrinsic(intrinsic, result_type, &block);
        Ok((value, block))
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
