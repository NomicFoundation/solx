//!
//! Ahead-of-time classification of a function call's callee, so emission is one exhaustive `match`
//! rather than a chain of `Option`-returning probes.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;

/// The one emission kind a function call's callee resolves to. The variants are mutually exclusive
/// and tested in declaration order, so an earlier match wins.
pub enum CallKind {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A built-in invoked by bare identifier (`require`, `keccak256`) or a built-in reached through
    /// member access whose result type comes from the call (`abi.decode`).
    IdentifierBuiltinCall,
    /// A built-in reached through member access (`address.balance`, `abi.encode`).
    MemberBuiltinCall(MemberAccessExpression),
    /// A direct call to a named function.
    IdentifierFunctionCall(FunctionDefinition),
}

impl CallKind {
    /// Classifies `call`'s callee into the single kind that emits it.
    pub fn from_call(
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &PositionalArguments,
    ) -> Self {
        if let Expression::Identifier(identifier) = callee
            && let Some(Definition::Struct(struct_definition)) = identifier.resolve_to_definition()
        {
            return Self::StructConstruction(struct_definition);
        }
        if call.is_type_conversion() && arguments.len() == 1 {
            return Self::TypeConversion;
        }
        if let Expression::Identifier(identifier) = callee
            && identifier.resolve_to_built_in().is_some()
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee
            && matches!(access.member().resolve_to_built_in(), Some(BuiltIn::AbiDecode))
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee {
            return Self::MemberBuiltinCall(access.clone());
        }
        let Expression::Identifier(identifier) = callee else {
            unreachable!("unsupported callee expression");
        };
        let Some(Definition::Function(function_definition)) = identifier.resolve_to_definition()
        else {
            unreachable!("callee '{}' does not resolve to a function", identifier.name());
        };
        Self::IdentifierFunctionCall(function_definition)
    }
}
