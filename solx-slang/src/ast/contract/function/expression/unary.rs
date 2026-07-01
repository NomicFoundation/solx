//!
//! Unary expression emission: prefix and postfix operators.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::PostfixExpression;
use slang_solidity_v2::ast::PrefixExpression;

use solx_mlir::CmpPredicate;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::NotOperation;
use solx_mlir::ods::sol::SubOperation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::operator::Operator;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_place::EmitPlace;
use crate::ast::place::Place;

expression_emit!(PostfixExpression; |node, context, block| {
    let operator = match node.operator() {
        ast::PostfixExpressionOperator::MinusMinus(_) => Operator::Decrement,
        ast::PostfixExpressionOperator::PlusPlus(_) => Operator::Increment,
    };
    let operand = node.operand().unwrap_parentheses();
    let BlockAnd { value, block } = context.emit_postfix(&operand, operator, block);
    BlockAnd { block, value }
});

expression_emit!(PrefixExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type());
    let operator = match node.operator() {
        ast::PrefixExpressionOperator::Bang(_) => Operator::Not,
        ast::PrefixExpressionOperator::DeleteKeyword(_) => Operator::Delete,
        ast::PrefixExpressionOperator::Minus(_) => Operator::Subtract,
        ast::PrefixExpressionOperator::MinusMinus(_) => Operator::Decrement,
        ast::PrefixExpressionOperator::PlusPlus(_) => Operator::Increment,
        ast::PrefixExpressionOperator::Tilde(_) => Operator::BitwiseNot,
    };
    let operand = node.operand().unwrap_parentheses();
    let BlockAnd { value, block } =
        context.emit_prefix(operator, &operand, result_type, block);
    BlockAnd { block, value }
});

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Emits postfix `++` or `--` (returns the old value).
    pub fn emit_postfix(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        if let Some((old, _new, block)) =
            self.emit_increment_decrement_indexed(operand, operator, block)
        {
            return BlockAnd { block, value: old };
        }
        let (old, _) = self.emit_increment_decrement(operand, operator, &block);
        BlockAnd { block, value: old }
    }

    /// Emits prefix operators: `!`, `-`, `~`, `++`, `--`.
    ///
    /// When `target_type` is `Some`, unary operations use that type (matching
    /// solc's typed MLIR). When `None`, falls back to ui256 semantics.
    pub fn emit_prefix(
        &self,
        operator: Operator,
        operand: &Expression,
        target_type: Option<Type<'context>>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        match operator {
            Operator::Increment | Operator::Decrement => {
                if let Some((_old, new_value, block)) =
                    self.emit_increment_decrement_indexed(operand, operator, block)
                {
                    return BlockAnd {
                        block,
                        value: new_value,
                    };
                }
                let (_old, new_value) = self.emit_increment_decrement(operand, operator, &block);
                BlockAnd {
                    block,
                    value: new_value,
                }
            }
            Operator::BitwiseNot => {
                let BlockAnd { value, block } = operand.emit(self, block);
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let value = block
                    .append_operation(
                        NotOperation::builder(self.state.mlir_context, self.state.location())
                            .value(value)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.not always produces one result")
                    .into();
                BlockAnd { block, value }
            }
            Operator::Not => {
                let BlockAnd { value, block } = operand.emit(self, block);
                let zero =
                    AstValue::constant(0, AstType::new(value.r#type()), self.state, &block)
                        .into_mlir();
                let cmp = AstValue::new(value)
                    .compare(AstValue::new(zero), CmpPredicate::Eq, self.state, &block)
                    .into_mlir();
                let result_type = target_type.unwrap_or(
                    AstType::unsigned(self.state.mlir_context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir(),
                );
                let value = TypeConversion::from_target_type(result_type, self.state)
                    .emit(cmp, self.state, &block);
                BlockAnd { block, value }
            }
            Operator::Subtract => {
                let BlockAnd { value, block } = operand.emit(self, block);
                let operand_type = target_type.unwrap_or_else(|| value.r#type());
                let value = TypeConversion::from_target_type(operand_type, self.state)
                    .emit(value, self.state, &block);
                let zero =
                    AstValue::constant(0, AstType::new(operand_type), self.state, &block)
                        .into_mlir();
                let value = block
                    .append_operation(
                        SubOperation::builder(self.state.mlir_context, self.state.location())
                            .lhs(zero)
                            .rhs(value)
                            .build()
                            .into(),
                    )
                    .result(0)
                    .expect("sol.sub always produces one result")
                    .into();
                BlockAnd { block, value }
            }
            _ => unreachable!("unsupported prefix operator: {operator:?}"),
        }
    }

    /// Loads, increments or decrements, stores, and returns `(old, new, continuation block)` for a
    /// computed lvalue such as `a[i]` or a struct field, addressed via [`EmitPlace`]. Returns `None`
    /// for any other operand, so the caller falls back to the named-lvalue path.
    fn emit_increment_decrement_indexed(
        &self,
        operand: &Expression,
        operator: Operator,
        block: BlockRef<'context, 'block>,
    ) -> Option<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let BlockAnd {
            value:
                Place {
                    address,
                    element_type,
                },
            block,
        } = match operand {
            Expression::IndexAccessExpression(index_access) => {
                index_access.emit_place(self, block)
            }
            Expression::MemberAccessExpression(access) => access.emit_place(self, block),
            _ => return None,
        };
        let pointer = Pointer::new(address);
        let old = pointer
            .load(AstType::new(element_type), self.state, &block)
            .into_mlir();
        let one =
            AstValue::constant(1, AstType::new(element_type), self.state, &block).into_mlir();
        let new_value = block
            .append_operation(operator.emit_sol_binary_operation(
                self.checked,
                self.state.mlir_context,
                self.state.location(),
                old,
                one,
            ))
            .result(0)
            .expect("binary operation always produces one result")
            .into();
        pointer.store(AstValue::new(new_value), self.state, &block);
        Some((old, new_value, block))
    }

    /// Loads, increments or decrements, stores, and returns `(old, new)`.
    ///
    /// Handles both local variables and state variables via
    /// `resolve_to_definition()`.
    fn emit_increment_decrement(
        &self,
        operand: &Expression,
        operator: Operator,
        block: &BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, Value<'context, 'block>) {
        let Expression::Identifier(identifier) = operand else {
            unreachable!("unsupported operand for {operator:?}");
        };
        let name = identifier.name();

        match identifier.resolve_to_definition() {
            Some(Definition::StateVariable(state_variable)) => {
                let slot = self
                    .storage_layout
                    .get(&state_variable.node_id())
                    .expect("state variable is registered in the storage layout");
                let element_type =
                    TypeConversion::resolve_state_variable_type(&state_variable, self.state);
                let old = self.emit_storage_load(slot, element_type, block);
                let one =
                    AstValue::constant(1, AstType::new(element_type), self.state, block)
                        .into_mlir();
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.mlir_context,
                        self.state.location(),
                        old,
                        one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                self.emit_storage_store(slot, new_value, element_type, block);
                (old, new_value)
            }
            Some(Definition::Variable(_) | Definition::Parameter(_)) => {
                let (pointer, element_type) = self.environment.variable_with_type(&name);
                let old = Pointer::new(pointer)
                    .load(AstType::new(element_type), self.state, block)
                    .into_mlir();
                let typed_one =
                    AstValue::constant(1, AstType::new(element_type), self.state, block)
                        .into_mlir();
                let new_value = block
                    .append_operation(operator.emit_sol_binary_operation(
                        self.checked,
                        self.state.mlir_context,
                        self.state.location(),
                        old,
                        typed_one,
                    ))
                    .result(0)
                    .expect("binary operation always produces one result")
                    .into();
                Pointer::new(pointer).store(AstValue::new(new_value), self.state, block);
                (old, new_value)
            }
            None => unreachable!("slang resolves every identifier reference: {name}"),
            Some(_) => unreachable!("unsupported operand for {operator:?}: {name}"),
        }
    }
}
