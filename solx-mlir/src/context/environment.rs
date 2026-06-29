//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::Value as MlirValue;
use slang_solidity_v2::ast::NodeId;

use crate::Context;
use crate::Pointer;
use crate::Type;
use crate::Value;

/// Tracks variable places (alloca'd pointers) for lexical scoping.
///
/// Bindings are keyed by each declaration's Slang `NodeId`, so same-named locals across scopes
/// stay distinct. Lookups search from the innermost scope outward.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping a declaration's `NodeId` to its place (scope 0 holds parameters).
    pub scopes: Vec<HashMap<NodeId, MlirValue<'context, 'block>>>,
    /// Declarations bound directly to an SSA value: a read yields the value with no `sol.load`. Used
    /// inside a `sol.modifier_call_blk`, whose `IsolatedFromAbove` block exposes the wrapping
    /// function's parameters as block-argument values.
    pub value_bindings: HashMap<NodeId, MlirValue<'context, 'block>>,
}

impl<'context, 'block> Environment<'context, 'block> {
    /// Creates a new environment with a single root scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            value_bindings: HashMap::new(),
        }
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
    pub fn define_variable(&mut self, declaration: NodeId, pointer: MlirValue<'context, 'block>) {
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
        value: MlirValue<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) {
        let cast = Value::new(value).cast(Type::new(parameter_type), context, block);
        let pointer = Pointer::stack_slot(Type::new(parameter_type), context, block);
        pointer.store(cast, context, block);
        self.define_variable(declaration, pointer.into_mlir());
    }

    /// Spills the entry block's argument at `argument_index` into a fresh stack slot and binds `declaration` to it.
    pub fn bind_block_argument(
        &mut self,
        declaration: NodeId,
        mlir_type: MlirType<'context>,
        argument_index: usize,
        entry_block: &BlockRef<'context, 'block>,
        context: &Context<'context>,
    ) {
        let pointer =
            Pointer::from_argument(Type::new(mlir_type), argument_index, entry_block, context);
        self.define_variable(declaration, pointer.into_mlir());
    }

    /// Binds `declaration` directly to the SSA `value`: a read returns `value` itself, emitting no
    /// `sol.load`. Used to expose a `sol.modifier_call_blk` block argument as a parameter.
    pub fn bind_value(&mut self, declaration: NodeId, value: MlirValue<'context, 'block>) {
        self.value_bindings.insert(declaration, value);
    }

    /// Looks up a variable's place by its declaration's `NodeId`, searching from the innermost scope outward.
    pub fn variable(&self, declaration: NodeId) -> MlirValue<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(pointer) = scope.get(&declaration) {
                return *pointer;
            }
        }
        unreachable!("unregistered local variable: {declaration:?}");
    }

    /// The SSA value `declaration` is directly bound to, if any (see [`Self::bind_value`]).
    pub fn value_binding(&self, declaration: NodeId) -> Option<MlirValue<'context, 'block>> {
        self.value_bindings.get(&declaration).copied()
    }
}
