//!
//! The assembly scope: the enclosing function scope, the Yul bindings an assembly block introduces,
//! and the frame being lowered.
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

/// Which frame of an assembly block is being lowered. `yul.func` is `IsolatedFromAbove`, so a
/// function body reaches neither the enclosing Solidity locals nor an enclosing Yul function's
/// variables, and it is the only frame `leave` is legal in.
pub enum YulFrame<'context> {
    /// The assembly block's own body.
    Block,
    /// A `yul.func` body, holding the return-variable pointers `leave` loads and returns.
    Function(Vec<Pointer<'context>>),
}

/// The assembly scope: the enclosing function scope a Solidity reference resolves through, the
/// `sol.inline_asm` body the block's Yul functions are emitted into, the Yul variables and
/// functions the block declares, and the frame being lowered.
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
    /// The pointer of each Yul variable and parameter the current frame binds, keyed by its
    /// declaring identifier.
    pub variables: HashMap<NodeId, Pointer<'context>>,
    /// The signature of each Yul function named so far, keyed by its definition. A key is present
    /// exactly when the function's `yul.func` has been emitted into [`Self::body`].
    pub function_signatures: HashMap<NodeId, YulFunction<'context>>,
    /// The frame being lowered.
    pub frame: YulFrame<'context>,
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
            frame: YulFrame::Block,
        }
    }

    /// The Yul block the insertion cursor points at.
    pub fn current_block(&self) -> YulBlock<'context> {
        YulBlock::from(self.function.current_block())
    }

    /// Emits into `block`, terminating it with `yul.yield` if the emitted code did not, and restores
    /// the cursor to the enclosing block. Every Yul control-flow region takes this shape.
    pub fn region(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        self.in_block(block, |scope| {
            emit(scope);
            let end = scope.current_block();
            if !end.is_terminated() {
                end.r#yield(scope);
            }
        });
    }

    /// Emits a `yul.func` body into `block` and restores the cursor, appending no terminator. The
    /// enclosing frame's variables go out of scope for the duration, since `yul.func` reaches
    /// nothing defined above it; `emit` declares the frame's return variables through
    /// [`Self::bind_return`].
    pub fn function_body(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        let variables = std::mem::take(&mut self.variables);
        let frame = std::mem::replace(&mut self.frame, YulFrame::Function(Vec::new()));
        self.in_block(block, emit);
        self.frame = frame;
        self.variables = variables;
    }

    /// Whether the lowering sits inside a `yul.func` body.
    pub fn in_function(&self) -> bool {
        matches!(self.frame, YulFrame::Function(_))
    }

    /// The return-variable pointers a `leave` in the current frame loads and returns.
    pub fn returns(&self) -> &[Pointer<'context>] {
        match &self.frame {
            YulFrame::Function(returns) => returns,
            YulFrame::Block => unreachable!("`leave` is legal only inside a Yul function"),
        }
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

    /// Binds a Yul function's zero-initialized return variable and records it in the frame.
    pub fn bind_return(&mut self, declaration: NodeId) {
        let pointer = Pointer::alloca(self);
        let zero = Word::zero(self);
        pointer.store(zero, self);
        self.variables.insert(declaration, pointer);
        match &mut self.frame {
            YulFrame::Function(returns) => returns.push(pointer),
            YulFrame::Block => unreachable!("a return variable is declared by a Yul function"),
        }
    }

    /// The pointer of the Yul variable `declaration` declares.
    pub fn variable(&self, declaration: NodeId) -> Pointer<'context> {
        *self
            .variables
            .get(&declaration)
            .expect("every Yul reference resolves to a declaration this block bound")
    }

    /// Emits into `block` and restores the cursor to the enclosing block.
    fn in_block(&mut self, block: YulBlock<'context>, emit: impl FnOnce(&mut Self)) {
        let enclosing = self.current_block_mut().replace(block.into());
        emit(self);
        *self.current_block_mut() = enclosing;
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
