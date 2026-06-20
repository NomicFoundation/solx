//!
//! TODO: pure-Slang queries pending a home (Slang dev-solx vs solx vs fold) —
//! query-sorting pass. Lifted off `ExpressionContext` (member-access and call
//! emission), which classified a member-access operand by its resolved definition.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

/// A member-access operand `x` (in `x.f`, `x.f(...)`, `E.Variant`, …), viewed for
/// the definition it resolves to — whether it is a value, a namespace qualifier,
/// or a type reference. `Expression` is a foreign Slang node, so the classification
/// lives on this local lens over the operand, never on a trait bolted onto it.
pub struct MemberAccessOperand<'expression>(pub &'expression Expression);

impl MemberAccessOperand<'_> {
    /// Resolves the operand to its definition: a bare name (`E.Variant`, whose
    /// operand is the `Identifier` `E`) or a qualified path whose operand is itself
    /// a member access (`C.E.Variant`). Anything else has no definition.
    pub fn resolve(&self) -> Option<Definition> {
        match self.0 {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }

    /// Whether the operand in `x.f(...)` is a namespace qualifier — a library or
    /// import alias (`L.f` / `M.f`), which is not a value — rather than a
    /// `using for` receiver, which becomes the implicit `self` argument.
    pub fn is_namespace_qualifier(&self) -> bool {
        let Expression::Identifier(identifier) = self.0 else {
            return false;
        };
        matches!(
            identifier.resolve_to_definition(),
            Some(Definition::Library(_) | Definition::Import(_) | Definition::ImportedSymbol(_))
        )
    }

    /// Whether the operand is a namespace or type reference (a contract / interface
    /// / library / import / enum / struct / user-defined-value-type name) rather
    /// than a runtime value — such an operand carries no side effects, so a
    /// `.selector` taken through it evaluates nothing.
    pub fn is_namespace_or_type(&self) -> bool {
        matches!(
            self.resolve(),
            Some(
                Definition::Contract(_)
                    | Definition::Interface(_)
                    | Definition::Library(_)
                    | Definition::Import(_)
                    | Definition::ImportedSymbol(_)
                    | Definition::Enum(_)
                    | Definition::Struct(_)
                    | Definition::UserDefinedValueType(_)
            )
        )
    }
}
