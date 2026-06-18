//!
//! The MLIR symbol name a Slang function definition lowers to.
//!

use slang_solidity_v2::abi::AbiEntry;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;

/// The MLIR symbol name a function definition lowers to. Every `sol.func`
/// definition and every internal call site routes through this trait, so a
/// function's defining symbol and the symbol its callers reference stay
/// identical — the projection has one naming authority, not a per-site format.
pub trait MlirSymbolName {
    /// The unique MLIR symbol name for this function.
    ///
    /// Externally-callable functions use Slang's canonical ABI signature (a
    /// struct parameter expands to its component tuple, so overloads taking
    /// different structs do not collapse onto one symbol); internal/private
    /// functions use Slang's internal signature. Constructor / fallback /
    /// receive have neither — not callable by name, so the base name alone is
    /// unique.
    fn mlir_function_name(&self) -> String;

    /// The base identifier of this function's MLIR symbol, synthesising a name
    /// for the special functions (fallback, receive, constructor) that have no
    /// Solidity-level identifier.
    fn mlir_base_name(&self) -> String;

    /// This function's MLIR symbol qualified by its globally-unique node id, so
    /// two file-level functions of the same canonical signature — reachable
    /// together when one is imported under an alias — do not collide on a single
    /// symbol. Such functions are only ever resolved by node id, so the exact
    /// spelling is immaterial.
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
