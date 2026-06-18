//!
//! TODO: pure-Slang query pending a home (Slang dev-solx vs solx vs fold) —
//! query-sorting pass. Lifted from the inline maps `AstEmitter::emit_contract` and
//! `ContractEmitter::emit_library` built.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::ContractMember;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::LibraryDefinition;

/// The ABI `method_identifiers` map: externally-dispatchable signature → 4-byte
/// selector (lower-case hex), as solc emits it in the contract / library
/// artifact.
pub trait MethodIdentifiers {
    /// The signature → selector map for this object's externally-callable members.
    fn method_identifiers(&self) -> BTreeMap<String, String>;
}

impl MethodIdentifiers for ContractDefinition {
    fn method_identifiers(&self) -> BTreeMap<String, String> {
        let mut method_identifiers = BTreeMap::new();
        // Walk the C3-linearised function list (inherited + own) so a derived
        // contract exposes its inherited external functions in the ABI — not only
        // the contract's own members.
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
        // A `delegatecall`ed library dispatches only its externally-visible
        // functions; an internal / private function has no selector and is
        // excluded, so iterating every regular function and keeping the ones with
        // a selector yields exactly the deployed dispatch set.
        let mut method_identifiers = BTreeMap::new();
        for member in self.members().iter() {
            let ContractMember::FunctionDefinition(function) = member else {
                continue;
            };
            if !matches!(function.kind(), FunctionKind::Regular) {
                continue;
            }
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
