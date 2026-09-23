//!
//! The deployable object a module emits: a contract or a library.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use slang_solidity_v2::ast::ContractBase;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
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

    /// A contract's functions after resolving overrides and getter shadowing, in declaration
    /// order within each contract of its linearisation, most derived first; a library's own
    /// functions in declaration order.
    pub fn functions(&self) -> Vec<FunctionDefinition> {
        match self {
            Self::Contract(node) => {
                let resolved: HashSet<NodeId> = node
                    .linearised_functions()
                    .iter()
                    .map(|function| function.node_id())
                    .collect();
                self.contracts()
                    .iter()
                    .flat_map(|base| base.functions())
                    .filter(|function| resolved.contains(&function.node_id()))
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

    /// The ABI `method_identifiers` map (externally dispatchable signature to 4-byte selector,
    /// lower-case hex): each function keyed by the signature its selector hashes, each public
    /// state variable by its canonical one. `convert-sol-to-yul` builds the entry-point
    /// dispatcher from the function selectors.
    pub fn method_identifiers(&self) -> BTreeMap<String, String> {
        self.functions()
            .iter()
            .filter(|function| {
                matches!(function.kind(), FunctionKind::Regular) && function.is_externally_visible()
            })
            .map(|function| {
                (
                    function
                        .compute_selector_signature()
                        .expect("an externally visible function has a selector signature"),
                    function
                        .compute_selector()
                        .expect("an externally visible function has a selector"),
                )
            })
            .chain(
                self.state_variables()
                    .iter()
                    .filter(|state_variable| state_variable.is_externally_visible())
                    .map(|state_variable| {
                        (
                            state_variable
                                .compute_canonical_signature()
                                .expect("a public state variable has a canonical signature"),
                            state_variable
                                .compute_selector()
                                .expect("a public state variable has a selector"),
                        )
                    }),
            )
            .map(|(signature, selector)| (signature, format!("{selector:08x}")))
            .collect()
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
