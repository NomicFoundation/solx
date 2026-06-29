//!
//! Contract creation from `new C(args)`.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;

use crate::ast::BlockAnd;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::call_arguments::CallArguments;

/// A contract-creation call.
pub struct ContractCreation {
    /// The contract definition being created.
    pub definition: ContractDefinition,
    /// Constructor arguments ordered against the constructor parameters.
    pub arguments: CallArguments,
}

impl ContractCreation {
    /// Creates a contract-creation call form.
    pub fn new(definition: ContractDefinition, arguments: CallArguments) -> Self {
        Self {
            definition,
            arguments,
        }
    }

    /// Emits the contract creation.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call_value: Option<Value<'context, 'block>>,
        salt: Option<Value<'context, 'block>>,
        try_call: bool,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let contract_name = self.definition.name().name();
        let payable = self.definition.is_payable();
        context.state.add_dependency(contract_name.clone());

        let parameter_types = self
            .definition
            .constructor()
            .map(|constructor| {
                AstType::resolve_signature(
                    &constructor,
                    LocationPolicy::Declared(None),
                    context.state,
                )
                .0
            })
            .unwrap_or_default();
        let BlockAnd {
            value: ctor_args,
            block,
        } = self.arguments.emit_as(&parameter_types, context, block);
        let state = context.state;
        let result_type = AstType::contract(state.mlir_context, &contract_name, payable);
        let val = match call_value {
            Some(value) => AstValue::from(value),
            None => AstValue::uint256(0, state, &block),
        };
        let value = AstValue::create_contract(
            &contract_name,
            val,
            salt.map(AstValue::from),
            &ctor_args,
            result_type,
            try_call,
            state,
            &block,
        )
        .into_mlir();
        BlockAnd { value, block }
    }
}
