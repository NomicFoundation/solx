//!
//! TODO: pure-Slang queries pending a home (Slang dev-solx vs solx vs fold) —
//! query-sorting pass. Lifted off `ExpressionContext` (member-access and call
//! emission), which classified a member-access operand by its resolved definition.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

/// Classifies a member-access operand `x` (in `x.f`, `x.f(...)`, `E.Variant`, …)
/// by the definition it resolves to — whether it is a value, a namespace
/// qualifier, or a type reference. Carried by the operand expression itself.
pub trait MemberAccessOperand {
    /// Resolves this operand to its definition: a bare name (`E.Variant`, whose
    /// operand is the `Identifier` `E`) or a qualified path whose operand is
    /// itself a member access (`C.E.Variant`). Anything else has no definition.
    fn resolve_member_access_operand(&self) -> Option<Definition>;

    /// Whether this operand in `x.f(...)` is a namespace qualifier — a library or
    /// import alias (`L.f` / `M.f`), which is not a value — rather than a
    /// `using for` receiver, which becomes the implicit `self` argument.
    fn is_namespace_qualifier(&self) -> bool;

    /// Whether this operand is a namespace or type reference (a contract /
    /// interface / library / import / enum / struct / user-defined-value-type
    /// name) rather than a runtime value — such an operand carries no side
    /// effects, so a `.selector` taken through it evaluates nothing.
    fn is_namespace_or_type_operand(&self) -> bool;
}

impl MemberAccessOperand for Expression {
    fn resolve_member_access_operand(&self) -> Option<Definition> {
        match self {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }

    fn is_namespace_qualifier(&self) -> bool {
        let Expression::Identifier(identifier) = self else {
            return false;
        };
        matches!(
            identifier.resolve_to_definition(),
            Some(Definition::Library(_) | Definition::Import(_) | Definition::ImportedSymbol(_))
        )
    }

    fn is_namespace_or_type_operand(&self) -> bool {
        matches!(
            self.resolve_member_access_operand(),
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
