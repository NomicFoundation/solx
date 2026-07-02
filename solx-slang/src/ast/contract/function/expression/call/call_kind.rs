//!
//! Ahead-of-time classification of a function call's callee, so emission is one exhaustive `match`
//! rather than a chain of `Option`-returning probes.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::StateVariableMutability;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

/// The one emission kind a function call's callee resolves to. The variants are mutually exclusive
/// and tested in declaration order, so an earlier match wins.
pub enum CallKind {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A call through a function-typed value (a local, parameter, contract-static state variable, or
    /// struct field) rather than a named function.
    FunctionPointerCall(Expression),
    /// A built-in invoked by bare identifier (`require`, `keccak256`) or a built-in reached through
    /// member access whose result type comes from the call (`abi.decode`).
    IdentifierBuiltinCall,
    /// An external call to a contract-instance method or `public` state-variable getter:
    /// `c.foo(args)`, dispatched by ABI selector.
    ExternalMemberCall(MemberAccessExpression, Definition),
    /// A built-in reached through member access (`address.balance`, `abi.encode`).
    MemberBuiltinCall(MemberAccessExpression),
    /// A `new C(...)` contract or `new T[](...)` / `new bytes(...)` dynamic-array creation.
    NewExpressionCall,
    /// A direct call to a named function.
    IdentifierFunctionCall(FunctionDefinition),
}

impl CallKind {
    /// Classifies `call`'s callee into the single kind that emits it.
    pub fn from_call(
        call: &FunctionCallExpression,
        callee: &Expression,
        arguments: &ArgumentsDeclaration,
    ) -> Self {
        if let Expression::Identifier(identifier) = callee
            && let Some(Definition::Struct(struct_definition)) = identifier.resolve_to_definition()
        {
            return Self::StructConstruction(struct_definition);
        }
        if let ArgumentsDeclaration::PositionalArguments(positional) = arguments
            && positional.len() == 1
            && (call.is_type_conversion() || Self::is_array_type_cast_callee(callee))
        {
            return Self::TypeConversion;
        }
        if Self::is_function_pointer_callee(callee) {
            return Self::FunctionPointerCall(callee.clone());
        }
        if let Expression::Identifier(identifier) = callee
            && identifier.resolve_to_built_in().is_some()
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee
            && matches!(
                access.member().resolve_to_built_in(),
                Some(BuiltIn::AbiDecode | BuiltIn::Wrap | BuiltIn::Unwrap)
            )
        {
            return Self::IdentifierBuiltinCall;
        }
        if let Expression::MemberAccessExpression(access) = callee
            && let Some(definition) = Self::external_member_callee(access)
        {
            return Self::ExternalMemberCall(access.clone(), definition);
        }
        if let Expression::MemberAccessExpression(access) = callee {
            return Self::MemberBuiltinCall(access.clone());
        }
        if let Expression::NewExpression(_) = callee {
            return Self::NewExpressionCall;
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

    /// Whether `callee` is an array-type expression `T[]` written as the callee of a cast
    /// `T[](value)`, which Slang parses as an index access with neither index nor slice bounds.
    fn is_array_type_cast_callee(callee: &Expression) -> bool {
        let Expression::IndexAccessExpression(array_type) = callee else {
            return false;
        };
        array_type.start().is_none() && array_type.end().is_none() && !array_type.is_slice()
    }

    /// Whether `callee` is a function-typed value the call dispatches through indirectly: a local,
    /// parameter, or contract-static state variable of function type, or a struct-member field of
    /// function type. A bare function name, a built-in, and a library member resolve to a definition
    /// rather than a value, so they fall through to their own dispatch.
    fn is_function_pointer_callee(callee: &Expression) -> bool {
        let addresses_value = match callee {
            Expression::Identifier(identifier) => matches!(
                identifier.resolve_to_definition(),
                Some(
                    Definition::Variable(_)
                        | Definition::Parameter(_)
                        | Definition::StateVariable(_)
                )
            ),
            Expression::MemberAccessExpression(access) => {
                match access.member().resolve_to_definition() {
                    Some(Definition::StructMember(_)) => true,
                    Some(Definition::StateVariable(_)) => matches!(
                        &access.operand(),
                        Expression::Identifier(operand)
                            if matches!(
                                operand.resolve_to_definition(),
                                Some(Definition::Contract(_))
                            )
                    ),
                    _ => false,
                }
            }
            _ => false,
        };
        addresses_value && matches!(callee.get_type(), Some(SlangType::Function(_)))
    }

    /// The contract method or `public` state-variable getter an external member call `c.foo(args)`
    /// dispatches to, or `None` when the access is not one.
    ///
    /// The member must resolve to a function or a getter carrying an ABI selector, so an internal or
    /// private method (no selector) falls through. The receiver must be a contract or interface
    /// instance value, which excludes both a library member call and a namespace- or type-qualified
    /// static call: those resolve through a `Library` or type reference, never a runtime instance.
    fn external_member_callee(access: &MemberAccessExpression) -> Option<Definition> {
        if !matches!(
            access.operand().get_type(),
            Some(SlangType::Contract(_) | SlangType::Interface(_))
        ) {
            return None;
        }
        match access.member().resolve_to_definition()? {
            Definition::Function(function_definition)
                if function_definition.compute_selector().is_some() =>
            {
                Some(Definition::Function(function_definition))
            }
            Definition::StateVariable(state_variable)
                if state_variable.compute_selector().is_some()
                    && !matches!(
                        state_variable.mutability(),
                        StateVariableMutability::Constant
                            | StateVariableMutability::Immutable
                    ) =>
            {
                Some(Definition::StateVariable(state_variable))
            }
            _ => None,
        }
    }
}
