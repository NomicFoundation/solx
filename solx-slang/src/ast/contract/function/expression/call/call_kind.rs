//!
//! Ahead-of-time classification of a function-call expression.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::StructDefinition;

/// The resolved kind of a `FunctionCallExpression`, computed once ahead of
/// emission so dispatch is a single `match` rather than a speculative chain of
/// fallible attempts.
pub enum CallKind {
    /// Explicit type conversion `T(x)` — `is_type_conversion()` with exactly
    /// one argument.
    TypeConversion,
    /// Identifier callee resolving to an emittable built-in (`keccak256`,
    /// `require`, `addmod`, …).
    BuiltInIdentifier(BuiltIn),
    /// `abi.decode(payload, (T))` — a member-access built-in whose result type
    /// comes from the call's own type rather than from its operands.
    AbiDecode,
    /// Any other member-access built-in (`abi.encode`, `addr.send`,
    /// `arr.push`, `msg.sender`, …).
    BuiltInMemberAccess(MemberAccessExpression),
    /// Bare-identifier call to a user-defined function.
    LocalFunction(FunctionDefinition),
    /// Struct constructor `S(...)`.
    StructConstructor(StructDefinition),
}
