//!
//! The functions whose pointer the deploy code takes, which the runtime code can reach.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Function;
use solx_mlir::Value;
use solx_utils::CodeSegment;

use crate::scope::contract::ContractScope;

/// The functions whose pointer the deploy code takes, keyed by definition id. A pointer the deploy
/// code stored can reach them from the runtime code.
pub enum IndirectCallees<'source_unit> {
    /// Recorded as the deploy code takes their pointers.
    Taken(&'source_unit mut BTreeMap<NodeId, FunctionDefinition>),
    /// Defined by the runtime code.
    Reachable(&'source_unit BTreeMap<NodeId, FunctionDefinition>),
}

impl<'source_unit> IndirectCallees<'source_unit> {
    /// The code segment the callees are seen from: the deploy code takes their pointers, the
    /// runtime code defines them.
    pub fn segment(&self) -> CodeSegment {
        match self {
            Self::Taken(_) => CodeSegment::Deploy,
            Self::Reachable(_) => CodeSegment::Runtime,
        }
    }

    /// Records `function`, whose pointer the code takes. Only the deploy code records, since only
    /// the runtime code reads them.
    pub fn take(&mut self, function: &FunctionDefinition) {
        if let Self::Taken(taken) = self {
            taken
                .entry(function.node_id())
                .or_insert_with(|| function.clone());
        }
    }

    /// The functions a pointer the deploy code stored can reach, none while the deploy code takes
    /// them. The iterator borrows the deploy code's map alone, so the scope stays free to define
    /// them.
    pub fn reachable(
        &self,
    ) -> impl Iterator<Item = &'source_unit FunctionDefinition> + use<'source_unit> {
        match *self {
            Self::Taken(_) => None,
            Self::Reachable(reachable) => Some(reachable.values()),
        }
        .into_iter()
        .flatten()
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// The internal pointer to `function` (`sol.func_constant`), defining the function at its first
    /// naming and recording it among the taken indirect callees.
    pub fn pointer_constant(&mut self, function: &FunctionDefinition) -> Value<'context> {
        let signature = self.function_definition(function);
        self.indirect_callees.take(function);
        signature.pointer_constant(self)
    }

    /// Defines the reachable indirect callees and lists them on the contract.
    pub fn define_indirect_callees(&mut self) {
        let functions: Vec<Function<'context>> = self
            .indirect_callees
            .reachable()
            .map(|function| self.function_definition(function))
            .collect();
        self.contract.set_indirect_callees(&functions, self);
    }
}
