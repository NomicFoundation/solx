//!
//! External contract member calls and getter calls.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::getter::GetterSignature;

/// An external member call to a function or generated getter.
pub struct ExternalMemberCall {
    /// The member access that selected the callee.
    pub access: MemberAccessExpression,
    /// The resolved function or state variable.
    pub definition: Definition,
    /// Arguments ordered against ABI parameters.
    pub arguments: CallArguments,
}

impl ExternalMemberCall {
    /// Classifies an external member call.
    pub fn from_callee(callee: &Expression, arguments: &ArgumentsDeclaration) -> Option<Self> {
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        let definition = access.member().resolve_to_definition()?;
        let arguments = match &definition {
            Definition::Function(function) if function.compute_selector().is_some() => {
                let parameter_ids: Vec<NodeId> = function
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                CallArguments::for_parameter_ids(arguments, &parameter_ids)
            }
            Definition::StateVariable(_) => CallArguments::positional(arguments),
            _ => return None,
        };
        Some(Self {
            access: access.clone(),
            definition,
            arguments,
        })
    }

    /// Emits the external call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let (selector, parameter_types, return_types, is_static) = match &self.definition {
            Definition::Function(function) => {
                let (parameter_types, return_types) = AstType::resolve_signature(
                    function,
                    LocationPolicy::ForceMemory,
                    &context.state.builder,
                );
                (
                    function.compute_selector().expect("slang validated"),
                    parameter_types,
                    return_types,
                    matches!(
                        function.mutability(),
                        FunctionMutability::View | FunctionMutability::Pure
                    ),
                )
            }
            Definition::StateVariable(state_variable) => {
                let builder = &context.state.builder;
                let Some((parameter_types, return_types)) =
                    state_variable.getter_signature(builder)
                else {
                    unreachable!("slang rejects a getter on a struct with no returnable members");
                };
                (
                    state_variable.compute_selector().expect("slang validated"),
                    parameter_types,
                    return_types,
                    false,
                )
            }
            _ => unreachable!("an external member call resolves to a function or state variable"),
        };
        let BlockAnd {
            value: receiver,
            block,
        } = self.access.operand().emit(context, block);
        let BlockAnd {
            value: argument_values,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let builder = &context.state.builder;
        let callee = AstValue::external_callee(
            receiver,
            selector,
            &parameter_types,
            &return_types,
            builder,
            &block,
        );
        let results = callee.call_indirect(
            &argument_values,
            &return_types,
            call_value,
            call_gas,
            is_static,
            builder,
            &block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }
}
