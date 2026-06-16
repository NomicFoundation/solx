//!
//! Slang function-signature → MLIR type resolution: a function resolves its own
//! parameter and return types, the signature-level companion to [`ResolveType`].
//!
//! [`ResolveType`]: crate::ResolveType

use melior::ir::Type as MlirType;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Parameter;

use crate::LocationPolicy;
use crate::ResolveType;

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
        builder: &crate::Builder<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>);
}

impl ResolveSignature for FunctionDefinition {
    fn resolve_signature_types<'context>(
        &self,
        policy: LocationPolicy,
        builder: &crate::Builder<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>) {
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
