//!
//! The deployable object a module emits: a contract or a library.
//!

use std::collections::HashMap;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StateVariableDefinition;

use solx_mlir::ContractKind;

use crate::contract::storage_slot::StorageSlot;

/// The deployable object a module emits, each variant carrying the definition its kind
/// dispatches from.
pub enum Object {
    /// A contract: storage, a constructor, selector dispatch.
    Contract(ContractDefinition),
    /// A library: no storage, no constructor, `DELEGATECALL` dispatch.
    Library(LibraryDefinition),
}

impl Object {
    /// Classifies the definition a name resolves to, admitting the two that deploy an object.
    pub fn from_definition(definition: Definition) -> Option<Self> {
        match definition {
            Definition::Contract(contract) => Some(Self::Contract(contract)),
            Definition::Library(library) => Some(Self::Library(library)),
            _ => None,
        }
    }

    /// The object's name.
    pub fn name(&self) -> Identifier {
        match self {
            Self::Contract(node) => node.name(),
            Self::Library(node) => node.name(),
        }
    }

    /// The object's identifier, qualified by its file: linking keys objects by it, and two files
    /// may declare the same name.
    pub fn identifier(&self) -> String {
        let file_id = match self {
            Self::Contract(node) => node.get_file_id(),
            Self::Library(node) => node.get_file_id(),
        };
        solx_utils::ContractName::full_path(file_id.as_str(), self.name().name())
    }

    /// The kind the object's `sol.contract` declares.
    pub fn kind(&self) -> ContractKind {
        match self {
            Self::Contract(_) => ContractKind::Contract,
            Self::Library(_) => ContractKind::Library,
        }
    }

    /// Whether the object accepts value: a library is reached by `DELEGATECALL` alone, which
    /// carries the caller's.
    pub fn is_payable(&self) -> bool {
        match self {
            Self::Contract(node) => node.is_payable(),
            Self::Library(_) => false,
        }
    }

    /// The contracts of the object's linearisation, itself first: an interface base declares
    /// nothing an object runs, and a library is in no linearisation.
    pub fn contracts(&self) -> Vec<ContractDefinition> {
        match self {
            Self::Contract(node) => node
                .linearised_bases()
                .into_iter()
                .filter_map(|base| match base {
                    ContractBase::Contract(base) => Some(base),
                    ContractBase::Interface(_) => None,
                })
                .collect(),
            Self::Library(_) => Vec::new(),
        }
    }

    /// The functions the object dispatches and defines: a contract's resolved hierarchy, where an
    /// overridden or getter-shadowed function has given way to its override, listed in declaration
    /// order per contract of `contracts`, its linearisation with itself first, as print-init emits
    /// them; a library's own.
    pub fn functions(&self, contracts: &[ContractDefinition]) -> Vec<FunctionDefinition> {
        match self {
            Self::Contract(node) => {
                let resolved = node.linearised_functions();
                contracts
                    .iter()
                    .flat_map(|base| base.functions())
                    .filter(|function| {
                        resolved
                            .iter()
                            .any(|resolved| resolved.node_id() == function.node_id())
                    })
                    .collect()
            }
            Self::Library(node) => node.functions(),
        }
    }

    /// The state variables the object declares over its hierarchy, in storage order.
    pub fn state_variables(&self) -> Vec<StateVariableDefinition> {
        match self {
            Self::Contract(node) => node.linearised_state_variables(),
            Self::Library(node) => node.state_variables(),
        }
    }

    /// The storage slot of each state variable the object stores, persistent and transient in one
    /// map keyed by definition id. A library declares only constants, which occupy no slot.
    pub fn storage_layout(&self) -> HashMap<NodeId, StorageSlot> {
        match self {
            Self::Contract(node) => {
                let abi = node
                    .compute_abi()
                    .expect("slang admits a contract whose ABI it cannot compute");
                abi.storage_layout()
                    .iter()
                    .chain(abi.transient_storage_layout().iter())
                    .map(|item| (item.node_id(), StorageSlot::from(item)))
                    .collect()
            }
            Self::Library(_) => HashMap::new(),
        }
    }
}
