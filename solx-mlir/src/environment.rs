//!
//! Variable environment and loop context for MLIR code generation.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use melior::ir::Value;

use crate::LoopTarget;

/// Tracks variable bindings (alloca'd pointers) and loop break/continue targets.
///
/// Variables are stored as alloca'd pointers on the stack (address space 0).
/// Each variable name maps to the `llvm.alloca` result pointer. Reads produce
/// `llvm.load`, writes produce `llvm.store`.
pub struct Environment<'context, 'block> {
    /// Variable name -> alloca'd pointer value.
    variables: HashMap<String, Value<'context, 'block>>,
    /// Names of variables with signed integer types (`int8`..`int256`).
    signed_variables: HashSet<String>,
    /// Stack of (break_block, continue_block) for nested loops.
    loop_targets: Vec<LoopTarget<'context, 'block>>,
}

impl<'context, 'block> Environment<'context, 'block> {
    /// Creates a new empty environment.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            signed_variables: HashSet::new(),
            loop_targets: Vec::new(),
        }
    }

    /// Registers a variable with its alloca'd pointer.
    pub fn define_variable(&mut self, name: String, ptr: Value<'context, 'block>) {
        self.variables.insert(name, ptr);
    }

    /// Marks a variable as having a signed integer type.
    pub fn mark_signed(&mut self, name: &str) {
        self.signed_variables.insert(name.to_owned());
    }

    /// Returns whether a variable has a signed integer type.
    pub fn is_signed(&self, name: &str) -> bool {
        self.signed_variables.contains(name)
    }

    /// Looks up a variable's alloca'd pointer by name.
    pub fn get_variable(&self, name: &str) -> Option<Value<'context, 'block>> {
        self.variables.get(name).copied()
    }

    /// Pushes a new loop context for break/continue resolution.
    pub fn push_loop(&mut self, target: LoopTarget<'context, 'block>) {
        self.loop_targets.push(target);
    }

    /// Pops the innermost loop context.
    pub fn pop_loop(&mut self) {
        self.loop_targets.pop();
    }

    /// Returns the current innermost loop target.
    pub fn current_loop(&self) -> Option<&LoopTarget<'context, 'block>> {
        self.loop_targets.last()
    }
}
