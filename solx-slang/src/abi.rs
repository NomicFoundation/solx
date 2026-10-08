//!
//! The ABI outputs of a definition in standard-JSON form: the JSON ABI Slang computes and
//! serializes in solc's spelling, and the `evm.methodIdentifiers` map.
//!

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Range;

use slang_solidity_v2::abi::ContractAbi;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::Identifier;
use slang_solidity_v2::ast::InterfaceDefinition;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::compilation::FileId;

use crate::contract::object::Object;
use crate::contract::storage_slot::StorageSlot;

/// The `abi` field of a standard-JSON contract.
pub struct Abi(ContractAbi);

impl Abi {
    /// The storage slot of each state variable the definition stores, persistent and transient in
    /// one map keyed by definition id.
    pub fn storage_layout(&self) -> HashMap<NodeId, StorageSlot> {
        self.0
            .storage_layout()
            .iter()
            .chain(self.0.transient_storage_layout().iter())
            .map(|item| (item.node_id(), StorageSlot::from(item)))
            .collect()
    }

    /// The value the standard-JSON output stores: Slang's entries in solc's JSON-ABI spelling.
    pub fn to_value(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).expect("Slang's ABI serializer writes only string-keyed maps")
    }
}

/// A source-unit member that has an ABI, whether or not it deploys an object.
pub enum AbiDefinition {
    /// A contract, abstract or not.
    Contract(ContractDefinition),
    /// An interface.
    Interface(InterfaceDefinition),
    /// A library.
    Library(LibraryDefinition),
}

impl AbiDefinition {
    /// The definition `member` declares, or `None` for a member with no ABI.
    pub fn from_member(member: &SourceUnitMember) -> Option<Self> {
        match member {
            SourceUnitMember::ContractDefinition(node) => Some(Self::Contract(node.clone())),
            SourceUnitMember::InterfaceDefinition(node) => Some(Self::Interface(node.clone())),
            SourceUnitMember::LibraryDefinition(node) => Some(Self::Library(node.clone())),
            _ => None,
        }
    }

    /// The definition's name.
    pub fn name(&self) -> Identifier {
        match self {
            Self::Contract(node) => node.name(),
            Self::Interface(node) => node.name(),
            Self::Library(node) => node.name(),
        }
    }

    /// The file declaring the definition and the byte range it spans there.
    pub fn source_range(&self) -> (&FileId, &Range<usize>) {
        match self {
            Self::Contract(node) => (node.get_file_id(), node.get_text_range()),
            Self::Interface(node) => (node.get_file_id(), node.get_text_range()),
            Self::Library(node) => (node.get_file_id(), node.get_text_range()),
        }
    }

    /// The definition's JSON ABI, or `None` for an input Slang accepts without diagnosing but
    /// cannot answer, such as an interface with a contract base.
    pub fn abi(&self) -> Option<Abi> {
        match self {
            Self::Contract(node) => node.compute_abi(),
            Self::Interface(node) => node.compute_abi(),
            Self::Library(node) => node.compute_abi(),
        }
        .map(Abi)
    }

    /// The `evm.methodIdentifiers` map: each externally dispatchable function of the hierarchy
    /// keyed by the signature its selector hashes, each public state variable by its canonical
    /// one, selectors in lower-case hex. `convert-sol-to-yul` builds the entry-point dispatcher
    /// from the same selectors.
    pub fn method_identifiers(&self) -> BTreeMap<String, String> {
        match self {
            Self::Contract(node) => Self::method_identifiers_over(
                node.linearised_functions(),
                node.linearised_state_variables(),
            ),
            Self::Interface(node) => {
                Self::method_identifiers_over(node.linearised_functions(), Vec::new())
            }
            Self::Library(node) => {
                Self::method_identifiers_over(node.functions(), node.state_variables())
            }
        }
    }

    /// The object the definition deploys, holding `abi`, or `None` for an interface or an
    /// abstract contract, which deploy nothing.
    pub fn into_object(self, abi: Abi) -> Option<Object> {
        match self {
            Self::Contract(node) if node.is_abstract() => None,
            Self::Contract(node) => Some(Object::Contract(node, abi)),
            Self::Interface(_) => None,
            Self::Library(node) => Some(Object::Library(node, abi)),
        }
    }

    /// A function that is not a regular, externally visible one, or a state variable that is not
    /// public, has no selector and is skipped.
    fn method_identifiers_over(
        functions: Vec<FunctionDefinition>,
        state_variables: Vec<StateVariableDefinition>,
    ) -> BTreeMap<String, String> {
        functions
            .into_iter()
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
                state_variables
                    .into_iter()
                    .filter(StateVariableDefinition::is_externally_visible)
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
}
