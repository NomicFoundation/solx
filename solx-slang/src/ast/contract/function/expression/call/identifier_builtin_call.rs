//!
//! Identifier-position Solidity built-in calls.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use solx_mlir::ods::sol::AddModOperation;
use solx_mlir::ods::sol::AssertOperation;
use solx_mlir::ods::sol::BlobHashOperation;
use solx_mlir::ods::sol::BlockHashOperation;
use solx_mlir::ods::sol::EcrecoverOperation;
use solx_mlir::ods::sol::MulModOperation;
use solx_mlir::ods::sol::RequireOperation;
use solx_mlir::ods::sol::Ripemd160Operation;
use solx_mlir::ods::sol::SelfdestructOperation;
use solx_mlir::ods::sol::Sha256Operation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A Solidity built-in called by identifier.
pub struct IdentifierBuiltinCall {
    /// The resolved built-in.
    pub built_in: BuiltIn,
    /// The call arguments.
    pub arguments: CallArguments,
}

impl IdentifierBuiltinCall {
    /// Classifies an identifier-position built-in call.
    pub fn from_callee(
        callee: &Expression,
        arguments: &slang_solidity_v2::ast::ArgumentsDeclaration,
    ) -> Option<Self> {
        let Expression::Identifier(identifier) = callee else {
            return None;
        };
        let built_in = identifier.resolve_to_built_in()?;
        if !matches!(
            built_in,
            BuiltIn::Assert
                | BuiltIn::Require
                | BuiltIn::Gasleft
                | BuiltIn::Blockhash
                | BuiltIn::Keccak256
                | BuiltIn::Sha256
                | BuiltIn::Ripemd160
                | BuiltIn::Ecrecover
                | BuiltIn::Addmod
                | BuiltIn::Mulmod
                | BuiltIn::Selfdestruct
                | BuiltIn::Blobhash
        ) {
            return None;
        }
        Some(Self {
            built_in,
            arguments: CallArguments::positional(arguments),
        })
    }

    /// Emits the built-in call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        match self.built_in {
            BuiltIn::Assert => self.emit_assert(context, block),
            BuiltIn::Require => self.emit_require(context, block),
            BuiltIn::Gasleft => BlockAnd {
                value: vec![AstValue::gas_left(context.state, &block).into_mlir()],
                block,
            },
            BuiltIn::Blockhash => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let block_number = AstValue::from(values[0])
                    .cast(
                        AstType::unsigned(state.mlir(), solx_utils::BIT_LENGTH_FIELD),
                        state,
                        &block,
                    )
                    .into_mlir();
                let value = mlir_op!(
                    state,
                    block,
                    BlockHashOperation
                        .block_number(block_number)
                        .val(AstType::fixed_bytes(state.mlir(), 32))
                );
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Blobhash => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let index = AstValue::from(values[0])
                    .cast(
                        AstType::unsigned(state.mlir(), solx_utils::BIT_LENGTH_FIELD),
                        state,
                        &block,
                    )
                    .into_mlir();
                let value = mlir_op!(
                    state,
                    block,
                    BlobHashOperation
                        .idx(index)
                        .val(AstType::fixed_bytes(state.mlir(), 32))
                );
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Selfdestruct => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let recipient = AstValue::from(values[0])
                    .cast(AstType::address(state.mlir(), true), state, &block)
                    .into_mlir();
                mlir_op_void!(state, &block, SelfdestructOperation.recipient(recipient));
                BlockAnd {
                    value: vec![],
                    block,
                }
            }
            BuiltIn::Keccak256 => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let value = AstValue::keccak256(AstValue::from(values[0]), context.state, &block)
                    .into_mlir();
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Sha256 => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let value = mlir_op!(
                    state,
                    block,
                    Sha256Operation
                        .data(values[0])
                        .result(AstType::fixed_bytes(state.mlir(), 32))
                );
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Ripemd160 => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let value = mlir_op!(
                    state,
                    block,
                    Ripemd160Operation
                        .data(values[0])
                        .result(AstType::fixed_bytes(state.mlir(), 20))
                );
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Ecrecover => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let bytes32 = AstType::fixed_bytes(state.mlir(), 32).into_mlir();
                let ui8 = Type::from(IntegerType::unsigned(state.mlir(), 8));
                let hash = AstValue::from(values[0])
                    .cast(AstType::new(bytes32), state, &block)
                    .into_mlir();
                let v = AstValue::from(values[1])
                    .cast(AstType::new(ui8), state, &block)
                    .into_mlir();
                let r = AstValue::from(values[2])
                    .cast(AstType::new(bytes32), state, &block)
                    .into_mlir();
                let s = AstValue::from(values[3])
                    .cast(AstType::new(bytes32), state, &block)
                    .into_mlir();
                let value = mlir_op!(
                    state,
                    block,
                    EcrecoverOperation
                        .hash(hash)
                        .v(v)
                        .r(r)
                        .s(s)
                        .result(AstType::address(state.mlir(), false))
                );
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            BuiltIn::Addmod | BuiltIn::Mulmod => {
                let BlockAnd {
                    value: values,
                    block,
                } = self.arguments.emit(context, block);
                let state = context.state;
                let ui256 =
                    AstType::unsigned(state.mlir(), solx_utils::BIT_LENGTH_FIELD).into_mlir();
                let x = AstValue::from(values[0])
                    .cast(AstType::new(ui256), state, &block)
                    .into_mlir();
                let y = AstValue::from(values[1])
                    .cast(AstType::new(ui256), state, &block)
                    .into_mlir();
                let modulus = AstValue::from(values[2])
                    .cast(AstType::new(ui256), state, &block)
                    .into_mlir();
                let value = if matches!(self.built_in, BuiltIn::Addmod) {
                    mlir_op!(state, block, AddModOperation.x(x).y(y).r#mod(modulus))
                } else {
                    mlir_op!(state, block, MulModOperation.x(x).y(y).r#mod(modulus))
                };
                BlockAnd {
                    value: vec![value],
                    block,
                }
            }
            _ => unreachable!("only identifier built-ins are classified here"),
        }
    }

    /// Emits `assert`.
    pub fn emit_assert<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let condition = self
            .arguments
            .expressions
            .iter()
            .next()
            .expect("assert has one argument");
        let BlockAnd {
            value: condition_value,
            block,
        } = condition.emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(context.state, &block)
            .into_mlir();
        mlir_op_void!(
            context.state,
            &block,
            AssertOperation.cond(condition_boolean)
        );
        BlockAnd {
            value: vec![],
            block,
        }
    }

    /// Emits `require`.
    pub fn emit_require<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let mut iter = self.arguments.expressions.iter();
        let condition = iter.next().expect("require has a condition argument");
        let message = iter.next();
        let BlockAnd {
            value: condition_value,
            block,
        } = condition.emit(context, block);
        let condition_boolean = condition_value
            .is_nonzero(context.state, &block)
            .into_mlir();
        let state = context.state;
        let block = match message {
            Some(Expression::StringExpression(string_expression)) => {
                let bytes = string_expression.value();
                let literal = String::from_utf8(bytes).expect("require message is valid UTF-8");
                mlir_op_void!(
                    state,
                    &block,
                    RequireOperation
                        .cond(condition_boolean)
                        .args(&[])
                        .msg(StringAttribute::new(state.mlir(), &literal))
                );
                block
            }
            Some(expression) => {
                if let Expression::FunctionCallExpression(error_call) = expression
                    && let Some(Definition::Error(error_definition)) = (match error_call.operand() {
                        Expression::Identifier(identifier) => identifier.resolve_to_definition(),
                        Expression::MemberAccessExpression(access) => {
                            access.member().resolve_to_definition()
                        }
                        _ => None,
                    })
                {
                    let signature = error_definition
                        .compute_canonical_signature()
                        .expect("slang validated");
                    let parameters = error_definition.parameters();
                    let parameter_ids: Vec<NodeId> = parameters
                        .iter()
                        .map(|parameter| parameter.node_id())
                        .collect();
                    let parameter_types: Vec<_> = parameters
                        .iter()
                        .map(|parameter| {
                            AstType::resolve(
                                &parameter.get_type().expect("slang validated"),
                                LocationPolicy::Declared(None),
                                context.state,
                            )
                        })
                        .collect();
                    let error_arguments =
                        CallArguments::for_parameter_ids(&error_call.arguments(), &parameter_ids);
                    let BlockAnd {
                        value: argument_values,
                        block: current_block,
                    } = error_arguments.emit_as(&parameter_types, context, block);
                    let state = context.state;
                    mlir_op_void!(
                        state,
                        &current_block,
                        RequireOperation
                            .cond(condition_boolean)
                            .args(&argument_values)
                            .msg(StringAttribute::new(state.mlir(), &signature))
                            .call(Attribute::unit(state.mlir()))
                    );
                    current_block
                } else {
                    let BlockAnd {
                        value: message_value,
                        block,
                    } = expression.emit(context, block);
                    let string_memory_type =
                        AstType::string(state.mlir(), solx_utils::DataLocation::Memory).into_mlir();
                    let message_value = message_value
                        .cast(AstType::new(string_memory_type), state, &block)
                        .into_mlir();
                    mlir_op_void!(
                        state,
                        &block,
                        RequireOperation
                            .cond(condition_boolean)
                            .args(&[message_value])
                            .msg(StringAttribute::new(state.mlir(), "Error(string)"))
                            .call(Attribute::unit(state.mlir()))
                    );
                    block
                }
            }
            None => {
                mlir_op_void!(
                    state,
                    &block,
                    RequireOperation.cond(condition_boolean).args(&[])
                );
                block
            }
        };
        BlockAnd {
            value: vec![],
            block,
        }
    }
}
