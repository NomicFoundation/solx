//!
//! Assignment expression lowering.
//!

use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::AssignmentExpression;
use slang_solidity_v2::ast::AssignmentExpressionOperator;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::TupleExpression;
use solx_mlir::Builder;

use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::arithmetic::ArithmeticOperation;
use crate::ast::contract::function::expression::bitwise::BitwiseOperation;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::expression::lvalue::Lvalue;

/// The binary operation a compound assignment `x op= y` applies as
/// `x = x op y`.
enum CompoundOperation {
    /// An overflow-checkable arithmetic op (`+=`, `-=`, `*=`, `/=`, `%=`).
    Arithmetic(ArithmeticOperation),
    /// A bitwise or shift op (`&=`, `|=`, `^=`, `<<=`, `>>=`).
    Bitwise(BitwiseOperation),
}

impl CompoundOperation {
    /// The operation a compound-assignment operator applies, or `None` for a
    /// plain `=`.
    fn from_operator(operator: &AssignmentExpressionOperator) -> Option<Self> {
        let operation = match operator {
            AssignmentExpressionOperator::Equal(_) => return None,
            AssignmentExpressionOperator::PlusEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Add)
            }
            AssignmentExpressionOperator::MinusEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Subtract)
            }
            AssignmentExpressionOperator::AsteriskEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Multiply)
            }
            AssignmentExpressionOperator::SlashEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Divide)
            }
            AssignmentExpressionOperator::PercentEqual(_) => {
                Self::Arithmetic(ArithmeticOperation::Remainder)
            }
            AssignmentExpressionOperator::AmpersandEqual(_) => Self::Bitwise(BitwiseOperation::And),
            AssignmentExpressionOperator::BarEqual(_) => Self::Bitwise(BitwiseOperation::Or),
            AssignmentExpressionOperator::CaretEqual(_) => Self::Bitwise(BitwiseOperation::Xor),
            AssignmentExpressionOperator::LessThanLessThanEqual(_) => {
                Self::Bitwise(BitwiseOperation::ShiftLeft)
            }
            AssignmentExpressionOperator::GreaterThanGreaterThanEqual(_)
            | AssignmentExpressionOperator::GreaterThanGreaterThanGreaterThanEqual(_) => {
                Self::Bitwise(BitwiseOperation::ShiftRight)
            }
        };
        Some(operation)
    }

    /// Applies the operation to `(left, right)` through the builder.
    fn emit<'context, 'block>(
        self,
        checked: bool,
        builder: &Builder<'context>,
        left: Value<'context, 'block>,
        right: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match self {
            Self::Arithmetic(operation) => operation.emit(checked, builder, left, right, block),
            Self::Bitwise(operation) => operation.emit(builder, left, right, block),
        }
    }
}

impl<'state, 'context, 'block> ExpressionEmitter<'state, 'context, 'block> {
    /// Lowers an assignment `x = y` or a compound assignment `x op= y`.
    ///
    /// The stored value — the coerced right-hand side, or `x op y` for a
    /// compound operator — is both written to the target and returned as the
    /// expression's result.
    pub fn emit_assignment(
        &self,
        assignment: &AssignmentExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // `(a, b, ...) = rhs` — tuple / destructuring assignment. Only the plain
        // `=` operator is valid on a tuple left-hand side; a single-element
        // tuple is a parenthesised scalar and resolves below.
        if let Expression::TupleExpression(tuple) = &assignment.left_operand()
            && tuple.items().len() > 1
            && matches!(
                assignment.operator(),
                AssignmentExpressionOperator::Equal(_)
            )
        {
            return self.emit_tuple_assignment(tuple, &assignment.right_operand(), block);
        }

        let (lvalue, block) = self.resolve_lvalue(&assignment.left_operand(), block)?;
        let element_type = lvalue.element_type();
        let (value, block) = match CompoundOperation::from_operator(&assignment.operator()) {
            None => self.emit_value(&assignment.right_operand(), block)?,
            Some(operation) => {
                self.emit_compound_value(&lvalue, operation, &assignment.right_operand(), block)?
            }
        };
        let builder = &self.state.builder;
        let stored =
            TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block);
        self.emit_lvalue_store(&lvalue, stored, &block);
        Ok((stored, block))
    }

    /// Computes `x op y` for a compound assignment: loads the target's current
    /// value, evaluates the right-hand side coerced to the target type, and
    /// applies the operation.
    fn emit_compound_value(
        &self,
        lvalue: &Lvalue<'context, 'block>,
        operation: CompoundOperation,
        right_operand: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let element_type = lvalue.element_type();
        let old = self.emit_lvalue_load(lvalue, &block)?;
        let (right, block) = self.emit_value(right_operand, block)?;
        let builder = &self.state.builder;
        let right =
            TypeConversion::from_target_type(element_type, builder).emit(right, builder, &block);
        let value = operation.emit(self.checked, builder, old, right, &block);
        Ok((value, block))
    }

    /// Emits a tuple / destructuring assignment `(a, b, ...) = rhs`.
    ///
    /// Solidity evaluates the entire right-hand side before any assignment
    /// (so `(a, b) = (b, a)` swaps value types), so every RHS value is
    /// materialised first; the LHS lvalues are then resolved left-to-right
    /// against the pre-assignment state and stored right-to-left. A blank
    /// component (`(, b) = ...`) discards its value (still evaluated for any
    /// side effects).
    fn emit_tuple_assignment(
        &self,
        tuple: &TupleExpression,
        right: &Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        // Materialise `(lvalue-expression, value)` pairs, evaluating every value
        // before any store.
        let (assignments, mut block): (Vec<(Expression, Value<'context, 'block>)>, _) = match right
        {
            Expression::TupleExpression(rhs_tuple) => {
                // Recurse only where BOTH sides nest, so a blank slot opposite a
                // nested RHS tuple (`(a, ) = (4, (8, 16))`) discards it whole.
                let pairs = Self::pair_tuple_assignment(tuple, rhs_tuple);
                let mut assignments = Vec::new();
                let mut current = block;
                for (lvalue, rhs_expression) in pairs {
                    match lvalue {
                        Some(lvalue) => {
                            let (value, next) = self.emit_value(&rhs_expression, current)?;
                            current = next;
                            assignments.push((lvalue, value));
                        }
                        // A discarded scalar is still evaluated for side effects;
                        // a discarded nested tuple is dropped wholesale.
                        None if !matches!(rhs_expression, Expression::TupleExpression(_)) => {
                            let (_discarded, next) = self.emit_value(&rhs_expression, current)?;
                            current = next;
                        }
                        None => {}
                    }
                }
                (assignments, current)
            }
            // A call yields a flat value list, so the LHS leaves are flattened
            // (no syntactic nesting can match a flat result) and zipped by leaf.
            Expression::FunctionCallExpression(call) => {
                let lhs_leaves = Self::flatten_tuple_lvalues(tuple);
                let (values, current) =
                    CallEmitter::new(self).emit_function_call_results(call, block)?;
                assert!(
                    values.len() == lhs_leaves.len(),
                    "tuple assignment arity mismatch: {} LHS slots vs {} call results",
                    lhs_leaves.len(),
                    values.len(),
                );
                (Self::zip_assignments(lhs_leaves, values), current)
            }
            _ => unimplemented!(
                "tuple assignment right-hand side: {:?}",
                std::mem::discriminant(right)
            ),
        };

        // Resolve every LHS lvalue address first, left-to-right, so an index or
        // length is read against the pre-assignment state.
        let mut targets = Vec::with_capacity(assignments.len());
        for (lvalue, value) in assignments {
            let (target, next) = self.resolve_lvalue(&lvalue, block)?;
            block = next;
            targets.push((target, value));
        }

        // Store the components right-to-left. For value-typed slots the values
        // are already materialised so the order is irrelevant; for storage
        // aggregates it reproduces Solidity's documented quirk that a storage
        // `(x, y) = (y, x)` swap does not work.
        let mut last_stored = None;
        for (target, value) in targets.into_iter().rev() {
            let element_type = target.element_type();
            let stored = {
                let builder = &self.state.builder;
                TypeConversion::from_target_type(element_type, builder).emit(value, builder, &block)
            };
            self.emit_lvalue_store(&target, stored, &block);
            last_stored = Some(stored);
        }

        // A tuple-assignment expression's value is rarely consumed; yield the
        // last stored value, or a `0` sentinel if every slot was blank.
        let result = last_stored.unwrap_or_else(|| {
            self.state
                .builder
                .emit_sol_constant(0, self.state.builder.types.ui256, &block)
        });
        Ok((result, block))
    }

    /// Pairs a tuple LHS with a tuple RHS into `(lvalue, value-expression)`
    /// pairs, recursing into nested tuples only where both sides nest. A blank
    /// LHS slot yields `None` for its lvalue.
    fn pair_tuple_assignment(
        lhs: &TupleExpression,
        rhs: &TupleExpression,
    ) -> Vec<(Option<Expression>, Expression)> {
        let lhs_items = lhs.items();
        let rhs_items = rhs.items();
        assert!(
            lhs_items.len() == rhs_items.len(),
            "tuple assignment arity mismatch: {} LHS slots vs {} RHS values",
            lhs_items.len(),
            rhs_items.len(),
        );
        let mut pairs = Vec::new();
        for (lhs_item, rhs_item) in lhs_items.iter().zip(rhs_items.iter()) {
            let lhs_expression = lhs_item.expression();
            let rhs_expression = rhs_item
                .expression()
                .expect("empty tuple element on the RHS of an assignment");
            match (&lhs_expression, &rhs_expression) {
                (
                    Some(Expression::TupleExpression(lhs_nested)),
                    Expression::TupleExpression(rhs_nested),
                ) => {
                    pairs.extend(Self::pair_tuple_assignment(lhs_nested, rhs_nested));
                }
                _ => pairs.push((lhs_expression, rhs_expression)),
            }
        }
        pairs
    }

    /// Flattens a tuple LHS into its leaves, recursing into nested tuples
    /// (`(a, (b, c))` → `[a, b, c]`). A blank slot is `None`. Used for a call
    /// right-hand side, whose values are already flat.
    fn flatten_tuple_lvalues(tuple: &TupleExpression) -> Vec<Option<Expression>> {
        let mut leaves = Vec::new();
        for item in tuple.items().iter() {
            match item.expression() {
                Some(Expression::TupleExpression(nested)) => {
                    leaves.extend(Self::flatten_tuple_lvalues(&nested));
                }
                other => leaves.push(other),
            }
        }
        leaves
    }

    /// Zips flattened LHS leaves with their values, dropping blank slots.
    fn zip_assignments(
        lhs_leaves: Vec<Option<Expression>>,
        values: Vec<Value<'context, 'block>>,
    ) -> Vec<(Expression, Value<'context, 'block>)> {
        lhs_leaves
            .into_iter()
            .zip(values)
            .filter_map(|(lvalue, value)| lvalue.map(|lvalue| (lvalue, value)))
            .collect()
    }
}
