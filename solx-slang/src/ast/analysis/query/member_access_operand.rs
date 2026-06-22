//!
//! Member-access operand classification (a pure-Slang query).
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;

/// A local lens over a member-access operand `x` (in `x.f`, `E.Variant`, …), viewed for the
/// definition it resolves to — a value, a namespace qualifier, or a type reference.
pub struct MemberAccessOperand<'expression>(pub &'expression Expression);

impl MemberAccessOperand<'_> {
    /// Resolves the operand to its definition: a bare name `E` or a qualified path `C.E`. Else `None`.
    pub fn resolve(&self) -> Option<Definition> {
        match self.0 {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }

    /// Whether the operand is a namespace qualifier (a library / import alias, not a value)
    /// rather than a `using for` receiver.
    pub fn is_namespace_qualifier(&self) -> bool {
        let Expression::Identifier(identifier) = self.0 else {
            return false;
        };
        matches!(
            identifier.resolve_to_definition(),
            Some(Definition::Library(_) | Definition::Import(_) | Definition::ImportedSymbol(_))
        )
    }

    /// Whether the operand is a namespace or type reference (not a runtime value), so a `.selector`
    /// taken through it evaluates nothing.
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
