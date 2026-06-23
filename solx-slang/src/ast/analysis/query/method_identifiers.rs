//!
//! ABI `method_identifiers` map query (pure-Slang).
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;

/// The ABI `method_identifiers` map: externally-dispatchable signature → 4-byte selector (lower-case hex).
pub trait MethodIdentifiers {
    /// The signature → selector map for this object's externally-callable members.
    fn method_identifiers(&self) -> BTreeMap<String, String>;
}

impl MethodIdentifiers for ContractDefinition {
    fn method_identifiers(&self) -> BTreeMap<String, String> {
        let mut method_identifiers = BTreeMap::new();
        for member in self.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        method_identifiers
    }
}
