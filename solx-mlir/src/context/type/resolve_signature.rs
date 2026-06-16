//!
//! Slang function-signature → MLIR type resolution.
//!

use melior::ir::Type as MlirType;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::Parameter;

use crate::Builder;
use crate::LocationPolicy;
use crate::Type;

impl<'context> Type<'context> {
    /// Resolves a function's `(parameter_types, return_types)` from Slang to MLIR
    /// under `policy`: [`LocationPolicy::Declared`] for the declared signature
    /// (used inside the callee's own body), [`LocationPolicy::ForceMemory`] for
    /// the external (ABI) signature — an external call ABI-encodes its arguments
    /// and decodes its results into memory (`calldata` cannot cross the call
    /// boundary), so solc shows a `bytes calldata` parameter as
    /// `!sol.string<Memory>` in the call's `callee_type`.
    pub fn resolve_signature(
        function: &FunctionDefinition,
        policy: LocationPolicy,
        builder: &Builder<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>) {
        let resolve = |parameter: Parameter| {
            Type::resolve(
                &parameter
                    .get_type()
                    .expect("slang validated"),
                policy,
                builder,
            )
        };
        let parameter_types = function.parameters().iter().map(&resolve).collect();
        let return_types = match function.returns() {
            Some(returns) => returns.iter().map(&resolve).collect(),
            None => Vec::new(),
        };
        (parameter_types, return_types)
    }
}
