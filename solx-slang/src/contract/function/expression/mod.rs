//!
//! Expression lowering to MLIR SSA values, routed to each expression kind's lowering.
//!

pub mod arithmetic;
pub mod array;
pub mod assignment;
pub mod call;
pub mod comparison;
pub mod conditional;
pub mod identifier;
pub mod index_access;
pub mod literal;
pub mod logical;
pub mod member;
pub mod this;
pub mod tuple;
pub mod unary;

use slang_solidity_v2::ast::Expression as SlangExpression;

use solx_mlir::Context as MlirContext;
use solx_mlir::Value;

use crate::scope::FunctionScope;

use self::arithmetic::AdditiveExpression;
use self::arithmetic::BitwiseAndExpression;
use self::arithmetic::BitwiseOrExpression;
use self::arithmetic::BitwiseXorExpression;
use self::arithmetic::ExponentiationExpression;
use self::arithmetic::MultiplicativeExpression;
use self::arithmetic::ShiftExpression;
use self::array::ArrayExpression;
use self::assignment::AssignmentExpression;
use self::call::FunctionCallExpression;
use self::comparison::EqualityExpression;
use self::comparison::InequalityExpression;
use self::conditional::ConditionalExpression;
use self::identifier::Identifier;
use self::index_access::IndexAccessExpression;
use self::literal::DecimalNumberExpression;
use self::literal::FalseKeyword;
use self::literal::HexNumberExpression;
use self::literal::StringExpression;
use self::literal::TrueKeyword;
use self::logical::AndExpression;
use self::logical::OrExpression;
use self::member::MemberAccessExpression;
use self::this::ThisKeyword;
use self::tuple::TupleExpression;
use self::unary::PostfixExpression;
use self::unary::PrefixExpression;

dispatch!(
    Expression(Expression) {
        /// A call in value position takes its one result.
        -> Value |node, scope| {
            DecimalNumberExpression,
            HexNumberExpression,
            TrueKeyword,
            FalseKeyword,
            StringExpression,
            Identifier,
            ThisKeyword,
            AdditiveExpression,
            MultiplicativeExpression,
            ExponentiationExpression,
            BitwiseAndExpression,
            BitwiseOrExpression,
            BitwiseXorExpression,
            ShiftExpression,
            EqualityExpression,
            InequalityExpression,
            AndExpression,
            OrExpression,
            PrefixExpression,
            PostfixExpression,
            AssignmentExpression,
            ConditionalExpression,
            TupleExpression,
            ArrayExpression,
            MemberAccessExpression,
            IndexAccessExpression,
        } else {
            SlangExpression::FunctionCallExpression(inner) => {
                FunctionCallExpression::emit_values(inner, scope)
                    .into_iter()
                    .next()
                    .expect("a call in value position yields a value")
            }
            SlangExpression::CallOptionsExpression(_) => {
                unimplemented!("call options are not yet supported")
            }
            SlangExpression::NewExpression(_) => {
                unimplemented!("`new` expressions are not yet supported")
            }
            SlangExpression::TypeExpression(_) => {
                unimplemented!("`type(..)` expressions are not yet supported")
            }
            SlangExpression::ElementaryType(_)
            | SlangExpression::PayableKeyword(_)
            | SlangExpression::SuperKeyword(_) => {
                unimplemented!("a type or `super`/`payable` keyword is not a value expression")
            }
        }

        /// A tuple yields its elements, a call its result list.
        -> Values |node, scope| {
            TupleExpression,
            FunctionCallExpression,
        } else {
            _ => unimplemented!("only a tuple or a call yields multiple values"),
        }

        /// Dispatches an assignable expression to the place it resolves to.
        -> Place |node, scope| {
            Identifier,
            MemberAccessExpression,
            IndexAccessExpression,
        } else {
            _ => unimplemented!(
                "expression is not an assignable place: {:?}",
                std::mem::discriminant(node)
            ),
        }

        /// Emits an expression for its side effects, discarding the values.
        pub fn emit_for_effect(node: &SlangExpression, scope: &mut FunctionScope) {
            match node {
                SlangExpression::FunctionCallExpression(call) => {
                    FunctionCallExpression::emit_values(call, scope);
                }
                _ => {
                    Self::emit(node, scope);
                }
            }
        }

        /// The shared `++`/`--` lowering: load the operand's place, apply the stepping operator to
        /// its value and one, store back, returning the value before and after the step.
        pub fn step<'context>(
            operand: &SlangExpression,
            operator: impl FnOnce(
                Value<'context>,
                Value<'context>,
                bool,
                &MlirContext<'context>,
            ) -> Value<'context>,
            scope: &mut FunctionScope<'_, '_, 'context>,
        ) -> (Value<'context>, Value<'context>) {
            let (place, element_type) = Self::emit_place(operand, scope);
            let old = place.load(element_type, scope);
            let one = Value::one(element_type, scope);
            let new = operator(old, one, scope.checked(), scope);
            place.store(new, scope);
            (old, new)
        }
    }
);
