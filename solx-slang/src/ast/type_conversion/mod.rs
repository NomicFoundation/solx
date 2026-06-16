//!
//! Solidity type conversion classification and dispatch.
//!

pub use solx_mlir::LocationPolicy;
pub use solx_mlir::ResolveSignature;
pub use solx_mlir::ResolveType;

use melior::ir::Type;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;

/// Transitional namespace for the two Slang→MLIR type-resolution entry points
/// not yet homed on the [`crate::ast::Type`] entity: the `Option`-lift over a
/// maybe-typed node and the always-typed state-variable resolution. Both
/// dissolve onto `Type` next; the recursive projection already lives there
/// ([`ResolveType`]).
pub struct TypeConversion;

impl TypeConversion {
    /// `Option`-lifted [`ResolveType::resolve_type`]: maps a possibly-absent
    /// slang type — as returned by `node.get_type()` on a node the binder left
    /// untyped (an unresolved reference or semantic error) — through with a
    /// `None` inherited location, yielding `None` when the slang type is absent.
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before emission.
    pub fn resolve_optional_slang_type<'context>(
        slang_type: Option<SlangType>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Type<'context>> {
        Some(slang_type?.resolve_type(LocationPolicy::Declared(None), builder))
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type<'context>(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> Type<'context> {
        let slang_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        slang_type.resolve_type(LocationPolicy::Declared(None), builder)
    }
}
