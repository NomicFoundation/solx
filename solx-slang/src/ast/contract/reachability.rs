//!
//! Transitive reachability walk over a contract's function bodies.
//!

use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;

/// A breadth-first worklist that walks a contract's function bodies and
/// accumulates a deduplicated set of reached functions.
///
/// The walk is seeded with the contract's linearised functions, its
/// constructor, and any caller-supplied extra roots. The caller drives it:
/// [`next_body`](Self::next_body) yields each not-yet-walked body in turn (each
/// at most once), the caller runs its own [`Visitor`](slang_solidity_v2::ast::visitor::Visitor)
/// over that body and reports what it reaches through [`reach`](Self::reach),
/// and [`into_reached`](Self::into_reached) returns the accumulated result.
///
/// Owns the BFS scaffolding — the worklist, the walked/collected dedup sets, and
/// the root-seeding; each reachability pass — free functions, library functions,
/// and a planned `super`-dispatch pass — supplies its own per-body Visitor and
/// decides what counts as reached, so they do not each re-implement the walk.
pub struct ReachabilityWalk {
    /// Reached functions, deduplicated by node id — the result set.
    collected: HashMap<NodeId, FunctionDefinition>,
    /// Node ids whose bodies have already been handed out by `next_body`.
    walked: HashSet<NodeId>,
    /// Function bodies still to walk.
    to_walk: Vec<FunctionDefinition>,
}

impl ReachabilityWalk {
    /// Seeds the walk with the contract's linearised functions, its constructor
    /// (not part of the linearised set, yet able to reach functions of its own),
    /// and `extra_roots` — bodies emitted into this contract's module that are
    /// outside the linearised set (e.g. `super`-reached base overrides).
    pub fn new(contract: &ContractDefinition, extra_roots: &[FunctionDefinition]) -> Self {
        let mut to_walk = contract.linearised_functions();
        if let Some(constructor) = contract.constructor() {
            to_walk.push(constructor);
        }
        to_walk.extend(extra_roots.iter().cloned());
        Self {
            collected: HashMap::new(),
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

    /// Whether `node_id` is already in the reached result set. Lets a caller
    /// decide one-time bookkeeping (e.g. marking a newly-reached function) before
    /// it calls [`reach`](Self::reach), which is idempotent.
    pub fn is_collected(&self, node_id: NodeId) -> bool {
        self.collected.contains_key(&node_id)
    }

    /// Queues `function`'s body to be walked without adding it to the result set
    /// — for a function reached only to follow the calls *it* makes (e.g. a free
    /// function emitted elsewhere). A no-op if the body was already walked.
    pub fn enqueue(&mut self, function: FunctionDefinition) {
        if !self.walked.contains(&function.node_id()) {
            self.to_walk.push(function);
        }
    }

    /// Consumes the walk and returns the reached functions, deduplicated by
    /// node id.
    pub fn into_reached(self) -> Vec<FunctionDefinition> {
        self.collected.into_values().collect()
    }
}
