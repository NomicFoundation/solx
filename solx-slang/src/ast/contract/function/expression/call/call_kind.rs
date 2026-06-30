//!
//! Ahead-of-time classification of a function call's callee, so emission is one exhaustive `match`
//! rather than a chain of `Option`-returning probes.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;

use crate::ast::contract::contract_dispatch::ContractDispatch;
use crate::ast::contract::function::expression::call::external_library_call::ExternalLibraryCall;
use crate::ast::contract::function::expression::call::external_member_call::ExternalMemberCall;
use crate::ast::contract::function::expression::call::function_pointer_call::FunctionPointerCall;
use crate::ast::contract::function::expression::call::identifier_builtin_call::IdentifierBuiltinCall;
use crate::ast::contract::function::expression::call::identifier_function_call::IdentifierFunctionCall;
use crate::ast::contract::function::expression::call::index_access_conversion::IndexAccessConversion;
use crate::ast::contract::function::expression::call::inherited_function_call::InheritedFunctionCall;
use crate::ast::contract::function::expression::call::internal_member_call::InternalMemberCall;
use crate::ast::contract::function::expression::call::member_builtin_call::MemberBuiltinCall;
use crate::ast::contract::function::expression::call::new_expression_call::NewExpressionCall;
use crate::ast::contract::function::expression::call::struct_construction::StructConstruction;
use crate::ast::contract::function::expression::call::type_conversion::TypeConversion;

/// The one emission kind a function call's callee resolves to. The variants are mutually exclusive
/// and tested in declaration order, so an earlier match wins: a member access onto a library
/// function is an [`ExternalLibraryCall`], never an [`ExternalMemberCall`].
pub enum CallKind {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructConstruction),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion(TypeConversion),
    /// A call through a function-typed value rather than a named function.
    FunctionPointerCall(FunctionPointerCall),
    /// A built-in invoked by bare identifier (`require`, `keccak256`, `selfdestruct`).
    IdentifierBuiltinCall(IdentifierBuiltinCall),
    /// A built-in reached through member access (`address.call`, `bytes.concat`).
    MemberBuiltinCall(MemberBuiltinCall),
    /// A `super.f(...)` or base-qualified call redirected by inherited dispatch.
    InheritedFunctionCall(InheritedFunctionCall),
    /// An external call into a library member.
    ExternalLibraryCall(ExternalLibraryCall),
    /// A member call to an internal function, which carries no ABI selector.
    InternalMemberCall(InternalMemberCall),
    /// An external call to a contract function or a generated public getter.
    ExternalMemberCall(ExternalMemberCall),
    /// A `new C(...)` / `new bytes(...)` contract or dynamic-array creation.
    NewExpressionCall(NewExpressionCall),
    /// An array-type cast written with an index-access callee (`uint8[](value)`).
    IndexAccessConversion(IndexAccessConversion),
    /// A direct call to a named function, resolved through virtual dispatch.
    IdentifierFunctionCall(IdentifierFunctionCall),
}

impl CallKind {
    /// Classifies `callee` into the single kind that emits it.
    pub fn from_call(
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
        dispatch: &ContractDispatch,
    ) -> Self {
        if let Some(inner) = StructConstruction::from_call(call, callee) {
            return Self::StructConstruction(inner);
        }
        if let Some(inner) = TypeConversion::from_call(call) {
            return Self::TypeConversion(inner);
        }
        if let Some(inner) = FunctionPointerCall::from_callee(callee, arguments) {
            return Self::FunctionPointerCall(inner);
        }
        if let Some(inner) = IdentifierBuiltinCall::from_callee(callee, arguments) {
            return Self::IdentifierBuiltinCall(inner);
        }
        if let Some(inner) = MemberBuiltinCall::from_call(call, callee) {
            return Self::MemberBuiltinCall(inner);
        }
        if let Some(inner) = InheritedFunctionCall::from_callee(callee, arguments, dispatch) {
            return Self::InheritedFunctionCall(inner);
        }
        if let Some(inner) = ExternalLibraryCall::from_callee(callee, arguments) {
            return Self::ExternalLibraryCall(inner);
        }
        if let Some(inner) = InternalMemberCall::from_callee(callee, arguments) {
            return Self::InternalMemberCall(inner);
        }
        if let Some(inner) = ExternalMemberCall::from_callee(callee, arguments) {
            return Self::ExternalMemberCall(inner);
        }
        if let Some(inner) = NewExpressionCall::from_call(call, callee) {
            return Self::NewExpressionCall(inner);
        }
        if let Some(inner) = IndexAccessConversion::from_call(call, callee) {
            return Self::IndexAccessConversion(inner);
        }
        if let Some(inner) = IdentifierFunctionCall::from_callee(callee, arguments, dispatch) {
            return Self::IdentifierFunctionCall(inner);
        }
        if let Expression::Identifier(identifier) = callee {
            unreachable!(
                "callee '{}' does not resolve to a function",
                identifier.name()
            );
        }
        unreachable!("unsupported callee expression");
    }
}
