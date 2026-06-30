//!
//! Inline-assembly (Yul) statement emission.
//!

pub mod block;
pub mod expression;
pub mod function_call;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Region;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulFunctionDefinition;
use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::EmitStatement;
use crate::ast::EmitYul;
use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::storage_layout::StorageSlot;

/// The threaded scope of inline-assembly emission: the Yul-dialect peer of [`StatementContext`], pure data.
pub struct YulContext<'frame, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'frame Context<'context>,
    /// Variable environment, mutable for Yul `let` declarations.
    pub environment: &'frame mut Environment<'context, 'block>,
    /// Contract-local dispatch metadata.
    pub dispatch: &'frame ContractDispatch,
    /// The current region for creating new blocks. A raw pointer to allow
    /// switching between Yul op regions without lifetime conflicts.
    pub region_pointer: *const Region<'context>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    /// User-defined Yul functions in scope, keyed by name; each is inlined at its
    /// call sites and lives only for the declaring block / inlined frame.
    pub yul_functions: HashMap<String, YulFunctionDefinition>,
    /// Per-name inline-recursion guard: a function being inlined has depth >= 1, so
    /// a recursive call is rejected (it would loop the compiler).
    pub yul_inline_depth: HashMap<String, usize>,
}

impl<'frame, 'context, 'block> YulContext<'frame, 'context, 'block> {
    /// Opens a Yul scope over the enclosing function's context and region.
    pub fn new(
        state: &'frame Context<'context>,
        environment: &'frame mut Environment<'context, 'block>,
        dispatch: &'frame ContractDispatch,
        region_pointer: *const Region<'context>,
        storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            environment,
            dispatch,
            region_pointer,
            storage_layout,
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
        }
    }
}

statement_emit!(AssemblyStatement; |node, context, block| {
    let mut yul_context = YulContext::new(
        context.state,
        &mut *context.environment,
        context.dispatch,
        context.region_pointer,
        context.storage_layout,
    );
    node.body().emit(&mut yul_context, block)
});
