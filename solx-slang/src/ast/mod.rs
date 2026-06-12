//!
//! Slang AST lowering to MLIR.
//!

pub mod arguments_declaration_ext;
/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;
pub mod expression_ext;
pub mod library_ext;
pub mod named_arguments_ext;
pub mod operator_binding;
/// Solidity type conversion classification and dispatch.
pub mod type_conversion;

pub use self::arguments_declaration_ext::ArgumentsDeclarationExt;
pub use self::expression_ext::ExpressionExt;
pub use self::library_ext::LibraryExt;
pub use self::named_arguments_ext::NamedArgumentsExt;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;

use self::contract::ContractEmitter;
use self::operator_binding::OperatorBindings;

/// Walks a Slang AST and lowers its contract definitions to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits MLIR for a single contract definition into this emitter's module.
    ///
    /// Each contract becomes its own deployable object (the caller gives every
    /// contract a fresh MLIR context), with the contract's C3-linearised bases'
    /// state variables and functions pulled in by the contract emitter — so a
    /// multi-contract file emits one complete object per contract, not just the
    /// first.
    ///
    /// `free_functions` is the compilation unit's full set of file-level (free)
    /// functions; the contract emitter pre-registers and emits the ones this
    /// contract reaches. `operator_bindings` is the unit's set of user-defined
    /// operator bindings (`using {f as op} for T global;`), shared across every
    /// contract.
    ///
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    /// Returns the contract's name and its public-method selector table.
    pub fn emit_contract(
        &mut self,
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        operator_bindings: &OperatorBindings,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract, free_functions, operator_bindings)?;

        let mut method_identifiers = BTreeMap::new();
        // Walk the C3-linearised function list (inherited + own) so a derived
        // contract exposes its inherited external functions in the ABI — not
        // only the contract's own members.
        for function in contract.linearised_functions() {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        // Walk the C3-linearised state-variable list so every `public` state
        // variable's auto-generated getter — own or inherited — appears in the
        // ABI. The getter code is already emitted over the same linearised set
        // (`emit_state_variable_getters`), so a contract with only a `public`
        // state variable (`string public greet;`) still exposes its selector.
        for state_variable in contract.linearised_state_variables() {
            let Some(signature) = state_variable.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = state_variable.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }

        Ok((name, method_identifiers))
    }
}
