//!
//! Calls whose callee is a `new` expression.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::TypeName as SlangTypeName;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::expression::call::contract_creation::ContractCreation;

/// A call whose callee is a Solidity `new` expression.
pub struct NewExpressionCall {
    /// The full call expression.
    pub call: FunctionCallExpression,
    /// The call arguments.
    pub arguments: ArgumentsDeclaration,
}

impl NewExpressionCall {
    /// Classifies a call to `new`.
    pub fn from_call(call: &FunctionCallExpression, callee: &Expression) -> Option<Self> {
        if !matches!(callee, Expression::NewExpression(_)) {
            return None;
        }
        Some(Self {
            call: call.clone(),
            arguments: call.arguments(),
        })
    }

    /// Emits the `new` call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let slang_type = self.call.get_type();
        let dynamic_result_type = match &slang_type {
            Some(inner @ (SlangType::Array(_) | SlangType::Bytes(_) | SlangType::String(_))) => {
                Some(AstType::resolve(
                    inner,
                    LocationPolicy::Declared(Some(DataLocation::Memory)),
                    context.state,
                ))
            }
            None if matches!(
                self.call.operand(),
                Expression::NewExpression(new_expression)
                    if matches!(new_expression.type_name(), SlangTypeName::ElementaryType(_))
            ) =>
            {
                Some(AstType::string(context.state.mlir(), DataLocation::Memory).into_mlir())
            }
            _ => None,
        };
        if let Some(result_type) = dynamic_result_type {
            let ArgumentsDeclaration::PositionalArguments(positional) = &self.arguments else {
                unreachable!("named arguments on a new array/bytes/string are not supported");
            };
            let BlockAnd {
                value: values,
                block: current_block,
            } = positional.emit(context, block);
            let state = context.state;
            let address = match values.first() {
                Some(&size_value) => {
                    let size = AstValue::from(size_value)
                        .cast(
                            AstType::unsigned(state.mlir(), solx_utils::BIT_LENGTH_FIELD),
                            state,
                            &current_block,
                        )
                        .into_mlir();
                    AstValue::malloc(result_type, Some(size), true, state, &current_block)
                        .into_mlir()
                }
                None => {
                    unreachable!("new array/bytes/string requires a size argument")
                }
            };
            return BlockAnd {
                value: vec![address],
                block: current_block,
            };
        }

        let Some(SlangType::Contract(contract_type)) = slang_type else {
            unreachable!("new expression has no resolved type or unsupported new target");
        };
        let Definition::Contract(contract_definition) = contract_type.definition() else {
            unreachable!("Slang ContractType always references a Contract definition");
        };
        let parameter_ids: Vec<NodeId> = contract_definition
            .constructor()
            .map(|constructor| {
                constructor
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect()
            })
            .unwrap_or_default();
        let ordered_arguments = CallArguments::for_parameter_ids(&self.arguments, &parameter_ids);
        let creation = ContractCreation::new(contract_definition, ordered_arguments);
        let BlockAnd { value, block } = creation.emit(context, call_value, salt, false, block);
        BlockAnd {
            value: vec![value],
            block,
        }
    }
}
