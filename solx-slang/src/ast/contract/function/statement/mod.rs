//!
//! Statement lowering to MLIR operations.
//!

pub mod assembly;
pub mod control_flow;
pub mod event;
pub mod revert;
pub mod try_statement;
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::Statements;
use slang_solidity_v2::ast::Type as SlangType;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::ExpressionExt;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::contract::function::expression::arithmetic_mode::ArithmeticMode;
use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::contract::function::modifier_body_call::ModifierBodyCall;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::type_conversion::TypeConversion;

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// The current region for creating new blocks.
    /// Stored as a raw pointer to allow switching between Sol op regions
    /// without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `emit_return` to cast to.
    return_types: &'state [Type<'context>],
    /// The function's return slots, parallel to `return_types` (`None` for an
    /// unnamed return). A bare `return;` and the fall-through epilogue load these
    /// so the `sol.return` arity matches the declared returns.
    return_slots: &'state [Option<Value<'context, 'block>>],
    /// Set while emitting a modifier stage: the hand-off the `_;` placeholder
    /// lowers to (call the wrapped body / next stage, threading the shared return
    /// values). `None` outside a modifier stage.
    modifier_body_call: Option<ModifierBodyCall<'context, 'block>>,
    /// Inline modifier-chain stages for a *constructor* body emission: each stage
    /// is one modifier's body statements, with the constructor body pushed as the
    /// final stage; a `_;` placeholder recurses to the next stage. Empty for a
    /// regular (non-constructor) emission, whose modifiers are wrapped as separate
    /// `sol.func`s reached through `modifier_body_call` instead. A constructor has
    /// no return value, so it needs no separate body func.
    modifier_stages: Vec<Statements>,
    /// Per-stage modifier parameter bindings, parallel to `modifier_stages`.
    modifier_stage_params: Vec<Vec<(NodeId, Value<'context, 'block>, Type<'context>)>>,
    /// The stage [`Self::emit_inline_modifier_chain`] is currently emitting.
    modifier_stage_index: usize,
    /// Arithmetic overflow-checking mode for binary operations.
    ///
    /// [`ArithmeticMode::Checked`] by default; [`ArithmeticMode::Unchecked`]
    /// inside `unchecked {}` blocks.
    arithmetic_mode: ArithmeticMode,
    /// User-defined Yul functions in scope within an `assembly { … }` block,
    /// keyed by name. Each is inlined at its call sites; an entry lives only
    /// for the duration of the Yul block (or inlined function frame) that
    /// declares it, then is removed so outer-scope definitions remain.
    yul_functions: HashMap<String, YulFunctionDefinition>,
    /// Per-name inline-recursion guard: a Yul function currently being inlined
    /// has depth ≥ 1, so a recursive call is rejected (it would otherwise loop
    /// the compiler) rather than emitted.
    yul_inline_depth: HashMap<String, usize>,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        region: &Region<'context>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
        return_slots: &'state [Option<Value<'context, 'block>>],
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer: region as *const Region<'context>,
            storage_layout,
            return_types,
            return_slots,
            modifier_body_call: None,
            modifier_stages: Vec::new(),
            modifier_stage_params: Vec::new(),
            modifier_stage_index: 0,
            arithmetic_mode: ArithmeticMode::Checked,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
        }
    }

    /// Sets the modifier-stage hand-off the `_;` placeholder lowers to. Called by
    /// [`FunctionEmitter::emit_modifier_stage_func`] before threading a modifier
    /// body's statements.
    ///
    /// [`FunctionEmitter::emit_modifier_stage_func`]: crate::ast::contract::function::FunctionEmitter::emit_modifier_stage_func
    pub fn set_modifier_body_call(&mut self, call: ModifierBodyCall<'context, 'block>) {
        self.modifier_body_call = Some(call);
    }

    /// Loads the inline modifier-chain stages for a constructor body emission
    /// (built by [`FunctionEmitter::build_modifier_stages`], with the constructor
    /// body pushed as the final stage), then drive them with
    /// [`Self::emit_inline_modifier_chain`].
    pub fn set_modifier_stages(
        &mut self,
        modifier_stages: Vec<Statements>,
        modifier_stage_params: Vec<Vec<(NodeId, Value<'context, 'block>, Type<'context>)>>,
    ) {
        self.modifier_stages = modifier_stages;
        self.modifier_stage_params = modifier_stage_params;
        self.modifier_stage_index = 0;
    }

    /// Emits the inline modifier chain for a constructor body from the current
    /// stage. Stage 0 is the outermost modifier body; each `_;` placeholder
    /// recurses to the next stage, and the constructor body (the final stage)
    /// runs at the innermost `_;`. Each stage's modifier parameters are bound in a
    /// scope that brackets the whole stage — including the `_;` tail — so a
    /// repeated modifier keeps a distinct binding per use and the binding is gone
    /// once the stage unwinds. A constructor has no return value, so the chain
    /// simply unwinds past the last stage (no separate body call).
    pub fn emit_inline_modifier_chain(
        &mut self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let stage = self.modifier_stage_index;
        let Some(statements) = self.modifier_stages.get(stage).cloned() else {
            return Ok(Some(block));
        };
        self.modifier_stage_index = stage + 1;
        let params = self
            .modifier_stage_params
            .get(stage)
            .cloned()
            .unwrap_or_default();
        self.environment.enter_scope();
        for (node_id, pointer, element_type) in params {
            self.environment
                .define_variable(node_id, pointer, element_type);
        }
        let result = self.emit_block(statements, block);
        self.environment.exit_scope();
        self.modifier_stage_index = stage;
        result
    }

    /// Returns a reference to the current region.
    ///
    /// # Safety
    ///
    /// The region pointer is valid as long as the MLIR module exists,
    /// which outlives all emitters.
    pub fn region(&self) -> &Region<'context> {
        // SAFETY: The region is owned by the MLIR module and outlives this emitter.
        unsafe { &*self.region_pointer }
    }

    /// Switches the current region for emitting into Sol op regions.
    pub fn set_region(&mut self, region: &Region<'context>) {
        self.region_pointer = region as *const Region<'context>;
    }

    /// Builds an [`ExpressionEmitter`] for the current statement context,
    /// supplying the shared state, environment, storage layout, and arithmetic
    /// mode — the four fields every expression emission needs — so call sites do
    /// not repeat the construction. The unchecked loop-step is the one site that
    /// builds its emitter explicitly, with [`ArithmeticMode::Unchecked`].
    fn expression_emitter(&self) -> ExpressionEmitter<'_, 'context, 'block> {
        ExpressionEmitter::new(
            self.state,
            self.environment,
            self.storage_layout,
            self.arithmetic_mode,
        )
    }

    /// Evaluates a loop `condition` in `condition_block` and emits the
    /// `sol.condition` terminator. Shared by `for`, `while`, and `do-while`.
    ///
    /// # Errors
    ///
    /// Returns an error if the condition expression contains unsupported
    /// constructs.
    fn emit_loop_condition(
        &self,
        condition: &Expression,
        condition_block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<()> {
        let emitter = self.expression_emitter();
        let (condition_value, condition_end) = emitter.emit_value(condition, condition_block)?;
        let condition_boolean = emitter.emit_is_nonzero(condition_value, &condition_end);
        self.state
            .builder
            .emit_sol_condition(condition_boolean, &condition_end);
        Ok(())
    }

    /// Emits MLIR for a statement.
    ///
    /// Returns `Some(block)` as the continuation block for the next statement,
    /// or `None` if control flow was terminated.
    ///
    /// # Errors
    ///
    /// Returns an error if the statement contains unsupported constructs.
    pub fn emit(
        &mut self,
        statement: &Statement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match statement {
            Statement::VariableDeclarationStatement(declaration) => {
                self.emit_variable_declaration(declaration, block)
            }
            Statement::ExpressionStatement(expression_statement) => {
                let expression = expression_statement.expression();
                // A bare `_;` inside a modifier body is the placeholder for the
                // wrapped body (or the next modifier stage). A constructor's
                // modifiers run as an inline chain (`modifier_stages` set), where
                // `_;` recurses to the next stage / the constructor body; a regular
                // function's modifiers are separate `sol.func`s, where `_;` hands
                // off through `modifier_body_call`.
                if let Expression::Identifier(identifier) = &expression
                    && matches!(
                        identifier.resolve_to_built_in(),
                        Some(BuiltIn::ModifierUnderscore)
                    )
                {
                    if self.modifier_stages.is_empty() {
                        return self.emit_modifier_body_call(block);
                    }
                    return self.emit_inline_modifier_chain(block);
                }
                if let Expression::FunctionCallExpression(call) = &expression
                    && let Expression::Identifier(identifier) = call.operand()
                    && matches!(identifier.resolve_to_built_in(), Some(BuiltIn::Revert))
                {
                    return self.emit_revert_call(call, block);
                }
                // A bare type-name or `super` reference used as a statement is
                // only the type/keyword and has no value and no side effect —
                // solc evaluates and discards it, so emit nothing. Besides
                // `uint256;` / `super;`, an array-type expression `s[7][];`
                // parses as an index access with neither an index/start nor slice
                // bounds (`a[i]` always has a start, `a[i:j]`/`a[:j]` a bound), so
                // a bound-less index access is the `T[]` type form, not a value.
                let is_type_or_super_noop = match &expression {
                    Expression::ElementaryType(_)
                    | Expression::TypeExpression(_)
                    | Expression::SuperKeyword(_) => true,
                    Expression::IndexAccessExpression(index_access) => {
                        index_access.start().is_none() && index_access.end().is_none()
                    }
                    _ => false,
                };
                if is_type_or_super_noop {
                    return Ok(Some(block));
                }
                // A discarded statement whose value is a *type* reference — e.g.
                // `(cond ? M : M).D;`, where `M` is an imported module and `D` a
                // contract in it — has no runtime value (materialising a
                // module/contract would have no `sol` representation), but its
                // subexpressions may still have side effects (here the ternary
                // condition's `flag = true`). solc evaluates those and discards the
                // type, so emit only the side-effecting subexpressions.
                if Self::is_type_reference(&expression) {
                    return self.emit_type_reference_side_effects(
                        expression_statement.expression(),
                        block,
                    );
                }
                let emitter = self.expression_emitter();
                // A discarded tuple-valued conditional `(c ? (1, 2, 3) : (3, 2, 1));`
                // has no single value to emit, but its condition and the selected
                // branch may have side effects; route it through the tuple path
                // and discard the results. The statement is usually parenthesised
                // (a single-element tuple), so peel those first. A single-valued
                // conditional resolves a scalar type and emits normally below.
                let unwrapped = expression_statement.expression().unwrap_parens();
                if let Expression::ConditionalExpression(conditional) = &unwrapped
                    && matches!(conditional.get_type(), Some(SlangType::Tuple(_)))
                {
                    let (_values, block) =
                        emitter.emit_conditional_tuple_values(conditional, block)?;
                    return Ok(Some(block));
                }
                let (_, block) = emitter.emit(&expression, block)?;
                Ok(Some(block))
            }
            Statement::ReturnStatement(return_statement) => {
                self.emit_return(return_statement, block)
            }
            Statement::IfStatement(if_statement) => self.emit_if(if_statement, block),
            Statement::ForStatement(for_statement) => self.emit_for(for_statement, block),
            Statement::WhileStatement(while_statement) => self.emit_while(while_statement, block),
            Statement::DoWhileStatement(do_while) => self.emit_do_while(do_while, block),
            Statement::BreakStatement(_) => self.emit_break(block),
            Statement::ContinueStatement(_) => self.emit_continue(block),
            Statement::Block(inner) => self.emit_block(inner.statements(), block),
            Statement::UncheckedBlock(inner) => {
                let saved_mode = self.arithmetic_mode;
                self.arithmetic_mode = ArithmeticMode::Unchecked;
                let result = self.emit_block(inner.block().statements(), block);
                self.arithmetic_mode = saved_mode;
                result
            }
            Statement::RevertStatement(revert) => self.emit_revert(revert, block),
            Statement::EmitStatement(emit_statement) => self.emit_event(emit_statement, block),
            Statement::AssemblyStatement(assembly) => self.emit_assembly(assembly, block),
            Statement::TryStatement(try_statement) => self.emit_try(try_statement, block),
        }
    }

    /// Emits a sequence of statements inside a new lexical scope.
    ///
    /// # Errors
    ///
    /// Returns an error if any statement contains unsupported constructs.
    pub fn emit_block(
        &mut self,
        statements: Statements,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.environment.enter_scope();
        let mut current = block;
        for statement in statements.iter() {
            match self.emit(&statement, current)? {
                Some(next) => current = next,
                None => {
                    self.environment.exit_scope();
                    return Ok(None);
                }
            }
        }
        self.environment.exit_scope();
        Ok(Some(current))
    }

    /// Whether `expression` is a member access that is a compile-time type /
    /// module reference (`M.D`, `(cond ? M : M).D`, `C.S`) rather than a runtime
    /// value. slang gives a value-returning member access (a field, getter,
    /// `.length`, a built-in like `block.timestamp`) a type; a member access onto
    /// a module/contract namespace that names a type has none.
    fn is_type_reference(expression: &Expression) -> bool {
        matches!(
            expression,
            Expression::MemberAccessExpression(access) if access.get_type().is_none()
        )
    }

    /// Emits only the side effects of a discarded type-reference expression: a
    /// member access recurses into its operand; a conditional whose branches are
    /// types runs only its condition (the type branches are compile-time, with no
    /// runtime value or side effect); any other type/module reference has none.
    fn emit_type_reference_side_effects(
        &mut self,
        expression: Expression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        match expression.unwrap_parens() {
            Expression::MemberAccessExpression(access) => {
                self.emit_type_reference_side_effects(access.operand(), block)
            }
            Expression::ConditionalExpression(conditional) => {
                let emitter = self.expression_emitter();
                let (_, block) = emitter.emit_value(&conditional.operand(), block)?;
                Ok(Some(block))
            }
            _ => Ok(Some(block)),
        }
    }

    /// Emits a `sol.break` terminator.
    fn emit_break(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_break(&block);
        Ok(None)
    }

    /// Emits a `sol.continue` terminator.
    fn emit_continue(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        self.state.builder.emit_sol_continue(&block);
        Ok(None)
    }

    /// Emits a return statement.
    ///
    /// A multi-element tuple expression in the return position is unpacked
    /// into one value per declared return slot; any other expression yields
    /// a single value. Each value is cast to its corresponding declared
    /// return type before being emitted as a `sol.return` operand.
    fn emit_return(
        &mut self,
        return_statement: &slang_solidity_v2::ast::ReturnStatement,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let Some(expression) = return_statement.expression() else {
            // A bare `return;` returns the current values of the return slots
            // (zero for an unnamed/unset slot), so its `sol.return` arity matches
            // the enclosing function — like the fall-through epilogue. This
            // matters for a `return;` in a modifier stage of a value-returning
            // function, where a 0-operand return would fail verification; a void
            // function has no slots and returns nothing.
            self.state
                .builder
                .emit_return_from_slots(self.return_types, self.return_slots, &block);
            return Ok(None);
        };

        let emitter = self.expression_emitter();

        let (values, block) = if let Expression::TupleExpression(tuple) = &expression
            && tuple.items().len() > 1
        {
            let items = tuple.items();
            let mut values = Vec::with_capacity(items.len());
            let mut current = block;
            for item in items.iter() {
                let inner = item
                    .expression()
                    .expect("a return tuple element has an inner expression");
                let (value, next) = emitter.emit_value(&inner, current)?;
                values.push(value);
                current = next;
            }
            (values, current)
        } else if self.return_types.len() > 1 {
            // A single expression that yields multiple values is either a
            // tuple-returning call (`return f();`), where solc emits one
            // `sol.call` with N results, or a conditional with tuple branches
            // (`return cond ? (1, 2) : (3, 4);`). Expand its full result list so
            // the `sol.return` arity matches rather than taking the first value.
            match &expression {
                Expression::FunctionCallExpression(call) => {
                    CallEmitter::new(&emitter).emit_function_call_results(call, block)?
                }
                Expression::ConditionalExpression(conditional) => {
                    emitter.emit_conditional_tuple_values(conditional, block)?
                }
                _ => {
                    unimplemented!("multi-value return of a non-call expression is not supported")
                }
            }
        } else {
            // A single-value return materialises a string literal toward the
            // declared return type (a `bytesN`/`byte` constant), not a runtime
            // string the cast below would reject.
            let return_type = self.return_types[0];
            let (value, block) = emitter.emit_value_for_target(&expression, return_type, block)?;
            (vec![value], block)
        };

        let cast_values: Vec<_> = values
            .into_iter()
            .zip(self.return_types.iter())
            .map(|(value, &return_type)| {
                TypeConversion::coerce(value, return_type, &self.state.builder, &block)
            })
            .collect();

        self.state.builder.emit_sol_return(&cast_values, &block);
        Ok(None)
    }

    /// Lowers a modifier stage's `_;` placeholder to the modifier-body hand-off
    /// (call the wrapped body / next stage, threading the shared return values),
    /// delegating to [`ModifierBodyCall::emit`]. Outside a modifier stage `_;`
    /// has no hand-off set and emits nothing.
    ///
    /// # Errors
    ///
    /// Returns an error if the hand-off call cannot be lowered.
    fn emit_modifier_body_call(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        if let Some(body_call) = &self.modifier_body_call {
            body_call.emit(&self.state.builder, &block)?;
        }
        Ok(Some(block))
    }
}
