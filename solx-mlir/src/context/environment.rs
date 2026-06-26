//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::Value;
use slang_solidity_v2::ast::NodeId;

use crate::Builder;
use crate::Pointer;
use crate::Type;

/// Tracks variable places (alloca'd pointers) for lexical scoping.
///
/// Bindings are keyed by the declaration's Slang `NodeId`, not its textual name, so same-named
/// locals across scopes (shadowing) are distinct. Lookups search from the innermost scope outward.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping a declaration's `NodeId` to its place (scope 0 holds parameters).
    scopes: Vec<HashMap<NodeId, Value<'context, 'block>>>,
    /// Declarations bound directly to an SSA value rather than a place: a read yields the value with
    /// no `sol.load`. Used inside a `sol.modifier_call_blk`, whose `IsolatedFromAbove` block exposes
    /// the wrapping function's parameters as block-argument values (no spilling to a stack slot).
    value_bindings: HashMap<NodeId, Value<'context, 'block>>,
}

impl<'context, 'block> Environment<'context, 'block> {
    /// Creates a new environment with a single root scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            value_bindings: HashMap::new(),
        }
    }

    /// Binds `declaration` directly to the SSA `value` (not a place): a read returns `value` itself,
    /// emitting no `sol.load`. Used to expose a `sol.modifier_call_blk` block argument as a parameter.
    pub fn bind_value(&mut self, declaration: NodeId, value: Value<'context, 'block>) {
        self.value_bindings.insert(declaration, value);
    }

    /// The SSA value `declaration` is directly bound to, if any (see [`Self::bind_value`]).
    pub fn value_binding(&self, declaration: NodeId) -> Option<Value<'context, 'block>> {
        self.value_bindings.get(&declaration).copied()
    }

    /// Pushes a new lexical scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the innermost lexical scope.
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Registers a variable's place in the current scope, keyed by its declaration's `NodeId`.
    pub fn define_variable(&mut self, declaration: NodeId, pointer: Value<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(declaration, pointer);
    }

    /// Coerces `value` to `parameter_type`, spills it to a fresh stack slot, and binds `declaration` to it.
    pub fn bind_parameter(
        &mut self,
        declaration: NodeId,
        parameter_type: MlirType<'context>,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let cast = crate::Value::new(value).cast(Type::new(parameter_type), builder, block);
        let pointer = Pointer::stack_slot(Type::new(parameter_type), builder, block);
        pointer.store(cast, builder, block);
        self.define_variable(declaration, pointer.into_mlir());
    }

    /// Spills the entry block's argument at `argument_index` into a fresh stack slot and binds `declaration` to it.
    pub fn bind_block_argument(
        &mut self,
        declaration: NodeId,
        mlir_type: MlirType<'context>,
        argument_index: usize,
        entry_block: &BlockRef<'context, 'block>,
        builder: &Builder<'context>,
    ) {
        let pointer =
            Pointer::from_argument(Type::new(mlir_type), argument_index, entry_block, builder);
        self.define_variable(declaration, pointer.into_mlir());
    }

    /// Looks up a variable's place by its declaration's `NodeId`, searching from the innermost scope outward.
    pub fn variable(&self, declaration: NodeId) -> Value<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(pointer) = scope.get(&declaration) {
                return *pointer;
            }
        }
        unreachable!("unregistered local variable: {declaration:?}");
    }
}
