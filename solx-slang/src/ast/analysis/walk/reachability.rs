//!
//! Transitive reachability walk over a contract's function bodies.
//!

use std::collections::BTreeMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;

/// A breadth-first worklist over a contract's function bodies, accumulating a deduplicated set of
/// reached functions. The caller drives it: `next_body` yields each body, the caller's own `Visitor`
/// reports what it reaches via `reach`, and `into_reached` returns the result.
pub struct ReachabilityWalk {
    /// Reached functions, deduplicated by node id: the result set. A `BTreeMap` so emission order is
    /// the deterministic ascending-node-id order rather than randomized hash iteration.
    collected: BTreeMap<NodeId, FunctionDefinition>,
    /// Node ids whose bodies have already been handed out by `next_body`.
    walked: HashSet<NodeId>,
    /// Function bodies still to walk.
    to_walk: Vec<FunctionDefinition>,
}

impl ReachabilityWalk {
    /// Seeds the walk with the contract's linearised functions, its constructor, and `extra_roots`
    /// (bodies outside the linearised set, e.g. `super`-reached base overrides).
    pub fn new(contract: &ContractDefinition, extra_roots: &[FunctionDefinition]) -> Self {
        let mut to_walk = contract.linearised_functions();
        if let Some(constructor) = contract.constructor() {
            to_walk.push(constructor);
        }
        to_walk.extend(extra_roots.iter().cloned());
        Self {
            collected: BTreeMap::new(),
            walked: HashSet::new(),
            to_walk,
        }
    }

    /// Pops the next body that has not yet been walked, or `None` once the
    /// worklist is exhausted. Each body is yielded at most once.
    pub fn next_body(&mut self) -> Option<FunctionDefinition> {
        while let Some(function) = self.to_walk.pop() {
            if self.walked.insert(function.node_id()) {
                return Some(function);
            }
        }
        None
    }

    /// Records `function` in the result set. The first time it is seen, its body
    /// is queued so the functions it reaches in turn are walked too.
    pub fn reach(&mut self, function: FunctionDefinition) {
        if self
            .collected
            .insert(function.node_id(), function.clone())
            .is_none()
        {
            self.to_walk.push(function);
        }
    }

    /// Queues `function`'s body to be walked WITHOUT adding it to the result set, to follow the calls
    /// it makes, e.g. a free function emitted elsewhere.
    pub fn enqueue(&mut self, function: FunctionDefinition) {
        if !self.walked.contains(&function.node_id()) {
            self.to_walk.push(function);
        }
    }

    /// Consumes the walk and returns the reached functions, deduplicated by node id and ordered by it,
    /// so emission is reproducible across runs.
    pub fn into_reached(self) -> Vec<FunctionDefinition> {
        self.collected.into_values().collect()
    }
}
