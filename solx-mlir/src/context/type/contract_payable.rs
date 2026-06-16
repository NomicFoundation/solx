//!
//! Contract payability: whether a contract accepts a plain ETH transfer.
//!

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::FunctionMutability;

/// Whether a contract is payable — the projection a `SlangType::Contract`
/// resolves through ([`Type::resolve`]) and the `sol.contract` op carries.
///
/// [`Type::resolve`]: super::Type
pub trait ContractPayable {
    /// Whether this contract is payable: it declares a `receive()` function or a
    /// `payable` `fallback()`. The single source of truth for payability, read
    /// both when emitting `sol.contract` and when resolving a contract type.
    fn is_payable(&self) -> bool;
}

impl ContractPayable for ContractDefinition {
    // TODO: walk the inheritance tree like solc does (`receiveFunction` /
    // `fallbackFunction` on `ContractDefinition`, `ContractType::isPayable`)
    // and move this into Slang as `ContractDefinition::is_payable()`.
    fn is_payable(&self) -> bool {
        self.functions().iter().any(|function| {
            matches!(function.kind(), FunctionKind::Receive)
                || (matches!(function.kind(), FunctionKind::Fallback)
                    && matches!(function.mutability(), FunctionMutability::Payable))
        })
    }
}
