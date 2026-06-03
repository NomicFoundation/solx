//!
//! Slang AST lowering to MLIR.
//!

/// Contract definition lowering to Sol dialect MLIR.
pub mod contract;
/// User-defined operator bindings (`using {f as op} for T global;`).
pub mod operator_binding;

use std::collections::BTreeMap;

use slang_solidity_v2::ast::ContractDefinition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;

use solx_mlir::Context;

use self::contract::ContractEmitter;
use self::operator_binding::OperatorBindings;

/// Walks a Slang AST and lowers a single contract definition to MLIR.
pub struct AstEmitter<'state, 'context> {
    /// The shared MLIR context.
    state: &'state mut Context<'context>,
}

impl<'state, 'context> AstEmitter<'state, 'context> {
    /// Creates a new AST emitter.
    pub fn new(state: &'state mut Context<'context>) -> Self {
        Self { state }
    }

    /// Emits MLIR for `contract` and returns its name and method-identifier
    /// table.
    ///
    /// One [`Context`] holds one contract's MLIR module, so the caller is
    /// expected to iterate the source unit's contracts and call this with a
    /// fresh [`Context`] per contract.
    ///
    /// # Errors
    ///
    /// Returns an error if code generation encounters unsupported constructs.
    pub fn emit(
        &mut self,
        contract: &ContractDefinition,
        free_functions: &[FunctionDefinition],
        operator_bindings: &OperatorBindings,
    ) -> anyhow::Result<(String, BTreeMap<String, String>)> {
        let name = contract.name().name();
        let mut emitter = ContractEmitter::new(self.state);
        emitter.emit(contract, free_functions, operator_bindings)?;

        let mut method_identifiers = BTreeMap::new();
        // Walk the inheritance-linearised function list so derived
        // contracts expose inherited externals in their ABI.
        for function in contract.compute_linearised_functions() {
            let Some(signature) = function.compute_canonical_signature() else {
                continue;
            };
            let Some(selector) = function.compute_selector() else {
                continue;
            };
            method_identifiers.insert(signature, format!("{selector:08x}"));
        }
        // Walk the inheritance-linearised state-variable list (not just this
        // contract's own members) so derived contracts expose inherited
        // `public` getters in their ABI — mirroring the function loop above.
        // The getter code itself is already emitted over the same linearised
        // set (see `emit_state_variable_getters`).
        for state_variable in contract.compute_linearised_state_variables() {
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

/// Extension methods on Slang's [`Expression`] AST node.
pub(crate) trait ExpressionExt {
    /// Peels redundant parenthesisation — single-element tuples — to a
    /// fixpoint, so a parenthesised expression (`(x)`, `((x))`, `(super)`) is
    /// treated like its bare inner form, mirroring how solc discards redundant
    /// parentheses. Returns the expression unchanged when it is not so wrapped.
    fn unwrap_parens(self) -> Self;
}

impl ExpressionExt for Expression {
    fn unwrap_parens(mut self) -> Self {
        loop {
            let inner = match &self {
                Expression::TupleExpression(tuple) if tuple.items().len() == 1 => {
                    tuple.items().iter().next().and_then(|item| item.expression())
                }
                _ => None,
            };
            match inner {
                Some(next) => self = next,
                None => return self,
            }
        }
    }
}
