//!
//! The contract creation `new C(args)` a `try` statement guards.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::CallContext;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// The contract creation `try new C(args)` guards, resolved ahead of emission so [`Self::emit`]
/// surfaces the creation's success status for the `try` op's regions.
pub struct TryNewExpression {
    /// The `{value: v}` / `{salt: s}` options layer, if any (`new C{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The contract being created.
    contract_definition: ContractDefinition,
    /// The constructor arguments, ordered against the constructor's parameters.
    arguments: Vec<Expression>,
}

impl TryNewExpression {
    /// Classifies `try new C(args)`: a contract creation, optionally in a call-options layer. A
    /// dynamic-aggregate `new T[](n)` / `new bytes(n)` or any other guarded shape yields `None`.
    pub fn from_expression(expression: &Expression) -> Option<Self> {
        let Expression::FunctionCallExpression(call) = expression else {
            return None;
        };
        let options = match call.operand().unwrap_parentheses() {
            Expression::NewExpression(_) => None,
            Expression::CallOptionsExpression(options) => {
                if !matches!(
                    options.operand().unwrap_parentheses(),
                    Expression::NewExpression(_)
                ) {
                    return None;
                }
                Some(options)
            }
            _ => return None,
        };
        let Some(SlangType::Contract(contract_type)) = call.get_type() else {
            return None;
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
        let arguments = call
            .arguments()
            .ordered_by(&parameter_ids)
            .expect("slang matches every constructor argument to a parameter");
        Some(Self {
            options,
            contract_definition,
            arguments,
        })
    }

    /// Emits the guarded creation with `try` semantics, returning its success status, the created
    /// contract as its single result, and the continuation block.
    ///
    /// The status mirrors solc: the created contract casts to `address`, then to `ui160`, and a
    /// `sol.cmp ne` against zero flags a non-null deployment as success.
    pub fn emit<'state, 'context: 'block, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let mut current_block = block;
        let mut call_value = None;
        let mut salt = None;
        if let Some(options) = &self.options {
            let (value, salt_value, _gas, next_block) =
                CallOptions(options).capture(context, current_block);
            current_block = next_block;
            call_value = value;
            salt = salt_value;
        }

        let (contract_value, current_block) = CallContext::new(context).emit_contract_creation(
            &self.contract_definition,
            &self.arguments,
            call_value,
            salt,
            true,
            current_block,
        );

        let state = context.state;
        let ui160 = AstType::unsigned(state.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS);
        let as_ui160 = AstValue::new(contract_value)
            .address_cast(AstType::address(state.mlir_context, false), state, &current_block)
            .address_cast(ui160, state, &current_block);
        let zero = AstValue::constant(0, ui160, state, &current_block);
        let status = as_ui160
            .compare(zero, CmpPredicate::Ne, state, &current_block)
            .into_mlir();
        (status, vec![contract_value], current_block)
    }
}
