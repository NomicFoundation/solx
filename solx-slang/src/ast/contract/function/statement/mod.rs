//!
//! Statement lowering to MLIR operations.
//!

/// Inline-assembly (Yul) statement lowering.
pub mod assembly;
/// Control flow statement lowering (`if`, `for`, `while`, `do`/`while`).
pub mod control_flow;
/// Event emit statement lowering.
pub mod event;
/// Expression statement lowering.
pub mod expression_statement;
/// Named call-argument ordering.
pub mod named_arguments;
/// Return statement lowering.
pub mod return_statement;
/// Revert statement lowering.
pub mod revert;
/// Try-catch statement lowering.
pub mod try_statement;
/// Local variable declaration statement lowering.
pub mod variable_declaration;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Statement;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::function::storage_slot::StorageSlot;

/// How a modifier stage's `_;` placeholder reaches the next stage (or the
/// wrapped function body): it calls `symbol`, forwarding `forward_params` (the
/// downstream stages' arguments followed by the wrapped function's parameters)
/// plus the current return values loaded from `return_slots`, and stores the
/// results back into `return_slots` so the modifier tail and epilogue observe
/// them.
pub struct ModifierBodyCall<'context, 'block> {
    /// Symbol of the `sol.func` to call at `_;` — the next stage, or `$body`.
    pub symbol: String,
    /// Values forwarded verbatim to the next stage (downstream arguments and
    /// the wrapped function's parameters).
    pub forward_params: Vec<Value<'context, 'block>>,
    /// The stage's return slots; the call's results are stored here.
    pub return_slots: Vec<Option<Value<'context, 'block>>>,
}

/// Lowers Solidity statements to MLIR operations with control flow.
///
/// Returns `Some(block)` as the continuation block, or `None` when control
/// flow has been terminated (by `return`, `break`, or `continue`).
pub struct StatementEmitter<'state, 'context, 'block> {
    /// The shared MLIR context.
    state: &'state Context<'context>,
    /// Variable environment (mutable for new declarations and loop targets).
    environment: &'state mut Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'state HashMap<NodeId, StorageSlot>,
    /// The function's declared return types, for `return` to cast to.
    return_types: &'state [Type<'context>],
    /// The function's named-return alloca pointers, parallel to `return_types`
    /// (`None` for an unnamed return). A bare `return;` loads these — the current
    /// values of the named returns — instead of returning no operands, matching
    /// the fall-off-the-end epilogue. Empty for a void function.
    return_slots: &'state [Option<Value<'context, 'block>>],
    /// Whether arithmetic operations use checked variants (`sol.cadd` etc.).
    ///
    /// `true` by default; `false` inside `unchecked {}` blocks.
    checked: bool,
    /// Yul function definitions in scope for the assembly block currently being
    /// lowered, keyed by name. Each `assembly { ... }` registers its own
    /// definitions (so calls resolve regardless of textual order) and removes
    /// them again on exit. Empty outside inline assembly.
    yul_functions: HashMap<String, YulFunctionDefinition>,
    /// Per-name inlining depth of Yul user-defined functions, used to reject
    /// recursive calls (which would otherwise inline forever at compile time).
    yul_inline_depth: HashMap<String, usize>,
    /// When emitting a modifier stage body, how its `_;` placeholder calls the
    /// next stage (or the wrapped body). `None` outside a modifier stage.
    modifier_body_call: Option<ModifierBodyCall<'context, 'block>>,
}

impl<'state, 'context, 'block> StatementEmitter<'state, 'context, 'block> {
    /// Creates a new statement emitter.
    pub fn new(
        state: &'state Context<'context>,
        environment: &'state mut Environment<'context, 'block>,
        storage_layout: &'state HashMap<NodeId, StorageSlot>,
        return_types: &'state [Type<'context>],
        return_slots: &'state [Option<Value<'context, 'block>>],
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            return_types,
            return_slots,
            checked: true,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
            modifier_body_call: None,
        }
    }

    /// Sets the modifier-stage `_;` target, returning the emitter. Called when
    /// emitting a modifier stage body so its `_;` calls the next stage / body.
    pub fn with_modifier_body_call(
        mut self,
        modifier_body_call: ModifierBodyCall<'context, 'block>,
    ) -> Self {
        self.modifier_body_call = Some(modifier_body_call);
        self
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
            Statement::ExpressionStatement(statement) => {
                // A bare `_;` inside a modifier body is the placeholder for the
                // next modifier stage (or the wrapped function body).
                if let Expression::Identifier(identifier) = &statement.expression()
                    && matches!(
                        identifier.resolve_to_built_in(),
                        Some(BuiltIn::ModifierUnderscore)
                    )
                {
                    return self.emit_modifier_body_call(block);
                }
                self.emit_expression_statement(statement, block)
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
            Statement::UncheckedBlock(inner) => self.emit_unchecked_block(inner, block),
            Statement::EmitStatement(emit_statement) => self.emit_event(emit_statement, block),
            Statement::RevertStatement(revert) => self.emit_revert(revert, block),
            Statement::TryStatement(try_statement) => self.emit_try(try_statement, block),
            Statement::AssemblyStatement(assembly) => self.emit_assembly(assembly, block),
        }
    }

    /// Emits a modifier stage's `_;` placeholder: calls the next stage / wrapped
    /// body (`modifier_body_call.symbol`), forwarding its parameters plus the
    /// current return values loaded from the return slots, then stores the
    /// results back into those slots so the modifier tail and epilogue observe
    /// them. Control resumes after `_;` (the modifier tail), so this returns
    /// `Some(block)`.
    fn emit_modifier_body_call(
        &self,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<BlockRef<'context, 'block>>> {
        let body_call = self
            .modifier_body_call
            .as_ref()
            .expect("`_;` placeholder outside a modifier body");
        let mut operands = body_call.forward_params.clone();
        for (slot, &return_type) in body_call.return_slots.iter().zip(self.return_types) {
            if let Some(pointer) = slot {
                operands.push(
                    self.state
                        .builder
                        .emit_sol_load(*pointer, return_type, &block)?,
                );
            }
        }
        let results = self.state.builder.emit_sol_call_results(
            &body_call.symbol,
            &operands,
            self.return_types,
            &block,
        )?;
        for (slot, value) in body_call.return_slots.iter().zip(results) {
            if let Some(pointer) = slot {
                self.state.builder.emit_sol_store(value, *pointer, &block);
            }
        }
        Ok(Some(block))
    }
}
