//!
//! Inline-assembly (Yul) statement emission.
//!

pub mod block;
pub mod expression;
pub mod function_call;
pub mod statement;

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Value as MlirValue;
use slang_solidity_v2::ast::AssemblyStatement;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Context;
use solx_mlir::Environment;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::analysis::query::storage_layout::StorageSlot;
use crate::ast::contract::function::statement::StatementContext;
use crate::ast::emit::emit_statement::EmitStatement;
use crate::ast::emit::emit_yul::EmitYul;

/// The threaded scope of inline-assembly emission: the Yul-dialect peer of [`StatementContext`], pure data.
///
/// Yul locals are opaque `!llvm.ptr` stack slots keyed by their declaration's `NodeId`, so same-named
/// locals across scopes stay distinct. They live in this context rather than the shared Sol
/// [`Environment`], whose bindings carry a Sol element type a Yul word does not have. The enclosing
/// function's environment is still borrowed so a Yul path to a Solidity `constant` folds its initializer.
pub struct YulContext<'frame, 'context, 'block> {
    /// The shared MLIR context.
    pub state: &'frame Context<'context>,
    /// The enclosing function's environment, borrowed to fold a referenced Solidity `constant`.
    pub environment: &'frame Environment<'context, 'block>,
    /// State variable node ID to storage slot mapping.
    pub storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    /// Stack of Yul-local scopes, each mapping a declaration's `NodeId` to its `!llvm.ptr` slot.
    scopes: Vec<HashMap<NodeId, MlirValue<'context, 'block>>>,
    /// In-scope user Yul functions, keyed by node id so like-named functions in disjoint scopes differ.
    pub yul_functions: HashMap<NodeId, YulFunctionDefinition>,
    /// Inline-recursion guard keyed by node id: depth >= 1 rejects a recursive inline.
    pub yul_inline_depth: HashMap<NodeId, usize>,
}

impl<'frame, 'context, 'block> YulContext<'frame, 'context, 'block> {
    /// Opens a Yul scope over the enclosing function's context.
    pub fn new(
        state: &'frame Context<'context>,
        environment: &'frame Environment<'context, 'block>,
        storage_layout: &'frame HashMap<NodeId, StorageSlot>,
    ) -> Self {
        Self {
            state,
            environment,
            storage_layout,
            scopes: vec![HashMap::new()],
            yul_functions: HashMap::new(),
            yul_inline_depth: HashMap::new(),
        }
    }

    /// Pushes a new Yul-local scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the innermost Yul-local scope.
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Registers a Yul local's slot in the current scope, keyed by its declaration's `NodeId`.
    pub fn define_variable(&mut self, declaration: NodeId, slot: MlirValue<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(declaration, slot);
    }

    /// Looks up a Yul local's slot by its declaration's `NodeId`, searching from the innermost scope outward.
    pub fn variable(&self, declaration: NodeId) -> MlirValue<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(slot) = scope.get(&declaration) {
                return *slot;
            }
        }
        unreachable!("unregistered yul local: {declaration:?}");
    }

    /// The `!llvm.ptr` slot a Yul variable reference reads or writes.
    ///
    /// A Yul `let` declaration is a native `!llvm.ptr` word slot in this context's scopes; a Solidity
    /// local, parameter, or return variable it aliases lives in the enclosing [`Environment`] as a
    /// `!sol.ptr<T, Stack>`, reinterpreted here to the `!llvm.ptr` Yul operates on.
    pub fn slot(
        &self,
        identifier: &Identifier,
        block: &BlockRef<'context, 'block>,
    ) -> MlirValue<'context, 'block> {
        let definition = identifier
            .resolve_to_definition()
            .expect("yul variable reference resolves to a declaration");
        if matches!(
            definition,
            Definition::YulVariable(_) | Definition::YulParameter(_)
        ) {
            return self.variable(definition.node_id());
        }
        let (pointer, _) = self.environment.variable_with_type(&identifier.name());
        AstValue::new(pointer)
            .reinterpret(AstType::llvm_ptr(self.state.mlir_context), self.state, block)
            .into_mlir()
    }
}

statement_emit!(AssemblyStatement; |node, context, block| {
    let mut yul_context =
        YulContext::new(context.state, context.environment, context.storage_layout);
    node.body().emit(&mut yul_context, block)
});
