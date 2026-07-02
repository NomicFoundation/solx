//!
//! Variable environment for MLIR code generation.
//!

use std::collections::HashMap;

use melior::ir::Type;
use melior::ir::Value;

/// Tracks variable bindings (alloca'd pointers) for lexical scoping.
///
/// Each variable stores the alloca'd pointer and the element type of that
/// pointer (e.g. `ui64` for a `uint64` variable). Reads produce `sol.load`
/// with the declared element type; writes produce `sol.store`.
///
/// Implements lexical scoping: variable lookups search from the innermost
/// scope outward. `enter_scope()` / `exit_scope()` bracket blocks that
/// introduce new variables.
pub struct Environment<'context, 'block> {
    /// Stack of scopes, each mapping variable names to `(pointer, element_type)`.
    /// The outermost scope (index 0) holds function parameters.
    scopes: Vec<HashMap<String, (Value<'context, 'block>, Type<'context>)>>,
    /// Names bound directly to an SSA value: a read yields the value with no `sol.load`. Used inside a
    /// `sol.modifier_call_blk`, whose `IsolatedFromAbove` block exposes the wrapping function's
    /// parameters as block-argument values rather than stack slots.
    value_bindings: HashMap<String, (Value<'context, 'block>, Type<'context>)>,
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
            value_bindings: HashMap::new(),
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

    /// Registers a variable with its alloca'd pointer and element type in the current scope.
    pub fn define_variable(
        &mut self,
        name: String,
        pointer: Value<'context, 'block>,
        element_type: Type<'context>,
    ) {
        self.scopes
            .last_mut()
            .expect("at least one scope exists")
            .insert(name, (pointer, element_type));
    }

    /// Looks up a variable's alloca'd pointer and element type by name.
    ///
    /// Searches from the innermost scope outward.
    ///
    /// # Panics
    ///
    /// Panics if no binding exists. Slang's semantic pass guarantees every
    /// emitted identifier reference resolves, so a miss here is a solx-internal
    /// invariant failure rather than a user error.
    pub fn variable_with_type(&self, name: &str) -> (Value<'context, 'block>, Type<'context>) {
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return *entry;
            }
        }
        unreachable!("unregistered local variable: {name}");
    }

    /// Binds `name` directly to the SSA `value` of `element_type`: a read returns `value` itself,
    /// emitting no `sol.load`. Used to expose a `sol.modifier_call_blk` block argument as a parameter.
    pub fn bind_value(
        &mut self,
        name: String,
        value: Value<'context, 'block>,
        element_type: Type<'context>,
    ) {
        self.value_bindings.insert(name, (value, element_type));
    }

    /// The SSA value `name` is directly bound to and its element type, if any (see [`Self::bind_value`]).
    pub fn value_binding(&self, name: &str) -> Option<(Value<'context, 'block>, Type<'context>)> {
        self.value_bindings.get(name).copied()
    }
}
