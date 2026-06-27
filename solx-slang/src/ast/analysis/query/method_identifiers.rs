//!
//! ABI `method_identifiers` map query (pure-Slang).
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::LibraryDefinition;

/// The ABI `method_identifiers` map: externally-dispatchable signature → 4-byte selector (lower-case hex).
pub trait MethodIdentifiers {
    /// The signature → selector map for this object's externally-callable members.
    fn method_identifiers(&self) -> BTreeMap<String, String>;
}

impl MethodIdentifiers for ContractDefinition {
    fn method_identifiers(&self) -> BTreeMap<String, String> {
        let mut method_identifiers = BTreeMap::new();
        for function in self.linearised_functions() {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        for state_variable in self.linearised_state_variables() {
            let Some(signature) = state_variable.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = state_variable.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        method_identifiers
    }
}

impl MethodIdentifiers for LibraryDefinition {
    fn method_identifiers(&self) -> BTreeMap<String, String> {
        use crate::ast::contract::function::signature::library_aware_selector;
        use crate::ast::contract::function::signature::library_aware_signature;

        let mut method_identifiers = BTreeMap::new();
        for member in self.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            if !matches!(function.kind(), FunctionKind::Regular) {
                continue;
            }
            let signature = library_aware_signature(&function)
                .or_else(|| function.compute_canonical_signature());
            let Some(signature) = signature else {
                continue;
            };
            let Some(selector) = library_aware_selector(&function) else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        method_identifiers
    }
}
