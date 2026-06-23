//!
//! Expression emission to MLIR SSA values.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod arithmetic;
pub mod arithmetic_mode;
pub mod assignment;
pub mod call;
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
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::BlockAnd;
use crate::ast::EmitAs;
use crate::ast::EmitExpression;
use crate::ast::EmitForEffect;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Arithmetic overflow-checking mode (Checked by default, Unchecked inside `unchecked {}` and for-loop steps).
    pub arithmetic_mode: ArithmeticMode,
}

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        arithmetic_mode: ArithmeticMode,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            arithmetic_mode,
        }
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for Expression {
    type Output = BlockAnd<'context, 'block, AstValue<'context, 'block>>;

    /// Dispatches an expression to its variant's emission.
    fn emit<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> Self::Output {
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
            Expression::CallOptionsExpression(_) => {
                unimplemented!("call options are not yet supported")
            }
            Expression::NewExpression(_)
            | Expression::TypeExpression(_)
            | Expression::ElementaryType(_)
            | Expression::PayableKeyword(_)
            | Expression::SuperKeyword(_) => {
                unimplemented!("expression emission: bare type/keyword")
            }
        }
    }
}

impl<'context: 'block, 'block> EmitAs<'context, 'block, Type<'context>> for Expression {
    type Output = AstValue<'context, 'block>;

    /// Emits this expression then casts it to `target_type`.
    fn emit_as<'state>(
        &self,
        target_type: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, AstValue<'context, 'block>> {
        let BlockAnd { value, block } = self.emit(context, block);
        let value = value.cast(AstType::new(target_type), &context.state.builder, &block);
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
            _ => self.emit(context, block).block,
        }
    }
}
