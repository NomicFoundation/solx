//!
//! Variable environment and loop context for MLIR code generation.
//!

mod loop_target;

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::Value;

pub use self::loop_target::LoopTarget;

/// Tracks variable bindings (alloca'd pointers) and loop break/continue targets.
///
/// Variables are stored as alloca'd pointers on the stack (address space 0).
/// Each variable name maps to the `llvm.alloca` result pointer. Reads produce
/// `llvm.load`, writes produce `llvm.store`.
///
/// Implements lexical scoping: variable lookups search from the innermost
/// scope outward. `enter_scope()` / `exit_scope()` bracket blocks that
/// introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping variable names to alloca'd pointers.
    /// The outermost scope (index 0) holds function parameters.
    scopes: Vec<HashMap<String, Value<'context, 'block>>>,
    /// Names of variables with signed integer types (`int8`..`int256`).
    signed_variables: HashSet<String>,
    /// Stack of (break_block, continue_block) for nested loops.
    loop_targets: Vec<LoopTarget<'context, 'block>>,
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
            signed_variables: HashSet::new(),
            loop_targets: Vec::new(),
        }
    }

    // ---- Scope management ----

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

    // ---- Public &mut self ----

    /// Registers a variable with its alloca'd pointer in the current scope.
    pub fn define_variable(&mut self, name: String, pointer: Value<'context, 'block>) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(name, pointer);
    }

    /// Marks a variable as having a signed integer type.
    pub fn mark_signed(&mut self, name: &str) {
        self.signed_variables.insert(name.to_owned());
    }

    /// Pushes a new loop context for break/continue resolution.
    pub fn push_loop(&mut self, target: LoopTarget<'context, 'block>) {
        self.loop_targets.push(target);
    }

    /// Pops the innermost loop context.
    pub fn pop_loop(&mut self) {
        self.loop_targets.pop();
    }

    // ---- Public &self ----

    /// Returns whether a variable has a signed integer type.
    pub fn is_signed(&self, name: &str) -> bool {
        self.signed_variables.contains(name)
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

    /// Returns the current innermost loop target.
    ///
    /// # Returns None
    ///
    /// Returns `None` if no loop context has been pushed.
    pub fn current_loop(&self) -> Option<&LoopTarget<'context, 'block>> {
        self.loop_targets.last()
    }
}
