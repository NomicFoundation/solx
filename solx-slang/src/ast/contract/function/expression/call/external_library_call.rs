//!
//! External calls to library functions.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Value;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::FunctionType;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use solx_mlir::ods::sol::ExtCallOperation;

use crate::ast::BlockAnd;
use crate::ast::EmitExpression;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::analysis::query::MemberAccessOperand;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;

/// An external call to a library function.
pub struct ExternalLibraryCall {
    /// The member access that selected the library function.
    pub access: MemberAccessExpression,
    /// The resolved library function.
    pub function: FunctionDefinition,
    /// Receiver value supplied as the library self parameter, if any.
    pub self_receiver: Option<Expression>,
    /// Arguments ordered against the library parameters.
    pub arguments: CallArguments,
}

impl ExternalLibraryCall {
    /// Classifies an external library call.
    pub fn from_callee(callee: &Expression, arguments: &ArgumentsDeclaration) -> Option<Self> {
        let Expression::MemberAccessExpression(access) = callee else {
            return None;
        };
        let Some(Definition::Function(function)) = access.member().resolve_to_definition() else {
            return None;
        };
        if function.compute_selector().is_none()
            || !(matches!(&access.operand(), Expression::Identifier(identifier)
                    if matches!(identifier.resolve_to_definition(), Some(Definition::Library(_))))
                || matches!(
                    function.enclosing_definition(),
                    Some(Definition::Library(_))
                ))
        {
            return None;
        }
        let Some(Definition::Library(_)) = function.enclosing_definition() else {
            unreachable!("an external library call's target is a library member");
        };
        let library_operand = access.operand();
        let self_receiver = (!MemberAccessOperand(&library_operand).is_namespace_qualifier())
            .then_some(library_operand);
        let parameter_ids: Vec<NodeId> = function
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();
        let arguments = if self_receiver.is_some() {
            CallArguments::after_receiver(arguments, &parameter_ids)
        } else {
            CallArguments::for_parameter_ids(arguments, &parameter_ids)
        };
        Some(Self {
            access: access.clone(),
            function,
            self_receiver,
            arguments,
        })
    }

    /// Emits the library call.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let Some(Definition::Library(library)) = self.function.enclosing_definition() else {
            unreachable!("an external library call's target is a library member");
        };
        let library_name = solx_utils::ContractName::new(
            library.get_file_id().to_owned(),
            Some(library.name().name()),
        );
        let (parameter_types, _) = AstType::resolve_signature(
            &self.function,
            LocationPolicy::Declared(None),
            &context.state.builder,
        );
        let return_types: Vec<_> = match self.function.returns() {
            Some(returns) => returns
                .iter()
                .map(|parameter| {
                    let policy = if matches!(
                        parameter.storage_location(),
                        Some(slang_solidity_v2::ast::StorageLocation::CallDataKeyword(_))
                    ) {
                        LocationPolicy::ForceMemory
                    } else {
                        LocationPolicy::Declared(None)
                    };
                    AstType::resolve(
                        &parameter.get_type().expect("slang validated"),
                        policy,
                        &context.state.builder,
                    )
                })
                .collect(),
            None => Vec::new(),
        };
        let selector = self.function.compute_selector().expect("slang validated");
        let mlir_name = self.function.mlir_function_name();
        let (argument_values, current_block) = match &self.self_receiver {
            Some(receiver) => {
                let (parameter_self, parameter_rest) =
                    parameter_types.split_first().expect("slang validated");
                let BlockAnd {
                    value: self_value,
                    block,
                } = receiver.emit(context, block);
                let builder = &context.state.builder;
                let self_value = self_value
                    .cast(AstType::new(*parameter_self), builder, &block)
                    .into_mlir();
                let BlockAnd {
                    value: mut rest_values,
                    block,
                } = self.arguments.emit_as(parameter_rest, context, block);
                rest_values.insert(0, self_value);
                (rest_values, block)
            }
            None => {
                let BlockAnd { value, block } =
                    self.arguments.emit_as(&parameter_types, context, block);
                (value, block)
            }
        };
        let builder = &context.state.builder;
        let address = AstValue::library_address(&library_name, builder, &current_block).into_mlir();
        let callee_type = FunctionType::new(builder.context, &parameter_types, &return_types);
        let gas = AstValue::gas_left(builder, &current_block).into_mlir();
        let value = AstValue::uint256(0, builder, &current_block).into_mlir();
        let selector_value =
            AstValue::uint256(i64::from(selector), builder, &current_block).into_mlir();
        let operation = current_block.append_operation(mlir_op_build!(
            builder,
            ExtCallOperation
                .callee(StringAttribute::new(builder.context, &mlir_name))
                .ins(&argument_values)
                .addr(address)
                .gas(gas)
                .val(value)
                .selector(selector_value)
                .delegate_call(Attribute::unit(builder.context))
                .library_call(Attribute::unit(builder.context))
                .callee_type(TypeAttribute::new(callee_type.into()))
                .status(AstType::signless(
                    builder.context,
                    solx_utils::BIT_LENGTH_BOOLEAN
                ))
                .outs(&return_types)
        ));
        let results = (0..return_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces the declared results")
                    .into()
            })
            .collect();
        BlockAnd {
            value: results,
            block: current_block,
        }
    }
}
