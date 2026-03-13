//!
//! EVM function selector computation.
//!

use slang_solidity::backend::ir::ast::FunctionDefinition;

use crate::codegen::types::TypeMapper;

/// Computes EVM function selectors from Solidity function signatures.
pub struct SelectorComputer;

impl SelectorComputer {
    /// Computes the 4-byte selector and canonical signature for a function.
    ///
    /// The selector is the first 4 bytes of `keccak256(signature)`.
    pub(crate) fn compute(function: &FunctionDefinition) -> ([u8; 4], String) {
        let signature = Self::canonical_signature(function);
        let selector = Self::selector_from_signature(&signature);
        (selector, signature)
    }

    /// Computes the 4-byte selector from a pre-built signature string.
    pub(crate) fn selector_from_signature(signature: &str) -> [u8; 4] {
        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
        let bytes = hash.to_vec();
        bytes[..4]
            .try_into()
            .expect("keccak256 always produces at least 4 bytes")
    }

    /// Builds the canonical signature string (e.g. `get()` or `transfer(address,uint256)`).
    fn canonical_signature(function: &FunctionDefinition) -> String {
        let name = function
            .name()
            .map(|id| id.name())
            .unwrap_or_default();

        let parameter_types: Vec<String> = function
            .parameters()
            .iter()
            .map(|parameter| TypeMapper::canonical_type(&parameter.type_name()))
            .collect();

        format!("{name}({})", parameter_types.join(","))
    }
}
