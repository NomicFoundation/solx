//!
//! Ahead-of-time classification of a function-call expression.
//!

use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::StructDefinition;

use crate::ast::contract::function::expression::call::member_call_kind::MemberCallKind;

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
    /// `T.wrap(x)` / `T.unwrap(x)` for a user-defined value type — like
    /// [`Self::AbiDecode`], a member-access built-in whose result type comes from
    /// the call's own type (the bound UDVT, or its underlying type), not its
    /// operands. The UDVT shares its underlying type's representation, so this
    /// lowers to a single conversion of the argument.
    UdvtWrapUnwrap,
    /// Any other member-access built-in (`abi.encode`, `addr.send`,
    /// `arr.push`, `msg.sender`, …).
    BuiltInMemberAccess(MemberAccessExpression),
    /// Bare-identifier call to a user-defined function.
    LocalFunction(FunctionDefinition),
    /// Struct constructor `S(...)`.
    StructConstructor(StructDefinition),
    /// `new T(...)` — contract creation / `new T[](n)` array allocation.
    New,
    /// `f{value: v, gas: g}(...)` — a call wrapped in `CallOptionsExpression`;
    /// recurses on the inner callee classification.
    WithOptions(Box<CallKind>),
    /// `T[](n)` — array type conversion of an empty `T[]` callee.
    ArrayTypeConversion,
    /// A call through an internal function pointer value (`(f)(x)`, a callee
    /// resolving to a `Variable`/`Parameter`/`StateVariable` of function type).
    IndirectPointer,
    /// A member call (`x.f(...)`), dispatched by [`MemberCallKind`].
    Member(MemberCallKind),
}
