//!
//! Inline-assembly (Yul) statement emission.
//!

/// Yul block emission: function-definition hoisting and the statement walk.
pub mod block;
/// Yul expression emission: literals, path reads, calls.
pub mod expression;
/// Yul function-call emission: EVM-opcode intrinsics and user-defined inlining.
pub mod function_call;
/// Yul statement emission.
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Region;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulFunctionDefinition;
use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::Emit;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::storage_layout::StorageSlot;

/// The threaded scope of inline-assembly emission: the Yul-dialect peer of
/// [`StatementContext`], pure data. Carries only what Yul emission reads — the
/// shared MLIR context, the variable environment (Sol locals and Yul locals
/// coexist), the region pointer, the storage layout (for `stateVar.slot`/`.offset`),
/// and the in-scope user-defined Yul functions with their inline-recursion guard.
pub struct YulContext<'frame, 'context, 'block> {
    /// The shared MLIR context.
    state: &'frame Context<'context>,
    /// Variable environment (mutable for Yul `let` declarations).
    environment: &'frame mut Environment<'context, 'block>,
    /// The current region for creating new blocks. A raw pointer to allow
    /// switching between Yul op regions without lifetime conflicts.
    region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    /// User-defined Yul functions in scope, keyed by name; each is inlined at its
    /// call sites and lives only for the declaring block / inlined frame.
    yul_functions: HashMap<String, YulFunctionDefinition>,
    /// Per-name inline-recursion guard: a function being inlined has depth ≥ 1, so
    /// a recursive call is rejected (it would loop the compiler).
    yul_inline_depth: HashMap<String, usize>,
}

impl<'frame, 'context, 'block> YulContext<'frame, 'context, 'block> {
    /// Opens a Yul scope over the enclosing function's context and region.
    pub fn new(
        state: &'frame Context<'context>,
        environment: &'frame mut Environment<'context, 'block>,
        region_pointer: *const Region<'context>,
        storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            environment,
            region_pointer,
            storage_layout,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
        }
    }

    /// Switches the current region for emitting into a Yul op's region.
    pub fn set_region(&mut self, region: &Region<'context>) {
        self.region_pointer = region as *const Region<'context>;
    }
}

// An `assembly { … }` block is the top-level Yul block, emitted with
// function-definition hoisting while reusing the enclosing function scope (no
// nested lexical scope).
statement_emit!(AssemblyStatement; |node, context, block| {
    let mut yul_context = YulContext::new(
        context.state,
        &mut *context.environment,
        context.region_pointer,
        context.storage_layout,
    );
    node.body().emit(&mut yul_context, block)
});
