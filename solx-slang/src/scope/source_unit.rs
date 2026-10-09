//!
//! The source unit scope: the owned MLIR context that every nested scope emits into.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::Type;

use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::Function;
use solx_mlir::Type as MlirType;

use crate::contract::indirect_callees::IndirectCallees;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::debug_locations::DebugLocations;
use crate::scope::contract::ContractScope;
use crate::r#type::position::Position;

/// The source unit scope: the owned MLIR context that every nested scope emits into.
pub struct SourceUnitScope<'context> {
    /// The owned MLIR context, surrendered by the conversion into it.
    pub mlir: Context<'context>,
    /// What the nodes lowered here resolve to: their locations when debug info was requested for
    /// the segment, the unknown location otherwise.
    pub debug_locations: DebugLocations<'context>,
    /// The mangled symbol and MLIR signature of each function, filled at its first naming.
    pub function_signatures: HashMap<NodeId, Function<'context>>,
    /// The definition ids and data locations of the recursive structs identified so far.
    pub identified_structures: HashSet<(NodeId, solx_utils::DataLocation)>,
    /// The current position.
    pub position: Position,
}

impl<'context> SourceUnitScope<'context> {
    /// Wraps the MLIR context for one segment's emission.
    pub fn new(mlir: Context<'context>, debug_locations: DebugLocations<'context>) -> Self {
        Self {
            mlir,
            debug_locations,
            function_signatures: HashMap::new(),
            identified_structures: HashSet::new(),
            position: Position::ByValue,
        }
    }

    /// Opens the contract scope around `emit`: the `sol.contract` an enclosed member is defined
    /// into, the object whose hierarchy it resolves against and its `storage_layout`, and the
    /// deploy code's indirect callees, which fix the code segment it lowers, with the `this` type
    /// installed on the MLIR context for its duration.
    pub fn contract(
        &mut self,
        contract_type: MlirType<'context>,
        contract: Contract<'context>,
        object: &Object,
        storage_layout: &HashMap<NodeId, StorageSlot>,
        indirect_callees: IndirectCallees<'_>,
        emit: impl FnOnce(&mut ContractScope<'_, 'context>),
    ) {
        self.mlir.current_contract_type = Some(contract_type);
        emit(&mut ContractScope::new(
            self,
            contract,
            object,
            storage_layout,
            indirect_callees,
        ));
        self.mlir.current_contract_type = None;
    }

    /// Runs `resolve` at `position`, restoring the enclosing position afterwards.
    pub fn at_position<R>(
        &mut self,
        position: Position,
        resolve: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let enclosing = std::mem::replace(&mut self.position, position);
        let result = resolve(self);
        self.position = enclosing;
        result
    }

    /// The function's mangled symbol and MLIR signature, computed at its first naming.
    pub fn function_signature(&mut self, function: &FunctionDefinition) -> Function<'context> {
        if let Some(signature) = self.function_signatures.get(&function.node_id()) {
            return signature.clone();
        }
        let Some(Type::Function(function_type)) = function.get_type() else {
            unreachable!("slang types every function definition");
        };
        let signature = Function::new(
            Self::function_symbol(function),
            self.function_type(&function_type),
        );
        self.function_signatures
            .insert(function.node_id(), signature.clone());
        signature
    }
}

impl<'context> Deref for SourceUnitScope<'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.mlir
    }
}

impl<'context> From<SourceUnitScope<'context>> for Context<'context> {
    fn from(scope: SourceUnitScope<'context>) -> Self {
        scope.mlir
    }
}
