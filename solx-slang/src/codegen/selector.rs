//!
//! EVM function selector computation.
//!

use slang_solidity::backend::ir::ir2_flat_contracts::FunctionDefinition;

use crate::codegen::types::TypeMapper;

/// Computes EVM function selectors from Solidity function signatures.
pub struct SelectorComputer;

impl SelectorComputer {
    /// Computes the 4-byte selector and canonical signature for a function.
    ///
    /// The selector is the first 4 bytes of `keccak256(signature)`.
    pub fn compute(func: &FunctionDefinition) -> ([u8; 4], String) {
        let signature = Self::canonical_signature(func);
        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
        let bytes = hash.to_vec();
        let selector = [bytes[0], bytes[1], bytes[2], bytes[3]];
        (selector, signature)
    }

    /// Computes the 4-byte selector from a pre-built signature string.
    pub fn selector_from_signature(signature: &str) -> [u8; 4] {
        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
        let bytes = hash.to_vec();
        [bytes[0], bytes[1], bytes[2], bytes[3]]
    }

    /// Builds the canonical signature string (e.g. `get()` or `transfer(address,uint256)`).
    fn canonical_signature(func: &FunctionDefinition) -> String {
        let name = func
            .name
            .as_ref()
            .map(|terminal| terminal.text.as_str())
            .unwrap_or("");

        let param_types: Vec<String> = func
            .parameters
            .iter()
            .map(|param| TypeMapper::canonical_type(&param.type_name))
            .collect();

        format!("{name}({})", param_types.join(","))
    }
}
