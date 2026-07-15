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
            Expression::PrefixExpression(inner) => self.prefix(inner),
            Expression::PostfixExpression(inner) => self.postfix(inner),
            Expression::AssignmentExpression(inner) => self.assignment(inner),
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
            _ => vec![self.expression(node)],
        }
    }

    /// Resolves an assignable expression to the place it denotes together with its element MLIR
    /// type, serving both the read path and the assignment lvalue path.
    pub fn expression_place(&mut self, node: &Expression) -> (Place<'context>, MlirType<'context>) {
        match node {
            Expression::Identifier(inner) => self.identifier_place(inner),
            Expression::MemberAccessExpression(inner) => self.member_access_place(inner),
            Expression::IndexAccessExpression(inner) => self.index_access_place(inner),
            _ => unimplemented!(
                "expression is not an assignable place: {:?}",
                std::mem::discriminant(node)
            ),
        }
    }

    /// Emits an expression for its side effects, discarding the values.
    pub fn expression_effect(&mut self, node: &Expression) {
        match node {
            Expression::FunctionCallExpression(call) => {
                Call::emit(call, self);
            }
            _ => {
                self.expression(node);
            }
        }
    }

    /// Emits an expression and coerces its value to `target_type`.
    pub fn coerced(
        &mut self,
        expression: &Expression,
        target_type: MlirType<'context>,
    ) -> Value<'context> {
        self.expression(expression).coerce(target_type, self)
    }

    /// Emits an expression and converts its value to `target_type` through an explicit `T(x)` cast,
    /// the explicit sibling of `coerced`.
    pub fn converted(
        &mut self,
        expression: &Expression,
        target_type: MlirType<'context>,
    ) -> Value<'context> {
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
}
