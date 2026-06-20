//!
//! The internal-function-pointer type of a bare function reference.
//!

use melior::ir::Type;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use solx_mlir::Context;

/// Recovers the `!sol.func_ref` type a bare internal-function reference carries.
/// Carried by the referencing expression.
pub trait BareFunctionRefType {
    /// If this expression is a bare function name — always an *internal* function
    /// pointer — its `!sol.func_ref` type, built from the function's declared
    /// signature.
    ///
    /// slang types such a reference from the function's visibility (a `Public`
    /// function resolves to its return type, not the pointer type), so a caller
    /// inferring a result type from the expression — e.g. a ternary whose branches
    /// are function names — uses this to recover the authoritative internal-pointer
    /// type the branch values carry. The target is routed exactly as a direct call
    /// is. Returns `None` for any expression that is not a bare function reference.
    fn bare_function_ref_type<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<Type<'context>>;
}

impl BareFunctionRefType for Expression {
    fn bare_function_ref_type<'context>(
        &self,
        context: &Context<'context>,
    ) -> Option<Type<'context>> {
        let Expression::Identifier(identifier) = self else {
            return None;
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            return None;
        };
        let function = context.resolve_function(function_definition.node_id());
        Some(function.func_ref_type(&context.builder).into_mlir())
    }
}
