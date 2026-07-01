//!
//! Inline-assembly (Yul) statement emission.
//!

pub mod block;
pub mod expression;
pub mod function_call;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockRef;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulFunctionDefinition;
use solx_mlir::Context;
use solx_mlir::Environment;

use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::contract::storage_layout::StorageSlot;
use crate::ast::emit::emit_statement::EmitStatement;
use crate::ast::emit::emit_yul::EmitYul;

/// The threaded scope of inline-assembly emission: the Yul-dialect peer of [`StatementContext`], pure data.
pub struct YulContext<'frame, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'frame Context<'context>,
    /// Variable environment, mutable for Yul `let` declarations.
    pub environment: &'frame mut Environment<'context, 'block>,
    /// Contract-local dispatch metadata.
    pub dispatch: &'frame ContractDispatch,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    /// In-scope user Yul functions, keyed by node id so like-named functions in disjoint scopes differ.
    pub yul_functions: HashMap<NodeId, YulFunctionDefinition>,
    /// Inline-recursion guard keyed by node id: depth >= 1 rejects a recursive inline.
    pub yul_inline_depth: HashMap<NodeId, usize>,
}

impl<'frame, 'context, 'block> YulContext<'frame, 'context, 'block> {
    /// Opens a Yul scope over the enclosing function's context.
    pub fn new(
        state: &'frame Context<'context>,
        environment: &'frame mut Environment<'context, 'block>,
        dispatch: &'frame ContractDispatch,
        storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            environment,
            dispatch,
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
        context.storage_layout,
    );
    node.body().emit(&mut yul_context, block)
});
