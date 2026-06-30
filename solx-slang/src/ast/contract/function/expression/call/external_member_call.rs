//!
//! External contract member calls and getter calls.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;
use solx_mlir::Context;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::analysis::query::ParameterNodeIds;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;
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
                let parameter_ids = function.parameters().node_ids();
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
        let (callee_name, selector, parameter_types, return_types, is_static) =
            self.resolve_call(context.state);
        let BlockAnd {
            value: receiver,
            block,
        } = self.access.operand().emit(context, block);
        let BlockAnd {
            value: argument_values,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let state = context.state;
        let (_status, results) = AstValue::external_call(
            receiver,
            &callee_name,
            selector,
            &parameter_types,
            &argument_values,
            &return_types,
            call_value,
            call_gas,
            is_static,
            false,
            state,
            &block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }

    /// Emits this call with `try` semantics, returning the success status flag, the decoded results,
    /// and the continuation block.
    pub fn emit_try<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        call_gas: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let (callee_name, selector, parameter_types, return_types, _is_static) =
            self.resolve_call(context.state);
        let BlockAnd {
            value: receiver,
            block,
        } = self.access.operand().emit(context, block);
        let BlockAnd {
            value: argument_values,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let state = context.state;
        let (status, results) = AstValue::external_call(
            receiver,
            &callee_name,
            selector,
            &parameter_types,
            &argument_values,
            &return_types,
            call_value,
            call_gas,
            false,
            true,
            state,
            &block,
        );
        (status, results, block)
    }

    /// Resolves the callee's MLIR name, ABI selector, Sol-typed parameter and result types, and
    /// whether the call is to a read-only callee.
    fn resolve_call<'context>(
        &self,
        context: &Context<'context>,
    ) -> (String, u32, Vec<Type<'context>>, Vec<Type<'context>>, bool) {
        match &self.definition {
            Definition::Function(function) => {
                let (parameter_types, return_types) =
                    AstType::resolve_signature(function, LocationPolicy::ForceMemory, context);
                (
                    function.mlir_function_name(),
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
                let Some((parameter_types, return_types)) =
                    state_variable.getter_signature(context)
                else {
                    unreachable!("a public accessor with no returnable members is invalid");
                };
                (
                    state_variable
                        .compute_canonical_signature()
                        .expect("slang validated"),
                    state_variable.compute_selector().expect("slang validated"),
                    parameter_types,
                    return_types,
                    true,
                )
            }
            _ => unreachable!("an external member call resolves to a function or state variable"),
        }
    }
}
