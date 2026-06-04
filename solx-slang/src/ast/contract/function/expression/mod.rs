//!
//! Expression lowering to MLIR SSA values.
//!

/// Index access expression lowering.
pub mod access;
/// Arithmetic expression lowering.
pub mod arithmetic;
/// Assignment expression lowering.
pub mod assignment;
/// Bitwise and shift expression lowering.
pub mod bitwise;
/// Function call and member access lowering.
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
/// Member access expression lowering.
pub mod member;
/// State variable storage access.
pub mod storage;
/// Tuple expression lowering.
pub mod tuple;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_utils::DataLocation;

use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
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
    /// `true` by default (Solidity 0.8+). Set to `false` inside `unchecked {}`
    /// blocks and for-loop step expressions.
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
        // Function calls (and parenthesized calls) may be void; every other
        // expression yields a value, wrapped here once.
        match expression {
            Expression::FunctionCallExpression(call) => {
                call::CallEmitter::new(self).emit_function_call(call, block)
            }
            Expression::TupleExpression(tuple) => self.emit_tuple(tuple, block),
            _ => {
                let (value, block) = self.emit_value_expression(expression, block)?;
                Ok((Some(value), block))
            }
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

    /// Dispatches a value-producing expression to its domain.
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
            Expression::ThisKeyword(_) => Ok((self.emit_this(&block), block)),
            Expression::StringExpression(string) => Ok((self.emit_string(string, &block), block)),
            Expression::ArrayExpression(array) => self.emit_array(array, block),
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

    /// Emits a `sol.cmp ne 0` producing `i1` from a value.
    ///
    /// Short-circuits when the value is already `i1` (e.g. from `sol.cmp`),
    /// avoiding the redundant `sol.cmp ne, %i1, %zero_i1 : i1` pattern.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if solx_mlir::TypeFactory::integer_bit_width(value.r#type()) == 1 {
            return value;
        }
        let zero = self
            .state
            .builder
            .emit_sol_constant(0, value.r#type(), block);
        self.state
            .builder
            .emit_sol_cmp(value, zero, CmpPredicate::Ne, block)
    }

    /// Resolves a Slang type to its MLIR type, propagating `None` when the
    /// binder has no type for the node (unresolved references, semantic errors).
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before lowering.
    pub fn resolve_slang_type(&self, slang_type: Option<SlangType>) -> Option<Type<'context>> {
        Some(TypeConversion::resolve_slang_type(
            &slang_type?,
            None,
            &self.state.builder,
        ))
    }

    /// Picks the MLIR type of the address yielded by `sol.gep` / `sol.map`.
    ///
    /// Mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule: when the
    /// element is itself a reference type and lives in `Storage` or
    /// `CallData`, the result address IS the element type rather than a
    /// pointer to it.
    pub fn address_type(
        builder: &Builder<'context>,
        element_type: Type<'context>,
        base_location: DataLocation,
        result_type: &SlangType,
    ) -> Type<'context> {
        if result_type.is_reference_type()
            && matches!(
                base_location,
                DataLocation::Storage | DataLocation::CallData
            )
        {
            element_type
        } else {
            builder.types.pointer(element_type, base_location)
        }
    }
}
