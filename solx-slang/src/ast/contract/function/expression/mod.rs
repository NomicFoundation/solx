//!
//! Expression emission to MLIR SSA values.
//!

use crate::ast::Pointer;
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

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;
use slang_solidity_v2::ast;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableMutability;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::AddrOfOperation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Materialize;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;

/// Lowers Solidity expressions to MLIR SSA values.
pub struct ExpressionContext<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment.
    environment: &'state Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// Arithmetic overflow-checking mode for binary operations.
    ///
    /// [`ArithmeticMode::Checked`] by default (Solidity 0.8+);
    /// [`ArithmeticMode::Unchecked`] inside `unchecked {}` blocks and for-loop
    /// step expressions.
    arithmetic_mode: ArithmeticMode,
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

    /// Emits an internal function used as a value (`g` in `f = g;`) as a
    /// `sol.func_constant` producing an `!sol.func_ref<…>` pointer.
    ///
    /// The target is routed through the virtual redirect exactly as a direct
    /// call is, so a base-body `f = g` binds the most-derived override of `g`
    /// — the lexical base version is shadowed and thus unregistered when the
    /// derived contract is compiled.
    fn emit_internal_function_pointer(
        &self,
        function_definition: &FunctionDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let node_id = function_definition.node_id();
        let target_id = self
            .state
            .virtual_redirect
            .get(&node_id)
            .copied()
            .unwrap_or(node_id);
        self.emit_function_constant(target_id, block)
    }

    /// Emits a `sol.func_constant` for the already-resolved internal function
    /// `target_id`, producing its `!sol.func_ref<…>` pointer. The literal target
    /// lowers as-is (no virtual redirect); a caller wanting the most-derived
    /// override resolves the redirect first (see
    /// [`Self::emit_internal_function_pointer`]).
    pub fn emit_function_constant(
        &self,
        target_id: NodeId,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let function = self.state.resolve_function(target_id);
        let func_ref_type = AstType::func_ref(
            self.state.builder.context,
            &function.parameter_types,
            &function.return_types,
        );
        let value = AstValue::function_constant(
            &function.mlir_name,
            func_ref_type,
            &self.state.builder,
            &block,
        )
        .into_mlir();
        (value, block)
    }

    /// If `expression` is a bare function name — always an *internal* function
    /// pointer — returns its `!sol.func_ref` type, built from the function's
    /// declared signature. slang types such a reference from the function's
    /// visibility (a `Public` function resolves to its return type, not the
    /// pointer type), so a caller inferring a result type from the expression —
    /// e.g. a ternary whose branches are function names — uses this to recover
    /// the authoritative internal-pointer type the branch values carry. Returns
    /// `None` for any expression that is not a bare reference to a function.
    fn bare_function_ref_type(&self, expression: &Expression) -> Option<Type<'context>> {
        let Expression::Identifier(identifier) = expression else {
            return None;
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            return None;
        };
        let function = self.state.resolve_function(function_definition.node_id());
        Some(
            AstType::func_ref(
                self.state.builder.context,
                &function.parameter_types,
                &function.return_types,
            )
            .into_mlir(),
        )
    }

    /// Reads a contract state variable's value: a `constant` inlines its
    /// compile-time initializer (exactly as a file-level `constant`), otherwise
    /// the storage slot is loaded. A value-typed slot reads through the shared
    /// storage-load path; a reference-typed one evaluates to its storage
    /// reference, whose address type is the reference itself (the single
    /// `address_type` rule). Shared by a bare identifier reference and a
    /// namespace-qualified `C.stateVar` / `L.CONST` access (the latter
    /// disambiguating from a shadowing local).
    fn emit_state_variable_read(
        &self,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let declared_type = state_variable.get_type().expect("slang validated");
        let element_type = AstType::resolve(
            &declared_type,
            LocationPolicy::Declared(None),
            &self.state.builder,
        );
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            let initializer = state_variable.value().expect("slang validated");
            // Emit toward the declared type so a `bytesN constant` initialised
            // from a string literal folds to a fixed-bytes constant.
            let BlockAnd { value, block } =
                if let Expression::StringExpression(string_literal) = &initializer {
                    string_literal.materialize(element_type, self, block)
                } else {
                    initializer.emit(self, block)
                };
            return (value.into_mlir(), block);
        }
        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            });
        let value = if declared_type.is_reference_type() {
            let address_type = AstType::new(element_type)
                .address_type(slot.location, self.state.builder.context)
                .into_mlir();
            let address = sol_op!(
                &self.state.builder,
                &block,
                AddrOfOperation
                    .var(FlatSymbolRefAttribute::new(
                        self.state.builder.context,
                        &slot.name,
                    ))
                    .addr(address_type)
            );
            Pointer::new(address)
                .load(AstType::new(element_type), &self.state.builder, &block)
                .into_mlir()
        } else {
            slot.load(&self.state.builder, element_type, &block)
        };
        (value, block)
    }
}

impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope> for Expression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = BlockAnd<'context, 'block, AstValue<'context, 'block>>;

    /// Dispatches an expression to its variant's emission, first folding a
    /// compile-time-constant arithmetic/bitwise expression straight to a constant:
    /// slang assigns it a `Literal` type carrying the exact computed value, which
    /// matches solc's exact rational arithmetic (`1/2*2 == 1`, `2**256-1` without
    /// 256-bit wraparound) and is the only way to lower a rational intermediate,
    /// which has no runtime type.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        // A COMPUTED constant expression (arithmetic / bitwise / shift / prefix)
        // folds to its exact integer — slang records the value on its `Literal`
        // type — and is emitted as that constant directly. A bare literal is
        // excluded so it keeps its own emit arm.
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
            let result_type = AstType::resolve_optional(self.get_type(), &context.state.builder)
                .expect("slang validated");
            let value = AstValue::constant_from_bigint(
                &folded,
                AstType::new(result_type),
                &context.state.builder,
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
                let (mut values, block) = inner.emit(context, block);
                BlockAnd {
                    value: AstValue::from(values.remove(0)),
                    block,
                }
            }
            Expression::TupleExpression(inner) => inner.emit(context, block),
            Expression::ConditionalExpression(inner) => {
                let (mut values, block) = inner.emit(context, block);
                BlockAnd {
                    value: AstValue::from(values.remove(0)),
                    block,
                }
            }
            Expression::ArrayExpression(inner) => inner.emit(context, block),
            Expression::MemberAccessExpression(inner) => inner.emit(context, block),
            Expression::IndexAccessExpression(inner) => inner.emit(context, block),
            Expression::CallOptionsExpression(inner) => inner.emit(context, block),
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

impl<'state, 'context, 'block, 'scope> Materialize<'context, 'block, 'state, 'scope, Type<'context>>
    for Expression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = AstValue<'context, 'block>;

    /// Emits this expression coerced to `target_type`: a string literal
    /// materialises in the target representation (a `bytesN` / `byte` constant),
    /// every other expression emits naturally; the result then casts to the target
    /// — a no-op when the literal already materialised at it.
    fn materialize(
        &self,
        target_type: Type<'context>,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> BlockAnd<'context, 'block, AstValue<'context, 'block>> {
        let BlockAnd { value, block } = match self {
            Expression::StringExpression(string_literal) => {
                string_literal.materialize(target_type, context, block)
            }
            _ => self.emit(context, block),
        };
        let value = value.cast(AstType::new(target_type), &context.state.builder, &block);
        BlockAnd { value, block }
    }
}
