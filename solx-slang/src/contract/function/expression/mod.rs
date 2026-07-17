//!
//! Expression lowering to MLIR SSA values, routed to each expression kind's lowering.
//!

pub mod arithmetic;
pub mod array;
pub mod assignment;
pub mod bitwise;
pub mod call;
pub mod comparison;
pub mod conditional;
pub mod identifier;
pub mod index_access;
pub mod keyword;
pub mod literal;
pub mod logical;
pub mod member;
pub mod tuple;
pub mod unary;

use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::Type;

use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

use self::call::Call;

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// Lowers an expression to its single MLIR value, routing each kind to its lowering. A call in
    /// value position takes its one result.
    pub fn expression(&mut self, node: &Expression) -> Value<'context> {
        match node {
            Expression::DecimalNumberExpression(_) | Expression::HexNumberExpression(_) => {
                self.number_literal(node)
            }
            Expression::TrueKeyword(_) => self.boolean_literal(true),
            Expression::FalseKeyword(_) => self.boolean_literal(false),
            Expression::StringExpression(inner) => self.string_literal(inner),
            Expression::Identifier(inner) => self.identifier(inner),
            Expression::ThisKeyword(_) => self.this_value(),
            Expression::AdditiveExpression(inner) => self.additive(inner),
            Expression::MultiplicativeExpression(inner) => self.multiplicative(inner),
            Expression::ExponentiationExpression(inner) => self.exponentiation(inner),
            Expression::BitwiseAndExpression(inner) => self.bitwise_and(inner),
            Expression::BitwiseOrExpression(inner) => self.bitwise_or(inner),
            Expression::BitwiseXorExpression(inner) => self.bitwise_xor(inner),
            Expression::ShiftExpression(inner) => self.shift(inner),
            Expression::EqualityExpression(inner) => self.equality(inner),
            Expression::InequalityExpression(inner) => self.inequality(inner),
            Expression::AndExpression(inner) => self.and(inner),
            Expression::OrExpression(inner) => self.or(inner),
            Expression::PrefixExpression(inner) => self
                .prefix(inner)
                .expect("a prefix expression in value position yields a value"),
            Expression::PostfixExpression(inner) => self.postfix(inner),
            Expression::AssignmentExpression(inner) => self
                .assignment(inner)
                .expect("an assignment in value position yields its assigned value"),
            Expression::ConditionalExpression(inner) => self.conditional(inner),
            Expression::TupleExpression(inner) => self.tuple(inner),
            Expression::ArrayExpression(inner) => self.array(inner),
            Expression::MemberAccessExpression(inner) => self.member_access(inner),
            Expression::IndexAccessExpression(inner) => self.index_access(inner),
            Expression::FunctionCallExpression(inner) => Call::emit(inner, self)
                .into_iter()
                .next()
                .expect("a call in value position yields a value"),
            Expression::CallOptionsExpression(_) => {
                unimplemented!("call options are not yet supported")
            }
            Expression::NewExpression(_) => {
                unimplemented!("`new` expressions are not yet supported")
            }
            Expression::TypeExpression(_) => {
                unimplemented!("`type(..)` expressions are not yet supported")
            }
            Expression::ElementaryType(_)
            | Expression::PayableKeyword(_)
            | Expression::SuperKeyword(_) => {
                unimplemented!("a type or `super`/`payable` keyword is not a value expression")
            }
        }
    }

    /// Lowers an expression that occupies a multi-value position: a tuple yields its elements, a
    /// call its result list, and any other expression its single value.
    pub fn expression_values(&mut self, node: &Expression) -> Vec<Value<'context>> {
        match node {
            Expression::TupleExpression(inner) => self.tuple_values(inner),
            Expression::FunctionCallExpression(inner) => Call::emit(inner, self),
            Expression::ConditionalExpression(inner) => self.conditional_values(inner),
            _ => vec![self.expression(node)],
        }
    }

    /// Resolves an assignable expression, its parentheses peeled, to the place it denotes together
    /// with its element MLIR type, serving both the read path and the assignment lvalue path. A
    /// parenthesized place is a single-element tuple, peeled here; a multi-element tuple denotes
    /// several places, resolved by `expression_places`.
    pub fn expression_place(&mut self, node: &Expression) -> (Place<'context>, MlirType<'context>) {
        match node {
            Expression::Identifier(inner) => self.identifier_place(inner),
            Expression::MemberAccessExpression(inner) => self.member_access_place(inner),
            Expression::IndexAccessExpression(inner) => self.index_access_place(inner),
            Expression::TupleExpression(inner) if inner.items().len() == 1 => {
                let operand = inner
                    .items()
                    .iter()
                    .next()
                    .and_then(|item| item.expression())
                    .expect("a parenthesized place wraps a single operand");
                self.expression_place(&operand)
            }
            _ => unimplemented!(
                "expression is not an assignable place: {:?}",
                std::mem::discriminant(node)
            ),
        }
    }

    /// Resolves an expression in multi-place position: a tuple yields its elements' places, nested
    /// and parenthesized tuples flattening in and a blank element denoting none; any other
    /// expression its single place. The place-side sibling of `expression_values`.
    pub fn expression_places(
        &mut self,
        node: &Expression,
    ) -> Vec<Option<(Place<'context>, MlirType<'context>)>> {
        match node {
            Expression::TupleExpression(inner) => self.tuple_places(inner),
            _ => vec![Some(self.expression_place(node))],
        }
    }

    /// Emits an expression for its side effects, discarding the values.
    pub fn expression_effect(&mut self, node: &Expression) {
        match node {
            Expression::FunctionCallExpression(call) => {
                Call::emit(call, self);
            }
            Expression::PrefixExpression(inner) => {
                self.prefix(inner);
            }
            Expression::AssignmentExpression(inner) => {
                self.assignment(inner);
            }
            Expression::ConditionalExpression(inner) => {
                self.conditional_effect(inner);
            }
            Expression::TupleExpression(inner) => self.tuple_effect(inner),
            _ => {
                self.expression(node);
            }
        }
    }

    /// Emits an expression and coerces its value to `target_type`. A string literal reaching a
    /// bytes-like target folds to that constant directly, since a materialized `sol.string_lit`
    /// value cannot be reinterpreted.
    pub fn coerced(
        &mut self,
        expression: &Expression,
        target_type: MlirType<'context>,
    ) -> Value<'context> {
        if let Expression::StringExpression(literal) = expression
            && target_type.is_bytes_like()
        {
            return Value::left_aligned_bytes(literal.value(), target_type, self);
        }
        self.expression(expression).coerce(target_type, self)
    }

    /// Emits an expression and converts its value to `target_type` through an explicit `T(x)` cast.
    pub fn converted(
        &mut self,
        expression: &Expression,
        target_type: MlirType<'context>,
    ) -> Value<'context> {
        if let Expression::StringExpression(_) = expression {
            return self.coerced(expression, target_type);
        }
        self.expression(expression).convert(target_type, self)
    }

    /// Evaluates both operands of a binary expression and coerces them to the binder's result type.
    pub fn coerced_operands(
        &mut self,
        slang_type: Option<Type>,
        left: &Expression,
        right: &Expression,
    ) -> (Value<'context>, Value<'context>) {
        let result_type = self.typing(slang_type);
        (
            self.coerced(left, result_type),
            self.coerced(right, result_type),
        )
    }

    /// Lowers a multi-value expression to values coerced to `targets`, folding a string-literal tuple
    /// element to its bytes-like constant the way `coerced` does for a scalar. A tuple coerces each
    /// element expression; a lone target folds the whole expression; any other expression
    /// materialises its results and coerces them. A `None` target passes its value through, for a
    /// blank destructuring slot.
    pub fn coerced_values(
        &mut self,
        node: &Expression,
        targets: &[Option<MlirType<'context>>],
    ) -> Vec<Value<'context>> {
        match node {
            Expression::TupleExpression(tuple) => tuple
                .items()
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let element = item.expression().expect("slang validates tuple elements");
                    match targets[index] {
                        Some(target) => self.coerced(&element, target),
                        None => self.expression(&element),
                    }
                })
                .collect(),
            _ => match targets {
                [Some(target)] => vec![self.coerced(node, *target)],
                _ => self
                    .expression_values(node)
                    .into_iter()
                    .enumerate()
                    .map(|(index, value)| match targets[index] {
                        Some(target) => value.coerce(target, self),
                        None => value,
                    })
                    .collect(),
            },
        }
    }
}
