//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::NodeId;

use crate::context::variable_binding::VariableBinding;

/// Tracks variable bindings (alloca'd pointers) for lexical scoping.
///
/// Each variable stores the alloca'd pointer and the element type of that
/// pointer (e.g. `ui64` for a `uint64` variable). Reads produce `sol.load`
/// with the declared element type; writes produce `sol.store`.
///
/// Bindings are keyed by the declaration's Slang [`NodeId`], not its textual
/// name, so same-named locals across scopes (shadowing) are distinct by
/// construction and identifier references resolve through the binder
/// (`resolve_to_definition().node_id()`) rather than by name. Lexical scoping is
/// still tracked: lookups search from the innermost scope outward, and
/// `enter_scope()` / `exit_scope()` bracket blocks that introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping a declaration's [`NodeId`] to its
    /// [`VariableBinding`]. The outermost scope (index 0) holds function
    /// parameters.
    scopes: Vec<HashMap<NodeId, VariableBinding<'context, 'block>>>,
}

impl<'context, 'block> Default for Environment<'context, 'block> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'context, 'block> Environment<'context, 'block> {
    /// Creates a new environment with a single root scope.
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    /// Pushes a new lexical scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Pops the innermost lexical scope.
    ///
    /// # Panics
    ///
    /// Panics if called when only the root scope remains.
    pub fn exit_scope(&mut self) {
        assert!(self.scopes.len() > 1, "cannot exit the root scope");
        self.scopes.pop();
    }

    /// Registers a variable, keyed by its declaration's [`NodeId`], with its
    /// alloca'd pointer and element type in the current scope.
    pub fn define_variable(
        &mut self,
        declaration: NodeId,
        pointer: Value<'context, 'block>,
        element_type: Type<'context>,
    ) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(
                declaration,
                VariableBinding {
                    pointer,
                    element_type,
                },
            );
    }

    /// Looks up a variable's alloca'd pointer and element type by its
    /// declaration's [`NodeId`] (from `resolve_to_definition().node_id()`).
    ///
    /// Searches from the innermost scope outward.
    ///
    /// # Panics
    ///
    /// Panics if no binding exists. Slang's semantic pass guarantees every
    /// emitted identifier reference resolves, so a miss here is a solx-internal
    /// invariant failure rather than a user error.
    pub fn variable_with_type(&self, declaration: NodeId) -> VariableBinding<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.get(&declaration) {
                return *binding;
            }
        }
        unreachable!("unregistered local variable: {declaration:?}");
    }
}
