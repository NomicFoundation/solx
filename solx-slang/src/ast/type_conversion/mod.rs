//!
//! Solidity type conversion classification and dispatch.
//!

pub mod location_policy;

pub use self::location_policy::LocationPolicy;
pub mod resolve_signature;
pub mod resolve_type;
pub use self::resolve_signature::ResolveSignature;
pub use self::resolve_type::ResolveType;

use melior::ir::Type;
use num_bigint::BigInt;
use num_traits::sign::Signed;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;

/// Solidity type resolution and default-initialisation.
///
/// A transitional namespace: the cast/coercion this type once classified now
/// lives on [`crate::ast::Value`] (`coerce_to` / `cast`), routed by the target
/// [`crate::ast::Type`]. What remains here — Slang→MLIR type resolution and
/// zero / default-initialisation — moves onto `Type` and `Value` / `Pointer` in
/// the resolution and constants stages.
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

    // TODO: Remove when nomicFoundation/slang#1793 is merged and we can instead
    // depend on `LiteralType::mobile_type()` for literal type conversion.
    fn integer_bits_required(value: &BigInt) -> u32 {
        if value.is_negative() {
            let magnitude_minus_one = -value - 1u32;
            u32::try_from(magnitude_minus_one.bits())
                .expect("literal magnitude bit count fits in u32")
                + 1
        } else {
            u32::try_from(value.bits())
                .expect("literal bit count fits in u32")
                .max(1)
        }
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
