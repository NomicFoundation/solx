//!
//! The assembly scope: the enclosing function scope, the Yul bindings an assembly block introduces,
//! and the return slots `leave` reaches.
//!

use std::collections::HashMap;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;

use solx_mlir::Context;
use solx_mlir::Slot;
use solx_mlir::YulBlock;
use solx_mlir::YulFunction;

use crate::scope::function::FunctionScope;

/// The assembly scope: the enclosing function scope a Solidity reference resolves through, the Yul
/// variables and functions the block declares, and the slots a `leave` returns.
///
/// Bindings are keyed on the declaration's node id rather than its name, so a name reused across
/// sibling Yul blocks resolves to the right slot without scope bookkeeping. Yul functions are
/// flattened into one map because `sol.inline_asm` is a single symbol table: a function nested in
/// another function's body is still a sibling symbol.
pub struct AssemblyScope<'function, 'contract, 'source_unit, 'context> {
    /// The function scope this assembly block is lowered within.
    pub function: &'function mut FunctionScope<'contract, 'source_unit, 'context>,
    /// The slot of each Yul variable and parameter, keyed by its declaring identifier.
    pub variables: HashMap<NodeId, Slot<'context>>,
    /// The signature of each Yul function the block declares, keyed by its definition.
    pub functions: HashMap<NodeId, YulFunction>,
    /// The return-variable slots of the Yul function being lowered, which `leave` loads and returns.
    /// Empty at the top level of the assembly block, where `leave` is illegal.
    pub returns: Vec<Slot<'context>>,
}

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// Opens an assembly scope within `function`.
    pub fn new(function: &'function mut FunctionScope<'contract, 'source_unit, 'context>) -> Self {
        Self {
            function,
            variables: HashMap::new(),
            functions: HashMap::new(),
            returns: Vec::new(),
        }
    }

    /// The Yul block the insertion cursor points at.
    pub fn current_block(&self) -> YulBlock<'context> {
        YulBlock::from(self.function.current_block())
    }

    /// Emits into `block`, terminating it with `yul.yield` if the emitted code did not, and restores
    /// the cursor to the enclosing block. Every Yul control-flow region takes this shape.
    pub fn region(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        let enclosing = self.cursor().replace(block.into());
        emit(self);
        let end = self.current_block();
        if !end.is_terminated() {
            end.r#yield(self);
        }
        *self.cursor() = enclosing;
    }

    /// Emits into `block` and restores the cursor, appending no terminator: the `sol.inline_asm`
    /// body carries none, and a `yul.func` body is closed by its own lowering.
    pub fn body(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        let enclosing = self.cursor().replace(block.into());
        emit(self);
        *self.cursor() = enclosing;
    }

    /// Runs `emit` with `returns` installed as the slots a `leave` returns, restoring the enclosing
    /// ones afterwards.
    pub fn in_function(&mut self, returns: Vec<Slot<'context>>, emit: impl FnOnce(&mut Self)) {
        let enclosing = std::mem::replace(&mut self.returns, returns);
        emit(self);
        self.returns = enclosing;
    }

    /// The signature of the Yul function `definition` declares.
    pub fn signature(&self, definition: NodeId) -> &YulFunction {
        self.functions
            .get(&definition)
            .expect("every Yul function in the block is declared before any body is emitted")
    }

    /// The slot of the Yul variable `declaration` declares.
    pub fn variable(&self, declaration: NodeId) -> Slot<'context> {
        *self
            .variables
            .get(&declaration)
            .expect("every Yul reference resolves to a declaration this block bound")
    }

    /// The insertion cursor the enclosing MLIR context holds.
    fn cursor(&mut self) -> &mut Option<solx_mlir::Block<'context>> {
        &mut self.function.contract.source_unit.mlir.current_block
    }
}

impl<'function, 'contract, 'source_unit, 'context> Deref
    for AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.function.contract.source_unit.mlir
    }
}
