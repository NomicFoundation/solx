//!
//! Identifier-position Solidity built-in calls.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::ErrorDefinition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::PositionalArguments;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::AssertOperation;
use solx_mlir::ods::sol::BlobHashOperation;
use solx_mlir::ods::sol::BlockHashOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::GasLeftOperation;
use solx_mlir::ods::sol::Keccak256Operation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::RequireOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::SelfdestructOperation;
use solx_mlir::ods::sol::Sha256Operation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Tries to emit `callee(arguments)` as a Solidity built-in invoked by bare identifier.
    ///
    /// Resolves the callee via slang's binder to a [`BuiltIn`] variant. On match, returns
    /// `Some((value, block))`, where `value` is `Some(...)` for value-producing built-ins
    /// (e.g. `gasleft()`) and `None` for statement-style built-ins (e.g. `assert`, `require`).
    /// Returns `None` if the callee is not a built-in.
    pub(super) fn try_emit_built_in_call(
        &self,
        callee: &Expression,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let built_in = identifier.resolve_to_built_in()?;
        match built_in {
            BuiltIn::Assert if arguments.len() == 1 => {
                let condition = arguments.iter().next().expect("argument count verified");
                Some((None, self.emit_assert(&condition, block)))
            }
            BuiltIn::Require if matches!(arguments.len(), 1 | 2) => {
                let mut iter = arguments.iter();
                let condition = iter.next().expect("argument count verified");
                let message = iter.next();
                Some((None, self.emit_require(&condition, message.as_ref(), block)))
            }
            BuiltIn::Gasleft if arguments.is_empty() => {
                let context = self.expression_context.state;
                let value = block
                    .append_operation(
                        GasLeftOperation::builder(context.mlir_context, context.location())
                            .val(AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("gasleft always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Blockhash if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let block_number = AstValue::new(values[0])
                    .cast(
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                        context,
                        &block,
                    )
                    .into_mlir();
                let value = block
                    .append_operation(
                        BlockHashOperation::builder(context.mlir_context, context.location())
                            .block_number(block_number)
                            .val(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("blockhash always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Blobhash if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let index = AstValue::new(values[0])
                    .cast(
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                        context,
                        &block,
                    )
                    .into_mlir();
                let value = block
                    .append_operation(
                        BlobHashOperation::builder(context.mlir_context, context.location())
                            .idx(index)
                            .val(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("blobhash always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Selfdestruct if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let recipient = AstValue::new(values[0])
                    .address_cast(
                        AstType::address(context.mlir_context, true),
                        context,
                        &block,
                    )
                    .into_mlir();
                block.append_operation(
                    SelfdestructOperation::builder(context.mlir_context, context.location())
                        .recipient(recipient)
                        .build()
                        .into(),
                );
                Some((None, block))
            }
            BuiltIn::Keccak256 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let value = block
                    .append_operation(
                        Keccak256Operation::builder(context.mlir_context, context.location())
                            .addr(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("keccak256 always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Sha256 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let value = block
                    .append_operation(
                        Sha256Operation::builder(context.mlir_context, context.location())
                            .data(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 32).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sha256 always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Ripemd160 if arguments.len() == 1 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let value = block
                    .append_operation(
                        Ripemd160Operation::builder(context.mlir_context, context.location())
                            .data(values[0])
                            .result(AstType::fixed_bytes(context.mlir_context, 20).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("ripemd160 always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Ecrecover if arguments.len() == 4 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let value = block
                    .append_operation(
                        EcrecoverOperation::builder(context.mlir_context, context.location())
                            .hash(values[0])
                            .v(values[1])
                            .r(values[2])
                            .s(values[3])
                            .result(AstType::address(context.mlir_context, false).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("ecrecover always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Addmod if arguments.len() == 3 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let field = AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD);
                let value = block
                    .append_operation(
                        AddModOperation::builder(context.mlir_context, context.location())
                            .x(AstValue::new(values[0]).cast(field, context, &block).into_mlir())
                            .y(AstValue::new(values[1]).cast(field, context, &block).into_mlir())
                            .r#mod(AstValue::new(values[2]).cast(field, context, &block).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("addmod always produces one result")
                    .into();
                Some((Some(value), block))
            }
            BuiltIn::Mulmod if arguments.len() == 3 => {
                let (values, block) = self.emit_argument_values(arguments, block);
                let context = self.expression_context.state;
                let field = AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD);
                let value = block
                    .append_operation(
                        MulModOperation::builder(context.mlir_context, context.location())
                            .x(AstValue::new(values[0]).cast(field, context, &block).into_mlir())
                            .y(AstValue::new(values[1]).cast(field, context, &block).into_mlir())
                            .r#mod(AstValue::new(values[2]).cast(field, context, &block).into_mlir())
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("mulmod always produces one result")
                    .into();
                Some((Some(value), block))
            }
            _ => None,
        }
    }

    /// Emits an `assert(condition)` built-in via `sol.assert`.
    fn emit_assert(
        &self,
        condition: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let BlockAnd { value: condition_value, block } =
            condition.emit(self.expression_context, block);
        let condition_boolean = self
            .expression_context
            .emit_is_nonzero(condition_value, &block);
        let context = self.expression_context.state;
        mlir_op_void!(context, &block, AssertOperation.cond(condition_boolean));
        block
    }

    /// Emits a `require(condition)` or `require(condition, message)` built-in
    /// via `sol.require`.
    ///
    /// Literal string messages lower to `sol.require %cond, "msg" : ()`. A
    /// `require(cond, CustomError(args))` lowers to the `call` form carrying the
    /// error's canonical signature and ABI-encoded arguments. Any other runtime
    /// expression evaluates at runtime and is ABI-encoded under the
    /// `Error(string)` selector via the `call` form of `sol.require`.
    fn emit_require(
        &self,
        condition: &Expression,
        message: Option<&Expression>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let BlockAnd { value: condition_value, block } =
            condition.emit(self.expression_context, block);
        let condition_boolean = self
            .expression_context
            .emit_is_nonzero(condition_value, &block);
        let context = self.expression_context.state;
        match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes).expect("require message is valid UTF-8");
                let operation_builder =
                    RequireOperation::builder(context.mlir_context, context.location())
                        .cond(condition_boolean)
                        .args(&[])
                        .msg(melior::ir::attribute::StringAttribute::new(context.mlir_context, &literal));
                block.append_operation(operation_builder.build().into());
                block
            }
            Some(Expression::FunctionCallExpression(error_call))
                if let Expression::Identifier(identifier) = error_call.operand()
                    && let Some(Definition::Error(error_definition)) =
                        identifier.resolve_to_definition() =>
            {
                self.emit_require_custom_error(
                    condition_boolean,
                    &error_definition,
                    &error_call.arguments(),
                    block,
                )
            }
            Some(expression) => {
                let BlockAnd { value: message_value, block } =
                    expression.emit(self.expression_context, block);
                let string_memory_type = AstType::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir();
                let message_value = TypeConversion::from_target_type(string_memory_type, context)
                    .emit(message_value, context, &block);
                let operation_builder =
                    RequireOperation::builder(context.mlir_context, context.location())
                        .cond(condition_boolean)
                        .args(&[message_value])
                        .msg(melior::ir::attribute::StringAttribute::new(context.mlir_context, "Error(string)"))
                        .call(melior::ir::Attribute::unit(context.mlir_context));
                block.append_operation(operation_builder.build().into());
                block
            }
            None => {
                let operation_builder =
                    RequireOperation::builder(context.mlir_context, context.location())
                        .cond(condition_boolean)
                        .args(&[]);
                block.append_operation(operation_builder.build().into());
                block
            }
        }
    }

    /// Emits `require(condition, CustomError(args))` as the `call` form of
    /// `sol.require`, carrying the error's canonical signature and its
    /// ABI-encoded arguments coerced to the declared parameter types.
    fn emit_require_custom_error(
        &self,
        condition_boolean: Value<'context, 'block>,
        error_definition: &ErrorDefinition,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let signature = error_definition
            .compute_canonical_signature()
            .expect("canonical signature is computable for a custom error");
        let parameters = error_definition.parameters();
        let parameter_ids: Vec<NodeId> =
            parameters.iter().map(|parameter| parameter.node_id()).collect();
        let ordered_arguments = arguments
            .ordered_by(&parameter_ids)
            .expect("slang matches every require custom-error argument to a parameter");
        let context = self.expression_context.state;
        let mut argument_values = Vec::with_capacity(ordered_arguments.len());
        let mut current_block = block;
        for (parameter, argument) in parameters.iter().zip(ordered_arguments) {
            let BlockAnd { value, block: next_block } =
                argument.emit(self.expression_context, current_block);
            current_block = next_block;
            let parameter_type = TypeConversion::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                context,
            );
            let value = TypeConversion::from_target_type(parameter_type, context).emit(
                value,
                context,
                &current_block,
            );
            argument_values.push(value);
        }
        let operation_builder =
            RequireOperation::builder(context.mlir_context, context.location())
                .cond(condition_boolean)
                .args(&argument_values)
                .msg(melior::ir::attribute::StringAttribute::new(context.mlir_context, &signature))
                .call(melior::ir::Attribute::unit(context.mlir_context));
        current_block.append_operation(operation_builder.build().into());
        current_block
    }
}
