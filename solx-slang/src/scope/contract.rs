//!
//! The contract scope: the enclosing source unit scope, the `sol.contract` the members are
//! defined into, and the object whose hierarchy a member resolves against.
//!

use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::Block;
use solx_mlir::Context;
use solx_mlir::Contract;
use solx_mlir::Type as MlirType;

use crate::contract::constructor_chain::ConstructorChain;
use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The contract scope: the enclosing source unit scope, the `sol.contract` the members are
/// defined into, and the object whose hierarchy a member resolves against.
pub struct ContractScope<'source_unit, 'context> {
    /// The source unit scope this contract is lowered within.
    pub source_unit: &'source_unit mut SourceUnitScope<'context>,
    /// The `sol.contract` the members are declared and defined into.
    pub contract: Contract<'context>,
    /// The object being emitted, whose linearisation resolves the references its bodies make.
    pub object: Object,
    /// The functions the object dispatches: the resolved set of its hierarchy.
    pub functions: Vec<FunctionDefinition>,
    /// The state variables the object stores, in storage order over its hierarchy.
    pub state_variables: Vec<StateVariableDefinition>,
    /// The state-variable slots keyed by definition id.
    pub storage_layout: HashMap<NodeId, StorageSlot>,
    /// The constructors the object's creation runs and the argument lists they pass along.
    pub chain: ConstructorChain,
    /// The definition ids of the functions defined so far.
    pub defined_functions: HashSet<NodeId>,
}

/// How a function reference is looked up in the object, by the reference's shape.
pub enum Lookup {
    /// The declaration itself: a contract-qualified, free or library function.
    Declared,
    /// A bare name: the most-derived override the object dispatches for a virtual function.
    Virtual,
    /// A `super` member written in the anchoring contract: the nearest implemented override after
    /// the anchor in the object's linearisation.
    Super(ContractDefinition),
}

impl<'source_unit, 'context> ContractScope<'source_unit, 'context> {
    /// Opens a contract scope within `source_unit`.
    pub fn new(
        source_unit: &'source_unit mut SourceUnitScope<'context>,
        contract: Contract<'context>,
        object: Object,
    ) -> Self {
        let contracts = object.contracts();
        let functions = object.functions(&contracts);
        let state_variables = object.state_variables();
        let storage_layout = object.storage_layout();
        let chain = ConstructorChain::new(&contracts);
        Self {
            source_unit,
            contract,
            object,
            functions,
            state_variables,
            storage_layout,
            chain,
            defined_functions: HashSet::new(),
        }
    }

    /// Opens the function scope around `emit`: whether the frame is a constructor's, a fresh
    /// variable environment, the declared return types a `return` converts to, and checked
    /// arithmetic, with the MLIR cursor on `entry` for the body's duration.
    pub fn function(
        &mut self,
        entry: Block<'context>,
        in_constructor: bool,
        return_types: Vec<MlirType<'context>>,
        emit: impl FnOnce(&mut FunctionScope<'_, '_, 'context>),
    ) {
        let enclosing = self.source_unit.mlir.current_block.replace(entry);
        emit(&mut FunctionScope::new(self, in_constructor, return_types));
        self.source_unit.mlir.current_block = enclosing;
    }

    /// The function a reference to `function` runs in this object, looked up as `lookup` says. A
    /// library's functions are never overridden, and `super` is written in a contract alone.
    pub fn resolve(&self, function: &FunctionDefinition, lookup: &Lookup) -> FunctionDefinition {
        match (&self.object, lookup) {
            (_, Lookup::Declared) => function.clone(),
            (Object::Contract(node), Lookup::Virtual) => node.resolve_virtual(function),
            (Object::Contract(node), Lookup::Super(anchor)) => node.resolve_super(function, anchor),
            (Object::Library(_), Lookup::Virtual) => function.clone(),
            (Object::Library(_), Lookup::Super(_)) => {
                unreachable!("`super` is written in a contract alone")
            }
        }
    }
}

impl<'source_unit, 'context> Deref for ContractScope<'source_unit, 'context> {
    type Target = Context<'context>;

    fn deref(&self) -> &Self::Target {
        &self.source_unit.mlir
    }
}
