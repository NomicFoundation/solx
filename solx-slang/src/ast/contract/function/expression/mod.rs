//!
//! Expression emission to MLIR SSA values.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod arithmetic;
pub mod arithmetic_mode;
pub mod assignment;
pub mod call;
pub mod call_options;
pub mod comparison;
pub mod conditional;
pub mod identifier;
pub mod index_access;
pub mod literal;
pub mod logical_operator;
pub mod member;
pub mod operator;
pub mod short_circuit;
pub mod storage;
pub mod unary;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::expression::assignment::AssignmentTarget;
use crate::ast::contract::storage_layout::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// Contract-local dispatch metadata.
    pub dispatch: &'state ContractDispatch,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Arithmetic overflow-checking mode; Checked by default, Unchecked inside `unchecked {}` and for-loop steps.
    pub arithmetic_mode: ArithmeticMode,
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        dispatch: &'state ContractDispatch,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        arithmetic_mode: ArithmeticMode,
    ) -> Self {
        Self {
            state,
            environment,
            dispatch,
            storage_layout,
            arithmetic_mode,
        }
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for Expression {
    type Output = BlockAnd<'context, 'block, AstValue<'context, 'block>>;

    /// Dispatches an expression to its variant's emission, first folding a compile-time-constant
    /// arithmetic/bitwise expression to a constant.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
        let folds = matches!(
            self,
            Expression::AdditiveExpression(_)
                | Expression::MultiplicativeExpression(_)
                | Expression::ExponentiationExpression(_)
                | Expression::ShiftExpression(_)
                | Expression::BitwiseAndExpression(_)
                | Expression::BitwiseOrExpression(_)
                | Expression::BitwiseXorExpression(_)
                | Expression::PrefixExpression(_)
        );
        if folds && let Some(folded) = self.integer_value() {
            let result_type =
                AstType::resolve_optional(self.get_type(), context.state).expect("slang validated");
            let value = AstValue::constant_from_bigint(
                &folded,
                AstType::new(result_type),
                context.state,
                &block,
            );
            return BlockAnd { block, value };
        }
        match self {
            Expression::DecimalNumberExpression(inner) => inner.emit(context, block),
            Expression::HexNumberExpression(inner) => inner.emit(context, block),
            Expression::TrueKeyword(inner) => inner.emit(context, block),
            Expression::FalseKeyword(inner) => inner.emit(context, block),
            Expression::ThisKeyword(inner) => inner.emit(context, block),
            Expression::StringExpression(inner) => inner.emit(context, block),
            Expression::Identifier(inner) => inner.emit(context, block),
            Expression::AssignmentExpression(inner) => inner.emit(context, block),
            Expression::AdditiveExpression(inner) => inner.emit(context, block),
            Expression::MultiplicativeExpression(inner) => inner.emit(context, block),
            Expression::ExponentiationExpression(inner) => inner.emit(context, block),
            Expression::EqualityExpression(inner) => inner.emit(context, block),
            Expression::InequalityExpression(inner) => inner.emit(context, block),
            Expression::AndExpression(inner) => inner.emit(context, block),
            Expression::OrExpression(inner) => inner.emit(context, block),
            Expression::PostfixExpression(inner) => inner.emit(context, block),
            Expression::PrefixExpression(inner) => inner.emit(context, block),
            Expression::BitwiseAndExpression(inner) => inner.emit(context, block),
            Expression::BitwiseOrExpression(inner) => inner.emit(context, block),
            Expression::BitwiseXorExpression(inner) => inner.emit(context, block),
            Expression::ShiftExpression(inner) => inner.emit(context, block),
            Expression::FunctionCallExpression(inner) => {
                let BlockAnd { mut value, block } = inner.emit(context, block);
                BlockAnd {
                    value: AstValue::from(value.remove(0)),
                    block,
                }
            }
            Expression::TupleExpression(inner) => inner.emit(context, block),
            Expression::ConditionalExpression(inner) => {
                let BlockAnd { mut value, block } = inner.emit(context, block);
                BlockAnd {
                    value: AstValue::from(value.remove(0)),
                    block,
                }
            }
            Expression::ArrayExpression(inner) => inner.emit(context, block),
            Expression::MemberAccessExpression(inner) => inner.emit(context, block),
            Expression::IndexAccessExpression(inner) => inner.emit(context, block),
            Expression::CallOptionsExpression(inner) => inner.emit(context, block),
            Expression::NewExpression(_) => {
                unreachable!("a new expression is consumed by its call or discarded for effect")
            }
            Expression::TypeExpression(_)
            | Expression::ElementaryType(_)
            | Expression::PayableKeyword(_)
            | Expression::SuperKeyword(_) => {
                unreachable!(
                    "a type or keyword is not a value; it is consumed by its enclosing conversion, call, or member access"
                )
            }
        }
    }
}

impl<'context: 'block, 'block> EmitAs<'context, 'block, Type<'context>> for Expression {
    type Output = AstValue<'context, 'block>;

    /// Emits this expression coerced to `target_type`.
    fn emit_as<'state>(
        &self,
        target_type: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, AstValue<'context, 'block>> {
        let BlockAnd { value, block } = match self {
            Expression::StringExpression(string_literal) => {
                string_literal.emit_as(target_type, context, block)
            }
            _ => self.emit(context, block),
        };
        let value = value.cast(AstType::new(target_type), context.state, &block);
        BlockAnd { value, block }
    }
}

impl<'context: 'block, 'block> EmitForEffect<'context, 'block> for Expression {
    /// Emits this expression for its side effects, discarding the value.
    fn emit_for_effect<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        match self {
            Expression::FunctionCallExpression(call) => call.emit(context, block).block,
            Expression::PrefixExpression(prefix)
                if matches!(
                    prefix.operator(),
                    ast::PrefixExpressionOperator::DeleteKeyword(_)
                ) =>
            {
                AssignmentTarget::delete(context, &prefix.operand(), block)
            }
            Expression::ConditionalExpression(conditional)
                if matches!(conditional.get_type(), Some(ast::Type::Void(_))) =>
            {
                conditional.emit_for_effect(context, block)
            }
            Expression::NewExpression(_) => block,
            _ => self.emit(context, block).block,
        }
    }
}
