//!
//! A contract-creation `new C(args)` in `try` position, classified ahead of emission.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::CmpPredicate;

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::ContractCreation;
use crate::ast::contract::function::expression::call_options::CallOptions;

/// A `try new C(args)` contract creation, resolved from the `try` expression (the other shape,
/// besides an external call, that carries a real catch path), so [`Self::emit`] is an infallible emitter.
pub struct TryNewExpression {
    /// The `{value: v}` / `{salt: s}` options layer, if any (`new C{value: v}(args)`).
    options: Option<CallOptionsExpression>,
    /// The contract being created.
    contract_definition: ContractDefinition,
    /// The positional constructor arguments.
    arguments: PositionalArguments,
}

impl TryNewExpression {
    /// Lowerable only when the `try` wraps a contract creation `new C(args)` (optionally in a
    /// call-options layer); a dynamic-aggregate `new T[](n)` / `new bytes(n)` or any other shape
    /// yields `None`.
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
            return None;
        };
        let ArgumentsDeclaration::PositionalArguments(arguments) = call.arguments() else {
            return None;
        };
        Some(Self {
            options,
            contract_definition,
            arguments,
        })
    }

    /// Emits the creation with `try` semantics, returning the success status flag, the created
    /// contract (the single declared result), and the continuation block.
    pub fn emit<'state, 'context, 'block>(
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
        let (contract_value, current_block) = self.contract_definition.emit_creation(
            context,
            self.arguments.iter().collect(),
            call_value,
            salt,
            true,
            current_block,
        );
        let builder = &context.state.builder;
        let ui160 = AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS);
        let address = AstValue::from(contract_value)
            .cast(AstType::address(builder.context, false), builder, &current_block);
        let as_ui160 = address.cast(ui160, builder, &current_block);
        let zero = AstValue::constant(0, ui160, builder, &current_block);
        let status = as_ui160
            .compare(zero, CmpPredicate::Ne, builder, &current_block)
            .into_mlir();
        (status, vec![contract_value], current_block)
    }
}
