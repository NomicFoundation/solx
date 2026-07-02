//!
//! External calls to library functions.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StorageLocation;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Context;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::analysis::query::member_access_operand::MemberAccessOperand;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::FunctionEmitter;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_expression::EmitExpression;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits an external call into a library member `L.f(args)` / `x.f(args)` through a `DELEGATECALL`,
    /// returning all of its result values in declaration order.
    ///
    /// The library's linked address is a `sol.lib_addr` placeholder the linker resolves. A namespace-
    /// qualified access orders its arguments against the callee's parameters; a `using for` receiver
    /// evaluates the receiver, casts it to the first parameter type, and forwards it ahead of the
    /// remaining arguments.
    pub(super) fn emit_external_library_call(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        arguments: &ArgumentsDeclaration,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        let context = self.expression_context.state;
        let Some(Definition::Library(library)) = function_definition.enclosing_definition() else {
            unreachable!("an external library call's target is a library member");
        };
        let library_name = solx_utils::ContractName::new(
            library.get_file_id().to_owned(),
            Some(library.name().name()),
        );
        let (parameter_types, _) =
            TypeConversion::resolve_function_types(function_definition, context);
        let return_types = Self::library_return_types(function_definition, context);
        let selector = function_definition
            .compute_selector()
            .expect("an external library call resolves to a selector-bearing member");
        let callee_name = FunctionEmitter::mlir_function_name(function_definition);
        let parameter_ids: Vec<NodeId> = function_definition
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();

        let operand = access.operand();
        let (argument_values, block) =
            if MemberAccessOperand(&operand).is_namespace_qualifier() {
                self.emit_ordered_arguments(arguments, &parameter_ids, &parameter_types, block)
            } else {
                let (&receiver_type, rest_types) = parameter_types
                    .split_first()
                    .expect("a `using for` receiver occupies the first parameter");
                let BlockAnd {
                    value: receiver,
                    block,
                } = operand.emit(self.expression_context, block);
                let receiver =
                    TypeConversion::from_target_type(receiver_type, context).emit(receiver, context, &block);
                let (mut argument_values, block) =
                    self.emit_ordered_arguments(arguments, &parameter_ids[1..], rest_types, block);
                argument_values.insert(0, receiver);
                (argument_values, block)
            };

        let address = AstValue::library_address(&library_name, context, &block);
        let results = AstValue::library_call(
            address,
            &callee_name,
            selector,
            &parameter_types,
            &argument_values,
            &return_types,
            context,
            &block,
        );
        BlockAnd {
            value: results,
            block,
        }
    }

    /// The library callee's return types, relocating a returned `calldata` reference to `memory`: a
    /// calldata reference cannot cross a call boundary, so the decoded `bytes` / `string` result lives
    /// in memory.
    fn library_return_types(
        function_definition: &FunctionDefinition,
        context: &Context<'context>,
    ) -> Vec<Type<'context>> {
        let Some(returns) = function_definition.returns() else {
            return Vec::new();
        };
        returns
            .iter()
            .map(|parameter| {
                let slang_type = parameter.get_type().expect("slang types every return parameter");
                if matches!(
                    parameter.storage_location(),
                    Some(StorageLocation::CallDataKeyword(_))
                ) && matches!(slang_type, SlangType::Bytes(_) | SlangType::String(_))
                {
                    return AstType::string(
                        context.mlir_context,
                        solx_utils::DataLocation::Memory,
                    )
                    .into_mlir();
                }
                TypeConversion::resolve_slang_type(&slang_type, None, context)
            })
            .collect()
    }
}
