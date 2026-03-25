//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::Value;

/// Tracks variable bindings (alloca'd pointers) for lexical scoping.
///
/// Variables are stored as alloca'd pointers on the stack.
/// Each variable name maps to the `sol.alloca` result pointer. Reads produce
/// `sol.load`, writes produce `sol.store`.
///
/// Implements lexical scoping: variable lookups search from the innermost
/// scope outward. `enter_scope()` / `exit_scope()` bracket blocks that
/// introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping variable names to alloca'd pointers.
    /// The outermost scope (index 0) holds function parameters.
    scopes: Vec<HashMap<String, Value<'context, 'block>>>,
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

    /// Registers a variable with its alloca'd pointer in the current scope.
    pub fn define_variable(&mut self, name: String, pointer: Value<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(name, pointer);
    }

    /// Looks up a variable's alloca'd pointer by name.
    ///
    /// Searches from the innermost scope outward.
    ///
    /// # Returns None
    ///
    /// Returns `None` if no variable with the given name has been defined
    /// in any enclosing scope.
    pub fn variable(&self, name: &str) -> Option<Value<'context, 'block>> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(*value);
            }
        }
        None
    }
}
