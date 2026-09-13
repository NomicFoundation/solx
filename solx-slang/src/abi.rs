//!
//! The ABI outputs of a definition in standard-JSON form: the JSON ABI Slang computes and
//! serializes in solc's spelling, and the `evm.methodIdentifiers` map.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::abi::ContractAbi;
use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::InterfaceDefinition;
use slang_solidity_v2::ast::LibraryDefinition;
use slang_solidity_v2::ast::StateVariableDefinition;

/// The `abi` field of a standard-JSON contract.
pub struct Abi(ContractAbi);

impl Abi {
    /// The value the standard-JSON output stores: Slang's entries in solc's JSON-ABI spelling.
    pub fn into_value(self) -> serde_json::Value {
        serde_json::to_value(self.0.entries()).expect("the ABI only holds strings and booleans")
    }
}

impl From<&ContractDefinition> for Abi {
    fn from(contract: &ContractDefinition) -> Self {
        Self(
            contract
                .compute_abi()
                .expect("slang admits a contract whose ABI it cannot compute"),
        )
    }
}

impl From<&InterfaceDefinition> for Abi {
    fn from(interface: &InterfaceDefinition) -> Self {
        Self(
            interface
                .compute_abi()
                .expect("slang admits an interface whose ABI it cannot compute"),
        )
    }
}

impl From<&LibraryDefinition> for Abi {
    fn from(library: &LibraryDefinition) -> Self {
        Self(
            library
                .compute_abi()
                .expect("slang admits a library whose ABI it cannot compute"),
        )
    }
}

/// The `evm.methodIdentifiers` map: each externally dispatchable function keyed by the signature
/// its selector hashes, each public state variable by its canonical one, selectors in lower-case
/// hex.
pub struct MethodIdentifiers(BTreeMap<String, String>);

impl MethodIdentifiers {
    /// Maps the given functions and state variables; a function that is not a regular external
    /// one, or a state variable that is not public, has no identifier and is skipped.
    pub fn new(
        functions: impl IntoIterator<Item = FunctionDefinition>,
        state_variables: impl IntoIterator<Item = StateVariableDefinition>,
    ) -> Self {
        Self(
            functions
                .into_iter()
                .filter(|function| {
                    matches!(function.kind(), FunctionKind::Regular)
                        && function.is_externally_visible()
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
                .collect(),
        )
    }

    /// The map the standard-JSON output stores.
    pub fn into_map(self) -> BTreeMap<String, String> {
        self.0
    }
}

/// The whole hierarchy, as the ABI lists it.
impl From<&ContractDefinition> for MethodIdentifiers {
    fn from(contract: &ContractDefinition) -> Self {
        Self::new(
            contract.linearised_functions(),
            contract.linearised_state_variables(),
        )
    }
}

impl From<&InterfaceDefinition> for MethodIdentifiers {
    fn from(interface: &InterfaceDefinition) -> Self {
        Self::new(
            interface
                .members()
                .iter()
                .filter_map(|member| match member {
                    ContractMember::FunctionDefinition(function) => Some(function),
                    _ => None,
                }),
            std::iter::empty(),
        )
    }
}
