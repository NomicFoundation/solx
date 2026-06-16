//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::Value;
use slang_solidity_v2::ast::NodeId;

/// Tracks variable places (alloca'd pointers) for lexical scoping.
///
/// Each variable maps to the place holding it — a `!sol.ptr<T, Stack>` for a
/// Solidity local, parameter, or named return, an `!llvm.ptr` for a Yul
/// inline-assembly local. A Solidity read reconstructs the [`Pointer`] from the
/// place and loads its `pointee()`; a Yul read reinterprets the place to an
/// `!llvm.ptr`. The element type is the place's own pointee, so it is not stored
/// separately.
///
/// [`Pointer`]: crate::Pointer
///
/// Bindings are keyed by the declaration's Slang [`NodeId`], not its textual
/// name, so same-named locals across scopes (shadowing) are distinct by
/// construction and identifier references resolve through the binder
/// (`resolve_to_definition().node_id()`) rather than by name. Lexical scoping is
/// still tracked: lookups search from the innermost scope outward, and
/// `enter_scope()` / `exit_scope()` bracket blocks that introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping a declaration's [`NodeId`] to its place.
    /// The outermost scope (index 0) holds function parameters.
    scopes: Vec<HashMap<NodeId, Value<'context, 'block>>>,
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
    pub fn exit_scope(&mut self) {
        self.scopes.pop();
    }

    /// Registers a variable, keyed by its declaration's [`NodeId`], with its
    /// place (alloca'd pointer) in the current scope.
    pub fn define_variable(&mut self, declaration: NodeId, pointer: Value<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(declaration, pointer);
    }

    /// Looks up a variable's place by its declaration's [`NodeId`] (from
    /// `resolve_to_definition().node_id()`).
    ///
    /// Searches from the innermost scope outward.
    pub fn variable(&self, declaration: NodeId) -> Value<'context, 'block> {
        for scope in self.scopes.iter().rev() {
            if let Some(pointer) = scope.get(&declaration) {
                return *pointer;
            }
        }
        unreachable!("unregistered local variable: {declaration:?}");
    }
}
