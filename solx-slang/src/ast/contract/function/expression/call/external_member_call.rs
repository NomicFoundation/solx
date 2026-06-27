//!
//! External contract member calls and getter calls.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionMutability;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use solx_mlir::Builder;
use solx_mlir::ods::sol::ExtICallOperation;

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
        let (selector, parameter_types, return_types, is_static) =
            self.resolve_call(&context.state.builder);
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
        let (selector, parameter_types, return_types, _is_static) =
            self.resolve_call(&context.state.builder);
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
        )
        .into_mlir();
        let value = call_value.unwrap_or_else(|| AstValue::uint256(0, builder, &block).into_mlir());
        let gas = call_gas.unwrap_or_else(|| AstValue::gas_left(builder, &block).into_mlir());
        let mut out_types = Vec::with_capacity(return_types.len() + 1);
        out_types
            .push(AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
        out_types.extend_from_slice(&return_types);
        let operation = block.append_operation(mlir_op_build!(
            builder,
            ExtICallOperation
                .outs(&out_types)
                .callee(callee)
                .callee_operands(&argument_values)
                .gas(gas)
                .value(value)
                .try_call(Attribute::unit(builder.context))
        ));
        let status = operation
            .result(0)
            .expect("sol.ext_icall try produces a status result")
            .into();
        let results = (0..return_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_icall try produces a status plus its declared results")
                    .into()
            })
            .collect();
        (status, results, block)
    }

    /// Resolves the callee's ABI selector, Sol-typed parameter and result types, and whether the
    /// call is to a read-only callee.
    fn resolve_call<'context>(
        &self,
        builder: &Builder<'context>,
    ) -> (u32, Vec<Type<'context>>, Vec<Type<'context>>, bool) {
        match &self.definition {
            Definition::Function(function) => {
                let (parameter_types, return_types) =
                    AstType::resolve_signature(function, LocationPolicy::ForceMemory, builder);
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
                let Some((parameter_types, return_types)) = state_variable.getter_signature(builder)
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
        }
    }
}
