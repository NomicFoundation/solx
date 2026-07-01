//!
//! Expression lowering to MLIR SSA values.
//!

pub mod arithmetic;
pub mod array_expression;
pub mod assignment;
pub mod call;
pub mod comparison;
pub mod conditional;
pub mod identifier;
pub mod index_access;
pub mod literal;
pub mod member;
pub mod operator;
pub mod short_circuit;
pub mod storage;
pub mod tuple_expression;
pub mod unary;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::TypeLike;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_utils::DataLocation;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::emit::emit_as::EmitAs;
use crate::ast::emit::emit_expression::EmitExpression;
use crate::ast::emit::emit_for_effect::EmitForEffect;
use crate::ast::emit::emit_values::EmitValues;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionContext<'state, 'context, 'block> {
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

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
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

    /// Emits a `sol.cmp ne 0` producing `i1` from a value.
    ///
    /// Short-circuits when the value is already `i1` (e.g. from `sol.cmp`),
    /// avoiding the redundant `sol.cmp ne, %i1, %zero_i1 : i1` pattern.
    pub fn emit_is_nonzero(
        &self,
        value: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        if AstType::new(value.r#type()).integer_bit_width() == 1 {
            return value;
        }
        let zero = AstValue::constant(0, AstType::new(value.r#type()), self.state, block);
        AstValue::new(value)
            .compare(zero, CmpPredicate::Ne, self.state, block)
            .into_mlir()
    }

    /// Resolves the Solidity type from Slang to an MLIR type.
    ///
    /// Returns `None` when the incoming slang type is `None`. This can happen when calling
    /// `node.get_type()` if the node doesn't have typing information, for example when
    /// there are unresolved references or semantic errors.
    /// Panics on types that `TypeConversion::resolve_slang_type` does not yet handle.
    pub fn resolve_slang_type(&self, slang_type: Option<SlangType>) -> Option<Type<'context>> {
        Some(TypeConversion::resolve_slang_type(
            &slang_type?,
            None,
            self.state,
        ))
    }

    /// Picks the MLIR type of the address yielded by `sol.gep` / `sol.map`.
    ///
    /// Mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule: when the
    /// element is itself a reference type and lives in `Storage` or
    /// `CallData`, the result address IS the element type rather than a
    /// pointer to it.
    pub(crate) fn address_type(
        context: &Context<'context>,
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
            AstType::pointer(context.mlir_context, element_type, base_location).into_mlir()
        }
    }

    /// Whether `candidate` is the single-byte `!sol.byte`, the element type of `bytes` / `string`.
    ///
    /// The dialect exposes no byte-type query and Rust cannot construct `!sol.byte` directly, so it
    /// is matched against the element type of a `!sol.string`.
    pub fn is_byte(&self, candidate: Type<'context>) -> bool {
        let string_type =
            AstType::string(self.state.mlir_context, DataLocation::Memory).into_mlir();
        let byte_type =
            unsafe { Type::from_raw(solx_mlir::ffi::mlirSolGetEltType(string_type.to_raw(), 0)) };
        candidate == byte_type
    }

    /// The byte width of a fixed-width byte type: `N` for `!sol.fixedbytes<N>`, `1` for the single
    /// `!sol.byte`, and `None` for any other type.
    ///
    /// The dialect exposes no width query, so each `bytes1 ..= bytes32` type is reconstructed and
    /// matched by equality.
    pub fn fixed_bytes_or_byte_width(&self, candidate: Type<'context>) -> Option<u32> {
        if self.is_byte(candidate) {
            return Some(1);
        }
        (1..=solx_utils::BYTE_LENGTH_FIELD as u32).find(|&width| {
            candidate == AstType::fixed_bytes(self.state.mlir_context, width).into_mlir()
        })
    }
}

impl<'context: 'block, 'block> EmitExpression<'context, 'block> for Expression {
    type Output = BlockAnd<'context, 'block, Value<'context, 'block>>;

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
                let BlockAnd { mut value, block } = inner.emit_values(context, block);
                BlockAnd {
                    value: value.remove(0),
                    block,
                }
            }
            Expression::TupleExpression(inner) => inner.emit(context, block),
            Expression::ConditionalExpression(inner) => inner.emit(context, block),
            Expression::ArrayExpression(inner) => inner.emit(context, block),
            Expression::MemberAccessExpression(inner) => inner.emit(context, block),
            Expression::IndexAccessExpression(inner) => inner.emit(context, block),
            _ => unreachable!(
                "unsupported expression: {:?}",
                std::mem::discriminant(self)
            ),
        }
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
            Expression::FunctionCallExpression(call) => call.emit_values(context, block).block,
            _ => self.emit(context, block).block,
        }
    }
}

impl<'context: 'block, 'block> EmitValues<'context, 'block> for Expression {
    /// A tuple yields its elements; a call its result list.
    fn emit_values<'state>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Vec<Value<'context, 'block>>> {
        match self {
            Expression::TupleExpression(tuple) => {
                let items = tuple.items();
                let mut values = Vec::with_capacity(items.len());
                let mut block = block;
                for item in items.iter() {
                    let inner = item.expression().expect("slang validates tuple element");
                    let BlockAnd { value, block: next } = inner.emit(context, block);
                    values.push(value);
                    block = next;
                }
                BlockAnd {
                    value: values,
                    block,
                }
            }
            Expression::FunctionCallExpression(call) => call.emit_values(context, block),
            _ => unreachable!("a multi-valued expression is a tuple or call"),
        }
    }
}

impl<'context: 'block, 'block> EmitAs<'context, 'block, Type<'context>> for Expression {
    type Output = Value<'context, 'block>;

    /// Emits this expression coerced to `target_type`.
    ///
    /// A string literal toward a `byte` / `bytesN` target materialises directly in that
    /// representation; every other expression emits then casts.
    fn emit_as<'state>(
        &self,
        target_type: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        if let Expression::StringExpression(string_literal) = self {
            return string_literal.emit_as(target_type, context, block);
        }
        let BlockAnd { value, block } = self.emit(context, block);
        let value = TypeConversion::from_target_type(target_type, context.state).emit(
            value,
            context.state,
            &block,
        );
        BlockAnd { value, block }
    }
}
