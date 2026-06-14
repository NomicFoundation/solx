//!
//! Slang function-signature → MLIR type resolution: a function resolves its own
//! parameter and return types, the signature-level companion to [`ResolveType`].
//!
//! [`ResolveType`]: super::ResolveType

use melior::ir::Type;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Parameter;

use super::LocationPolicy;
use super::ResolveType;

/// Resolves a function's parameter and return types from Slang to MLIR.
pub trait ResolveSignature {
    /// The function's `(parameter_types, return_types)` resolved under `policy`:
    /// [`LocationPolicy::Declared`] for the declared signature (used inside the
    /// callee's own body), [`LocationPolicy::ForceMemory`] for the external (ABI)
    /// signature — an external call ABI-encodes its arguments and decodes its
    /// results into memory (`calldata` cannot cross the call boundary), so solc
    /// shows a `bytes calldata` parameter as `!sol.string<Memory>` in the call's
    /// `callee_type`.
    fn resolve_signature_types<'context>(
        &self,
        policy: LocationPolicy,
        builder: &solx_mlir::Builder<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>);
}

impl ResolveSignature for FunctionDefinition {
    fn resolve_signature_types<'context>(
        &self,
        policy: LocationPolicy,
        builder: &solx_mlir::Builder<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let resolve = |parameter: Parameter| {
            parameter
                .get_type()
                .expect("parameter type resolved by semantic analysis")
                .resolve_type(policy, builder)
        };
        let parameter_types = self.parameters().iter().map(&resolve).collect();
        let return_types = self
            .returns()
            .map(|returns| returns.iter().map(&resolve).collect())
            .unwrap_or_default();
        (parameter_types, return_types)
    }
}
