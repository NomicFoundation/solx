//!
//! Expression lowering to MLIR SSA values.
//!

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
pub mod new;
pub mod operator;
pub mod short_circuit;
pub mod storage;
pub mod unary;

pub use self::literal::Toward;

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
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Builder;
use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::ods::sol::AddrOfOperation;
use solx_mlir::ods::sol::FuncConstantOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::expression_ext::ExpressionExt;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::ResolveType;
use crate::ast::type_conversion::TypeConversion;

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
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
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
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (mlir_name, parameter_types, return_types) = self.state.resolve_function(target_id)?;
        let func_ref_type =
            crate::ast::Type::func_ref(self.state.builder.context, parameter_types, return_types)
                .into_mlir();
        let mlir_name = mlir_name.to_owned();
        let value = sol_op!(
            &self.state.builder,
            &block,
            FuncConstantOperation
                .addr(func_ref_type)
                .sym(FlatSymbolRefAttribute::new(
                    self.state.builder.context,
                    &mlir_name,
                ))
        );
        Ok((value, block))
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
        let (_, parameter_types, return_types) = self
            .state
            .resolve_function(function_definition.node_id())
            .ok()?;
        Some(
            crate::ast::Type::func_ref(self.state.builder.context, parameter_types, return_types)
                .into_mlir(),
        )
    }

    /// Reads a contract state variable's value: a `constant` inlines its
    /// compile-time initializer (exactly as a file-level `constant`), otherwise
    /// the storage slot is loaded. A value-typed slot reads through the shared
    /// storage-load helper; a reference-typed one evaluates to its storage
    /// reference, whose address type is the reference itself (the single
    /// `address_type` rule). Shared by a bare identifier reference and a
    /// namespace-qualified `C.stateVar` / `L.CONST` access (the latter
    /// disambiguating from a shadowing local).
    fn emit_state_variable_read(
        &self,
        state_variable: &ast::StateVariableDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let declared_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        let element_type =
            declared_type.resolve_type(LocationPolicy::Declared(None), &self.state.builder);
        if matches!(
            state_variable.mutability(),
            StateVariableMutability::Constant
        ) {
            let initializer = state_variable
                .value()
                .expect("a constant state variable has an initializer");
            // Emit toward the declared type so a `bytesN constant` initialised
            // from a string literal folds to a fixed-bytes constant.
            let BlockAnd { value, block } = (Toward {
                expression: &initializer,
                target_type: element_type,
            })
            .emit(self, block)?;
            return Ok((value.into_mlir(), block));
        }
        let slot = self
            .storage_layout
            .get(&state_variable.node_id())
            .unwrap_or_else(|| {
                unimplemented!("unregistered state variable {:?}", state_variable.node_id())
            });
        let value = if declared_type.is_reference_type() {
            let address_type = Self::address_type(
                &self.state.builder,
                element_type,
                slot.location,
                &declared_type,
            );
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
            crate::ast::Pointer::new(address)
                .load(
                    crate::ast::Type::new(element_type),
                    &self.state.builder,
                    &block,
                )
                .into_mlir()
        } else {
            slot.load(&self.state.builder, element_type, &block)?
        };
        Ok((value, block))
    }

    /// Picks the MLIR type of the address yielded by `sol.gep` / `sol.map`.
    ///
    /// Mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule: when the
    /// element is itself a reference type and lives in `Storage` or
    /// `CallData`, the result address IS the element type rather than a
    /// pointer to it.
    fn address_type(
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
            crate::ast::Type::pointer(builder.context, element_type, base_location).into_mlir()
        }
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
    type Output = BlockAnd<'context, 'block, crate::ast::Value<'context, 'block>>;

    /// Dispatches an expression to its variant's lowering, first folding a
    /// compile-time-constant arithmetic/bitwise expression straight to a constant:
    /// slang assigns it a `Literal` type carrying the exact computed value, which
    /// matches solc's exact rational arithmetic (`1/2*2 == 1`, `2**256-1` without
    /// 256-bit wraparound) and is the only way to lower a rational intermediate,
    /// which has no runtime type.
    fn emit(
        &self,
        context: Self::Context,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Self::Output> {
        // A compile-time-constant arithmetic/bitwise expression folds to its
        // exact integer; emit that constant directly.
        if let Some(folded) = self.folded_constant_value() {
            let result_type = TypeConversion::resolve_optional_slang_type(
                self.get_type(),
                &context.state.builder,
            )
            .expect("slang types every folded constant expression");
            let value = crate::ast::Value::constant_from_bigint(
                &folded,
                crate::ast::Type::new(result_type),
                &context.state.builder,
                &block,
            );
            return Ok(BlockAnd { block, value });
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
            Expression::FunctionCallExpression(inner) => inner.emit(context, block),
            Expression::TupleExpression(inner) => inner.emit(context, block),
            Expression::ConditionalExpression(inner) => inner.emit(context, block),
            Expression::ArrayExpression(inner) => inner.emit(context, block),
            Expression::MemberAccessExpression(inner) => inner.emit(context, block),
            Expression::IndexAccessExpression(inner) => inner.emit(context, block),
            Expression::CallOptionsExpression(inner) => inner.emit(context, block),
            Expression::NewExpression(_)
            | Expression::TypeExpression(_)
            | Expression::ElementaryType(_)
            | Expression::PayableKeyword(_)
            | Expression::SuperKeyword(_) => {
                unimplemented!("expression lowering: bare type/keyword")
            }
        }
    }
}
