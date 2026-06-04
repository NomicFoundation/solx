//!
//! Expression lowering to MLIR SSA values.
//!

/// Index access expression lowering.
pub mod access;
/// Binary arithmetic expression lowering.
pub mod arithmetic;
/// Assignment expression lowering.
pub mod assignment;
/// Bitwise and shift expression lowering.
pub mod bitwise;
/// Function and built-in call lowering.
pub mod call;
/// Comparison expression lowering.
pub mod comparison;
/// Conditional (ternary) expression lowering.
pub mod conditional;
/// Identifier expression lowering.
pub mod identifier;
/// Literal expression lowering.
pub mod literal;
/// Short-circuit logical expression lowering.
pub mod logical;
/// Assignable locations (lvalues).
pub mod lvalue;
/// Member access expression lowering.
pub mod member;
/// State variable storage access.
pub mod storage;
/// Tuple expression lowering.
pub mod tuple;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::storage_slot::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'state Context<'context>,
    /// Variable environment.
    pub environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default (Solidity 0.8+); `false` inside `unchecked {}` blocks.
    pub checked: bool,
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Creates a new expression emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        checked: bool,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            checked,
        }
    }

    /// Emits MLIR for an expression, appending operations to `block`.
    ///
    /// Returns `None` for void expressions (calls with no return value); use
    /// [`Self::emit_value`] when a value is required.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression contains unsupported constructs.
    pub fn emit(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // A function call may be void (used as a statement); every other
        // expression yields a value, wrapped here once.
        if let Expression::FunctionCallExpression(call) = expression {
            return call::CallEmitter::new(self).emit_function_call(call, block);
        }
        let (value, block) = self.emit_value_expression(expression, block)?;
        Ok((Some(value), block))
    }

    /// Lowers an expression that always yields a value, dispatching each
    /// expression kind to its domain.
    fn emit_value_expression(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        match expression {
            Expression::DecimalNumberExpression(decimal) => {
                Ok((self.emit_decimal(decimal, &block), block))
            }
            Expression::HexNumberExpression(hex) => Ok((self.emit_hex(hex, &block), block)),
            Expression::TrueKeyword(_) => Ok((self.emit_boolean(true, &block), block)),
            Expression::FalseKeyword(_) => Ok((self.emit_boolean(false, &block), block)),
            Expression::Identifier(identifier) => self.emit_identifier(identifier, block),
            Expression::AdditiveExpression(expression) => self.emit_additive(expression, block),
            Expression::MultiplicativeExpression(expression) => {
                self.emit_multiplicative(expression, block)
            }
            Expression::ExponentiationExpression(expression) => {
                self.emit_exponentiation(expression, block)
            }
            Expression::EqualityExpression(expression) => self.emit_equality(expression, block),
            Expression::InequalityExpression(expression) => self.emit_inequality(expression, block),
            Expression::AssignmentExpression(assignment) => self.emit_assignment(assignment, block),
            Expression::PostfixExpression(expression) => self.emit_postfix(expression, block),
            Expression::PrefixExpression(expression) => self.emit_prefix(expression, block),
            Expression::ConditionalExpression(conditional) => {
                self.emit_conditional(conditional, block)
            }
            Expression::AndExpression(expression) => self.emit_and(
                &expression.left_operand(),
                &expression.right_operand(),
                block,
            ),
            Expression::OrExpression(expression) => self.emit_or(
                &expression.left_operand(),
                &expression.right_operand(),
                block,
            ),
            Expression::BitwiseAndExpression(expression) => {
                self.emit_bitwise_and(expression, block)
            }
            Expression::BitwiseOrExpression(expression) => self.emit_bitwise_or(expression, block),
            Expression::BitwiseXorExpression(expression) => {
                self.emit_bitwise_xor(expression, block)
            }
            Expression::ShiftExpression(expression) => self.emit_shift(expression, block),
            Expression::MemberAccessExpression(access) => self.emit_member_access(access, block),
            Expression::IndexAccessExpression(index_access) => {
                self.emit_index_access(index_access, block)
            }
            _ => unimplemented!(
                "expression lowering: {:?}",
                std::mem::discriminant(expression)
            ),
        }
    }

    /// Emits MLIR for an expression that must produce a value.
    ///
    /// # Errors
    ///
    /// Returns an error if the expression is void or unsupported.
    pub fn emit_value(
        &self,
        expression: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit(expression, block)?;
        let value = value.ok_or_else(|| anyhow::anyhow!("expression produced no value"))?;
        Ok((value, block))
    }

    /// Emits the contract's state-variable initializers into `block`, returning
    /// the continuation block.
    ///
    /// # Errors
    ///
    /// Returns an error if an initializer contains unsupported constructs.
    pub fn emit_state_var_initializers(
        &self,
        contract: &ContractDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        for member in contract.members().iter() {
            if let ContractMember::StateVariableDefinition(variable) = member
                && variable.value().is_some()
            {
                unimplemented!("state variable initializer lowering");
            }
        }
        Ok(block)
    }
}
