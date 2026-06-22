//!
//! The MLIR symbol name a Slang function definition lowers to.
//!

use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;

/// The MLIR symbol name a function definition lowers to — the one naming authority both definitions
/// and call sites route through, so a function and its callers agree on the symbol.
pub trait MlirSymbolName {
    /// The unique MLIR symbol name for this function: an externally-callable function uses Slang's
    /// canonical ABI signature, an internal one its internal signature, the rest their base name.
    fn mlir_function_name(&self) -> String;

    /// The base identifier of this function's MLIR symbol (a synthesised name for fallback / receive / constructor).
    fn mlir_base_name(&self) -> String;

    /// This function's MLIR symbol qualified by its node id, so two free functions of the same
    /// signature (reachable together via an alias) do not collide.
    fn node_id_qualified_symbol(&self) -> String;
}

impl MlirSymbolName for FunctionDefinition {
    fn mlir_function_name(&self) -> String {
        if let Some(AbiEntry::Function(abi_function)) = self.compute_abi_entry() {
            if let Some(signature) = self.compute_canonical_signature() {
                return signature;
            }
            let name = self.mlir_base_name();
            let inputs = abi_function.inputs();
            let types: Vec<&str> = inputs.iter().map(|input| input.type_name()).collect();
            return format!("{name}({})", types.join(","));
        }

        if let Some(signature) = self.compute_internal_signature() {
            return signature;
        }

        format!("{}()", self.mlir_base_name())
    }

    fn mlir_base_name(&self) -> String {
        match self.kind() {
            FunctionKind::Regular => self.name().expect("slang validated").name(),
            FunctionKind::Fallback => "fallback".to_owned(),
            FunctionKind::Receive => "receive".to_owned(),
            FunctionKind::Constructor => "constructor".to_owned(),
            FunctionKind::Modifier => unreachable!("modifiers are not emitted as functions"),
        }
    }

    fn node_id_qualified_symbol(&self) -> String {
        format!("{}#{:?}", self.mlir_function_name(), self.node_id())
    }
}
