//!
//! The resolved MLIR signature of a function.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionKind;
use solx_mlir::Builder;
use solx_mlir::StateMutability;

use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::contract::function::mlir_symbol_name::MlirSymbolName;

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
                (function.compute_selector(), mlir_kind)
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
