//!
//! The resolved MLIR signature of a function.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::StorageLocation;
use slang_solidity_v2::ast::TypeName;
use solx_mlir::Builder;
use solx_mlir::StateMutability;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;

/// The library external-function signature solc selects on, when it differs from Slang's plain ABI
/// signature, or `None` when the canonical signature already applies (a non-library function, or one
/// whose shape we don't recognise).
///
/// solc's `signatureInExternalFunction(structsByName = true)` re-names a library function's parameters
/// by their scope-qualified declared name. This implements the part of that rule the corpus exercises:
/// a **struct** parameter is named by its declared type-name path: a library-local `S` becomes `L.S`,
/// a qualified `I.S` stays `I.S`; and a `storage` reference parameter gets a trailing ` storage`
/// (e.g. `f(L.S storage)`, `g(uint256[] storage)`, `g(L.S)`, `a(I.S)`).
///
/// TODO: handle the cases that currently keep Slang's plain ABI form and so diverge from solc (see the
/// `slang-frontend-known-limitations` note): enum and contract/interface parameters (solc qualifies
/// them too, e.g. `L.E`, `Other`); a **file-level** struct (solc keeps the bare `S`, this yields the
/// wrong `L.S`); and structs referenced through an import alias (the written path is used verbatim
/// rather than the struct's canonical scope). A fully correct version needs each parameter's resolved
/// declaration and its enclosing-scope chain, which the public Slang API does not expose on a struct
/// definition.
pub fn library_aware_signature(function: &FunctionDefinition) -> Option<String> {
    let Some(Definition::Library(library)) = function.enclosing_definition() else {
        return None;
    };
    let library_name = library.name().name();

    let parameters: Vec<_> = function.parameters().iter().collect();
    let canonical = function.compute_canonical_signature()?;
    let open = canonical.find('(')?;
    let close = canonical.rfind(')')?;
    let canonical_types = split_top_level_commas(&canonical[open + 1..close]);
    if canonical_types.len() != parameters.len() {
        return None;
    }

    let is_storage = |parameter: &Parameter| {
        matches!(
            parameter.storage_location(),
            Some(StorageLocation::StorageKeyword(_))
        )
    };
    let mut located = Vec::with_capacity(parameters.len());
    for (parameter, canonical_type) in parameters.iter().zip(canonical_types.iter()) {
        let mut type_name = if canonical_type.contains('(') {
            let qualified = library_struct_name(parameter, &library_name)?;
            format!("{qualified}{suffix}", suffix = array_suffix(canonical_type))
        } else {
            (*canonical_type).to_owned()
        };
        if is_storage(parameter) {
            type_name.push_str(" storage");
        }
        located.push(type_name);
    }
    Some(format!("{}({})", &canonical[..open], located.join(",")))
}

/// The 4-byte selector of `function`, using [`library_aware_signature`] for a library function and
/// Slang's plain ABI selector otherwise. The one selector authority that the `sol.func` definition,
/// `.selector` reads, library call sites, and the ABI `methodIdentifiers` map all route through, so
/// the deployed dispatcher and the published ABI agree.
pub fn library_aware_selector(function: &FunctionDefinition) -> Option<u32> {
    let base_selector = function.compute_selector()?;
    let Some(signature) = library_aware_signature(function) else {
        return Some(base_selector);
    };
    let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
    let bytes = hash.as_bytes();
    Some(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// An approximation of the scope-qualified name solc gives a struct parameter of a library function
/// (`structsByName`): `Container.Struct` (e.g. `L.S`, `I.S`). The container is read from the
/// parameter's WRITTEN type-name path: an explicit qualifier (`I.S`) is taken verbatim, while an
/// unqualified name (`S`) is assumed to be a member of the enclosing library (`L.S`). Array wrappers
/// are peeled to the base type name; the caller re-attaches the array suffix. Returns `None` when the
/// base type is not an identifier path (then it is not a struct, and the caller uses the ABI form).
///
/// This is correct for a struct declared in the library or referenced through its real scope path
/// (the corpus cases).
///
/// TODO: a FILE-LEVEL struct diverges from solc's `StructDefinition::canonicalName()` (solc keeps
/// bare `S`, this yields `L.S`), as does an import alias (`Renamed.S` taken verbatim rather than the
/// canonical `I.S`). A faithful version needs the struct definition's enclosing-scope chain, which
/// the public Slang API does not expose on a struct definition.
fn library_struct_name(parameter: &Parameter, library_name: &str) -> Option<String> {
    let mut type_name = parameter.type_name();
    while let TypeName::ArrayTypeName(array) = type_name {
        type_name = array.operand();
    }
    let TypeName::IdentifierPath(path) = type_name else {
        return None;
    };
    let path_name = path.name();
    if path_name.contains('.') {
        Some(path_name)
    } else {
        Some(format!("{library_name}.{path_name}"))
    }
}

/// The array suffix (`[]`, `[2]`, `[][3]`, ...) of a canonical parameter type whose base is a struct
/// tuple: everything after the struct's leading balanced `(...)` group. Empty for a non-array
/// struct (`(uint256)` -> ``, `(uint256)[]` -> `[]`).
fn array_suffix(canonical_type: &str) -> &str {
    let mut depth = 0i32;
    for (index, character) in canonical_type.char_indices() {
        match character {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return &canonical_type[index + 1..];
                }
            }
            _ => {}
        }
    }
    ""
}

/// Splits a canonical parameter list on top-level commas, keeping nested tuple/array commas
/// (`(uint256,uint256)`, `uint256[2]`) within their parameter, so each returned slice is exactly one
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
    /// Resolves the MLIR signature of `function`: symbol, parameter and result
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
