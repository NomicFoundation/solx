//!
//! The assembly scope: the enclosing function scope, the Yul bindings an assembly block introduces,
//! and the return pointers `leave` reaches.
//!

use std::collections::HashMap;
use std::ops::Deref;

use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::YulFunctionDefinition;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Pointer;
use solx_mlir::Word;
use solx_mlir::YulBlock;
use solx_mlir::YulFunction;

use crate::scope::function::FunctionScope;

/// The assembly scope: the enclosing function scope a Solidity reference resolves through, the
/// `sol.inline_asm` body the block's Yul functions are emitted into, the Yul variables and
/// functions the block declares, and the pointers a `leave` returns.
///
/// Bindings are keyed on the declaration's node id rather than its name, so a name reused across
/// sibling Yul blocks resolves to the right pointer without scope bookkeeping. Yul functions are
/// flattened into one map because `sol.inline_asm` is a single symbol table: a function nested in
/// another function's body is still a sibling symbol.
pub struct AssemblyScope<'function, 'contract, 'source_unit, 'context> {
    /// The function scope this assembly block is lowered within.
    pub function: &'function mut FunctionScope<'contract, 'source_unit, 'context>,
    /// The `sol.inline_asm` body: the symbol table every `yul.func` is emitted into.
    pub body: Block<'context>,
    /// The pointer of each Yul variable and parameter, keyed by its declaring identifier.
    pub variables: HashMap<NodeId, Pointer<'context>>,
    /// The signature of each Yul function named so far, keyed by its definition. A key is present
    /// exactly when the function's `yul.func` has been emitted into [`Self::body`].
    pub function_signatures: HashMap<NodeId, YulFunction<'context>>,
    /// The return-variable pointers of the Yul function being lowered, which `leave` loads and
    /// returns. Empty at the top level of the assembly block, where `leave` is illegal.
    pub returns: Vec<Pointer<'context>>,
}

impl<'function, 'contract, 'source_unit, 'context>
    AssemblyScope<'function, 'contract, 'source_unit, 'context>
{
    /// Opens an assembly scope within `function`, with `body` as its symbol table.
    pub fn new(
        function: &'function mut FunctionScope<'contract, 'source_unit, 'context>,
        body: Block<'context>,
    ) -> Self {
        Self {
            function,
            body,
            variables: HashMap::new(),
            function_signatures: HashMap::new(),
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
        self.function_body(block, |scope| {
            emit(scope);
            let end = scope.current_block();
            if !end.is_terminated() {
                end.r#yield(scope);
            }
        });
    }

    /// Emits a `yul.func` body into `block` and restores the cursor, appending no terminator.
    pub fn function_body(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        let enclosing = self.current_block_mut().replace(block.into());
        emit(self);
        *self.current_block_mut() = enclosing;
    }

    /// Runs `emit` with `returns` installed as the pointers a `leave` returns, restoring the
    /// enclosing ones afterwards.
    pub fn in_function(&mut self, returns: Vec<Pointer<'context>>, emit: impl FnOnce(&mut Self)) {
        let enclosing = std::mem::replace(&mut self.returns, returns);
        emit(self);
        self.returns = enclosing;
    }

    /// Interns the signature of the Yul function `definition` declares and records it as named.
    /// The symbol carries the definition's node id because Yul scopes a function to its block
    /// while `sol.inline_asm` is one flat symbol table: sibling blocks may each declare an `f`,
    /// and the bare name would collide.
    pub fn signature(&mut self, definition: &YulFunctionDefinition) -> YulFunction<'context> {
        let signature = YulFunction::new(
            format!("{}_{}", definition.name().name(), definition.node_id()),
            definition.parameters().len(),
            definition.returns().map_or(0, |returns| returns.len()),
            self,
        );
        self.function_signatures
            .insert(definition.node_id(), signature.clone());
        signature
    }

    /// Binds a Yul variable: allocates its pointer, stores `value` into it, and records it under the
    /// declaring identifier a reference to it resolves through.
    pub fn bind(&mut self, declaration: NodeId, value: Word<'context>) -> Pointer<'context> {
        let pointer = Pointer::alloca(self);
        pointer.store(value, self);
        self.variables.insert(declaration, pointer);
        pointer
    }

    /// The pointer of the Yul variable `declaration` declares.
    pub fn variable(&self, declaration: NodeId) -> Pointer<'context> {
        *self
            .variables
            .get(&declaration)
            .expect("every Yul reference resolves to a declaration this block bound")
    }

    /// The insertion cursor the enclosing MLIR context holds, for repositioning it onto a region.
    fn current_block_mut(&mut self) -> &mut Option<Block<'context>> {
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
