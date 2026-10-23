//!
//! The segment a contract scope lowers, with the state only that segment keeps.
//!

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;

use solx_mlir::Value;
use solx_utils::CodeSegment;

use crate::contract::constructor::ConstructorBuilder;
use crate::scope::contract::ContractScope;

/// The segment a contract scope lowers. The deploy code takes function pointers and runs the
/// constructor chain, and the runtime code defines the functions a pointer it stored can reach.
pub enum Segment<'source_unit, 'context> {
    /// The deploy code.
    Deploy {
        /// The functions whose pointer the deploy code takes, keyed by definition id.
        indirect_callees: &'source_unit mut BTreeMap<NodeId, FunctionDefinition>,
        /// Mutable state for emitting the object's constructor chain.
        constructor: ConstructorBuilder<'context>,
    },
    /// The runtime code.
    Runtime {
        /// The functions whose pointer the deploy code took, keyed by definition id.
        indirect_callees: &'source_unit BTreeMap<NodeId, FunctionDefinition>,
    },
}

impl<'source_unit, 'context> Segment<'source_unit, 'context> {
    /// The code segment, without its state.
    pub fn code_segment(&self) -> CodeSegment {
        match self {
            Self::Deploy { .. } => CodeSegment::Deploy,
            Self::Runtime { .. } => CodeSegment::Runtime,
        }
    }

    /// Records `function`, whose pointer the code takes. Only the deploy code records, since only
    /// the runtime code reads them.
    pub fn record_deploy_indirect_callee(&mut self, function: &FunctionDefinition) {
        if let Self::Deploy {
            indirect_callees, ..
        } = self
        {
            indirect_callees
                .entry(function.node_id())
                .or_insert_with(|| function.clone());
        }
    }

    /// The functions a pointer the deploy code stored can reach, which the runtime `sol.contract`
    /// lists.
    pub fn runtime_indirect_callees(
        &self,
    ) -> impl Iterator<Item = &'source_unit FunctionDefinition> {
        match *self {
            Self::Deploy { .. } => None,
            Self::Runtime { indirect_callees } => Some(indirect_callees.values()),
        }
        .into_iter()
        .flatten()
    }

    /// The deploy code's constructor chain.
    pub fn constructor(&mut self) -> &mut ConstructorBuilder<'context> {
        let Self::Deploy { constructor, .. } = self else {
            unreachable!("only the deploy code emits a constructor");
        };
        constructor
    }
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// The internal pointer to `function` (`sol.func_constant`), defining the function at its first
    /// naming and recording it among the deploy code's indirect callees.
    pub fn pointer_constant(&mut self, function: &FunctionDefinition) -> Value<'context> {
        let signature = self.function_definition(function);
        self.segment.record_deploy_indirect_callee(function);
        signature.pointer_constant(self)
    }

    /// The identifier a pointer to `function` dispatches to, numbered from one in the order the
    /// object first defines its functions, so that no source outside the object changes its
    /// bytecode.
    pub fn function_identifier(&mut self, function: &FunctionDefinition) -> NonZeroU64 {
        let next = NonZeroU64::MIN.saturating_add(self.function_identifiers.len() as u64);
        *self
            .function_identifiers
            .entry(function.node_id())
            .or_insert(next)
    }
}
