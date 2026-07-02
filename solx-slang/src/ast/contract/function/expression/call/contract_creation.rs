//!
//! Contract creation from `new C(args)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Expression;

use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::ContractEmitter;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_as::EmitAs;

impl<'emitter, 'state, 'context, 'block> CallContext<'emitter, 'state, 'context, 'block> {
    /// Emits a contract creation `new C(args)` as `sol.new`, embedding `C`'s deploy bytecode.
    ///
    /// The created contract is recorded as a cross-contract dependency so the linker pulls its object
    /// in. Each constructor argument is coerced to its declared parameter type, so a literal
    /// materialises in the parameter's representation that the deployed constructor ABI-decodes. A
    /// plain `new C()` forwards zero wei.
    pub(super) fn emit_contract_creation(
        &self,
        contract_definition: &ContractDefinition,
        arguments: &[Expression],
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let context = self.expression_context.state;
        let contract_name = contract_definition.name().name();
        context.add_dependency(contract_name.clone());

        let parameter_types = contract_definition
            .constructor()
            .map(|constructor| TypeConversion::resolve_function_types(&constructor, context).0)
            .unwrap_or_default();
        let mut constructor_arguments = Vec::with_capacity(arguments.len());
        let mut current_block = block;
        for (argument, &parameter_type) in arguments.iter().zip(&parameter_types) {
            let BlockAnd { value, block: next } =
                argument.emit_as(parameter_type, self.expression_context, current_block);
            constructor_arguments.push(value);
            current_block = next;
        }

        let payable = ContractEmitter::is_contract_payable(contract_definition);
        let result_type = AstType::contract(context.mlir_context, &contract_name, payable);
        let call_value = AstValue::constant(
            0,
            AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
            context,
            &current_block,
        );
        let value = AstValue::create_contract(
            &contract_name,
            call_value,
            None,
            &constructor_arguments,
            result_type,
            context,
            &current_block,
        )
        .into_mlir();
        (value, current_block)
    }
}
