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
        // Walk the C3-linearised function list so a derived contract exposes inherited external functions too.
        for function in self.linearised_functions() {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        // Walk the C3-linearised state-variable list so every `public` state
        // variable's auto-generated getter — own or inherited — appears in the ABI.
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

        // A library dispatches only its externally-visible functions; keeping the regular functions
        // with a selector yields exactly the deployed dispatch set.
        let mut method_identifiers = BTreeMap::new();
        for member in self.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            if !matches!(function.kind(), FunctionKind::Regular) {
                continue;
            }
            // Both the signature key and the selector value use the library-aware form (a struct
            // parameter named by scope, a ` storage` suffix) so the published ABI matches the selector
            // the deployed dispatcher and `L.f.selector` resolve to — see `library_aware_signature`.
            let signature =
                library_aware_signature(&function).or_else(|| function.compute_canonical_signature());
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
