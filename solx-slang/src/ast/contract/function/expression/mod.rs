//!
//! Expression lowering to MLIR SSA values.
//!

pub mod access;
pub mod arithmetic;
pub mod assignment;
pub mod call;
pub mod logical;
pub mod member;
pub mod operator;
pub mod storage;

use std::collections::HashMap;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use slang_solidity_v2::ast::ArrayExpression;
use slang_solidity_v2::ast::ConditionalExpression;
use slang_solidity_v2::ast::DecimalNumberExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FalseKeyword;
use slang_solidity_v2::ast::HexNumberExpression;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StringExpression;
use slang_solidity_v2::ast::ThisKeyword;
use slang_solidity_v2::ast::TrueKeyword;
use slang_solidity_v2::ast::TupleExpression;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Pointer;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;
use solx_mlir::ods::sol::IfOperation;
use solx_mlir::ods::sol::ThisOperation;
use solx_mlir::ods::sol::YieldOperation;
use solx_utils::DataLocation;

use crate::ast::block_and::BlockAnd;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;
use crate::ast::contract::function::storage_slot::StorageSlot;
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
    fn emit_as<'state>(
        &self,
        target_type: Type<'context>,
        context: &ExpressionContext<'state, 'context, 'block>,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, Value<'context, 'block>> {
        let BlockAnd { value, block } = self.emit(context, block);
        let value = TypeConversion::from_target_type(target_type, context.state).emit(
            value,
            context.state,
            &block,
        );
        BlockAnd { value, block }
    }
}

expression_emit!(DecimalNumberExpression; |node, context, block| {
    let value = node.integer_value().expect(
        "decimal literal must evaluate to an integer after applying any units",
    );
    let result_type = context
        .resolve_slang_type(node.get_type())
        .expect("binder types every decimal literal node");
    let value = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        context.state,
        &block,
    )
    .into_mlir();
    BlockAnd { block, value }
});

expression_emit!(HexNumberExpression; |node, context, block| {
    let value = node
        .integer_value()
        .expect("hex literals always evaluate to integers");
    let result_type = context
        .resolve_slang_type(node.get_type())
        .expect("binder types every hex literal node");
    let value = AstValue::constant_from_bigint(
        &value,
        AstType::new(result_type),
        context.state,
        &block,
    )
    .into_mlir();
    BlockAnd { block, value }
});

expression_emit!(TrueKeyword; |context, block| {
    let value = AstValue::boolean(true, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

expression_emit!(FalseKeyword; |context, block| {
    let value = AstValue::boolean(false, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

expression_emit!(ThisKeyword; |context, block| {
    let contract_type = context
        .state
        .current_contract_type
        .expect("sol.this emitted outside a contract");
    let value = mlir_op!(context.state, &block, ThisOperation.addr(contract_type));
    BlockAnd { block, value }
});

expression_emit!(StringExpression; |node, context, block| {
    let bytes = node.value();
    let text = std::str::from_utf8(&bytes).expect("string literal is valid UTF-8");
    let value = AstValue::string_literal(text, context.state, &block).into_mlir();
    BlockAnd { block, value }
});

expression_emit!(Identifier; |node, context, block| {
    let name = node.name();
    match node.resolve_to_definition() {
        Some(Definition::StateVariable(state_variable)) => {
            let slot = context
                .storage_layout
                .get(&state_variable.node_id())
                .expect("state variable is registered in the storage layout");
            let declared_type = state_variable
                .get_type()
                .expect("binder types every state variable");
            let element_type =
                TypeConversion::resolve_slang_type(&declared_type, None, context.state);
            let address = Pointer::addr_of(
                &slot.name,
                AstType::new(ExpressionContext::address_type(
                    context.state,
                    element_type,
                    DataLocation::Storage,
                    &declared_type,
                )),
                context.state,
                &block,
            );
            let value = address
                .load(AstType::new(element_type), context.state, &block)
                .into_mlir();
            BlockAnd { block, value }
        }
        Some(Definition::Variable(_) | Definition::Parameter(_)) => {
            let (pointer, element_type) = context.environment.variable_with_type(&name);
            let value = Pointer::new(pointer)
                .load(AstType::new(element_type), context.state, &block)
                .into_mlir();
            BlockAnd { block, value }
        }
        Some(Definition::Constant(constant)) => {
            let initializer = constant.value().expect("constant has an initializer");
            initializer.emit(context, block)
        }
        None => unreachable!("slang resolves every identifier reference: {name}"),
        Some(_) => unreachable!("unsupported identifier reference: {name}"),
    }
});

expression_emit!(TupleExpression; |node, context, block| {
    let items = node.items();
    let item = items.iter().next().expect("slang validates non-empty tuple");
    let inner = item.expression().expect("tuple element is non-empty");
    inner.emit(context, block)
});

expression_emit!(ConditionalExpression; |node, context, block| {
    let result_type = context.resolve_slang_type(node.get_type()).unwrap_or_else(|| {
        AstType::unsigned(context.state.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
    });
    let condition = node.operand();
    let BlockAnd {
        value: condition_value,
        block,
    } = condition.emit(context, block);
    let condition_boolean = context.emit_is_nonzero(condition_value, &block);

    let result_slot = Pointer::stack(AstType::new(result_type), context.state, &block);
    let (then_block, else_block) = mlir_region_op!(
        context.state, &block,
        IfOperation.cond(condition_boolean); then_region, else_region
    );

    let true_expression = node.true_expression();
    let BlockAnd {
        value: then_value,
        block: then_end,
    } = true_expression.emit(context, then_block);
    let then_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(then_value, context.state, &then_end);
    result_slot.store(AstValue::new(then_cast), context.state, &then_end);
    mlir_op_void!(context.state, &then_end, YieldOperation.ins(&[]));

    let false_expression = node.false_expression();
    let BlockAnd {
        value: else_value,
        block: else_end,
    } = false_expression.emit(context, else_block);
    let else_cast = TypeConversion::from_target_type(result_type, context.state)
        .emit(else_value, context.state, &else_end);
    result_slot.store(AstValue::new(else_cast), context.state, &else_end);
    mlir_op_void!(context.state, &else_end, YieldOperation.ins(&[]));

    let value = result_slot
        .load(AstType::new(result_type), context.state, &block)
        .into_mlir();
    BlockAnd { block, value }
});

expression_emit!(ArrayExpression; |node, context, block| {
    let result_slang_type = node
        .get_type()
        .expect("slang types every array literal");
    let element_slang_type = match &result_slang_type {
        SlangType::FixedSizeArray(fixed_array_type) => fixed_array_type.element_type(),
        SlangType::Array(array_type) => array_type.element_type(),
        _ => unreachable!(
            "array literal has unexpected result type: {:?}",
            std::mem::discriminant(&result_slang_type)
        ),
    };
    let array_type =
        TypeConversion::resolve_slang_type(&result_slang_type, None, context.state);
    let element_type =
        TypeConversion::resolve_slang_type(&element_slang_type, None, context.state);
    let mut element_values = Vec::new();
    let mut current = block;
    for item in node.items().iter() {
        let BlockAnd { value, block: next } = item.emit(context, current);
        let cast_value = TypeConversion::from_target_type(element_type, context.state)
            .emit(value, context.state, &next);
        element_values.push(cast_value);
        current = next;
    }
    let value = AstValue::array_literal(
        &element_values,
        AstType::new(array_type),
        context.state,
        &current,
    )
    .into_mlir();
    BlockAnd {
        block: current,
        value,
    }
});
