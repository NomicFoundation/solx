//!
//! The resolved MLIR signature of a function.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::StorageLocation;
use solx_mlir::Builder;
use solx_mlir::StateMutability;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;

/// The ABI selector of `function`, including the ` storage` data-location suffix on any storage
/// reference parameter — which a library external function carries in its canonical signature
/// (e.g. `g(uint256[] storage)`), matching solc. Slang's `compute_canonical_signature` omits the
/// location, so when a parameter is declared `storage` the selector is recomputed from a
/// location-aware signature; every other function keeps Slang's selector unchanged.
///
/// Only library external functions can carry a `storage` parameter in an externally-visible
/// signature (Solidity forbids it elsewhere), so this is a no-op for ordinary functions.
pub fn library_aware_selector(function: &FunctionDefinition) -> Option<u32> {
    let base_selector = function.compute_selector()?;
    let parameters: Vec<_> = function.parameters().iter().collect();
    let is_storage = |parameter: &slang_solidity_v2::ast::Parameter| {
        matches!(
            parameter.storage_location(),
            Some(StorageLocation::StorageKeyword(_))
        )
    };
    if !parameters.iter().any(is_storage) {
        return Some(base_selector);
    }
    let signature = function.compute_canonical_signature()?;
    let open = signature.find('(')?;
    let close = signature.rfind(')')?;
    let parameter_types = split_top_level_commas(&signature[open + 1..close]);
    // A shape mismatch (each parameter is exactly one top-level type) means the signature is not what
    // we expect; keep Slang's selector rather than emit a wrong one.
    if parameter_types.len() != parameters.len() {
        return Some(base_selector);
    }
    let located: Vec<String> = parameter_types
        .iter()
        .zip(parameters.iter())
        .map(|(type_name, parameter)| {
            if is_storage(parameter) {
                format!("{type_name} storage")
            } else {
                (*type_name).to_owned()
            }
        })
        .collect();
    let located_signature = format!("{}({})", &signature[..open], located.join(","));
    let hash = solx_utils::Keccak256Hash::from_slice(located_signature.as_bytes());
    let bytes = hash.as_bytes();
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// Splits a canonical parameter list on top-level commas, keeping nested tuple/array commas
/// (`(uint256,uint256)`, `uint256[2]`) within their parameter — so each returned slice is exactly one
/// parameter's canonical type.
fn split_top_level_commas(parameters: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (index, character) in parameters.char_indices() {
        match character {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&parameters[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    parts.push(&parameters[start..]);
    parts
}

/// The resolved MLIR signature of a function: its symbol name, parameter and
/// result types, public selector, mutability, and MLIR kind. Built by
/// [`Self::resolve`]; the caller reads these `pub` fields directly.
pub struct Signature<'context> {
    /// The MLIR symbol the `sol.func` is emitted under.
    pub mlir_name: String,
    /// The Sol-typed parameter types.
    pub mlir_parameter_types: Vec<Type<'context>>,
    /// The Sol-typed result types, parallel to the function's returns.
    pub result_types: Vec<Type<'context>>,
    /// The 4-byte public selector, when the function is externally dispatched.
    pub selector: Option<u32>,
    /// The Sol dialect state mutability.
    pub state_mutability: StateMutability,
    /// The Sol dialect function kind (constructor / fallback / receive), or `None`
    /// for a regular function.
    pub mlir_kind: Option<solx_mlir::FunctionKind>,
}

impl<'context> Signature<'context> {
    /// Resolves the MLIR signature of `function` — symbol, parameter and result
    /// types, selector, mutability, and kind. A `symbol_override` (a free / library /
    /// shadowed-base function) carries no public selector or special function kind.
    pub fn resolve(
        function: &FunctionDefinition,
        symbol_override: Option<&str>,
        builder: &Builder<'context>,
    ) -> Self {
        let mlir_name = symbol_override
            .map(str::to_owned)
            .unwrap_or_else(|| function.mlir_function_name());

        let (mlir_parameter_types, result_types) =
            AstType::resolve_signature(function, LocationPolicy::Declared(None), builder);

        let state_mutability = StateMutability::from(function.mutability());

        let (selector, mlir_kind) = match symbol_override {
            None => {
                let mlir_kind = match function.kind() {
                    FunctionKind::Constructor => Some(solx_mlir::FunctionKind::Constructor),
                    FunctionKind::Fallback => Some(solx_mlir::FunctionKind::Fallback),
                    FunctionKind::Receive => Some(solx_mlir::FunctionKind::Receive),
                    FunctionKind::Regular => None,
                    FunctionKind::Modifier => {
                        unreachable!("modifiers are filtered before emission")
                    }
                };
                (library_aware_selector(function), mlir_kind)
            }
            Some(_) => (None, None),
        };

        Signature {
            mlir_name,
            mlir_parameter_types,
            result_types,
            selector,
            state_mutability,
            mlir_kind,
        }
    }
}
