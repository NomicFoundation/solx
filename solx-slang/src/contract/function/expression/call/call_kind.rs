//!
//! The classification of a function call's callee, owning each kind's emission.
//!

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression as SlangExpression;
use slang_solidity_v2::ast::FunctionCallExpression as SlangFunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments as SlangPositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::Function as MlirFunction;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::contract::function::expression::Expression;
use crate::contract::function::expression::call::positional_arguments::PositionalArguments;
use crate::contract::function::expression::literal::StringExpression;
use crate::scope::FunctionScope;
use crate::r#type::Type;

/// The one emission kind a function call's callee resolves to, owning both the classification and
/// the emission of each kind. The variants are mutually exclusive and tested in declaration order,
/// so an earlier match wins.
pub enum CallKind {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A built-in resolved to its [`BuiltIn`] variant: invoked by bare identifier (`require`,
    /// `keccak256`), or through member access when the result type comes from the call itself
    /// (`abi.decode`).
    BuiltinCall(BuiltIn),
    /// A member-access callee (`address.balance`, `abi.encode`). The built-in is resolved at
    /// emission, so a member resolving to no built-in or to one not lowered yet is rejected in one
    /// place rather than at both classification and emission.
    MemberBuiltinCall(MemberAccessExpression),
    /// A direct call to a named function.
    IdentifierFunctionCall(FunctionDefinition),
}

impl CallKind {
    /// Classifies `call`'s callee into the single kind that emits it. A type conversion is probed
    /// before the callee's shape, its callee may be an elementary type or `payable` keyword as
    /// well as a named type, and its one-argument arity is part of the classification, per the
    /// variant's definition.
    pub fn from_call(call: &SlangFunctionCallExpression) -> Self {
        let callee = call.operand();
        if let SlangExpression::Identifier(identifier) = &callee
            && let Some(Definition::Struct(struct_definition)) = identifier.resolve_to_definition()
        {
            return Self::StructConstruction(struct_definition);
        }
        if call.is_type_conversion()
            && let ArgumentsDeclaration::PositionalArguments(arguments) = &call.arguments()
            && arguments.len() == 1
        {
            return Self::TypeConversion;
        }
        match callee {
            SlangExpression::Identifier(identifier) => {
                if let Some(built_in) = identifier.resolve_to_built_in() {
                    return Self::BuiltinCall(built_in);
                }
                let Some(Definition::Function(function_definition)) =
                    identifier.resolve_to_definition()
                else {
                    unimplemented!(
                        "callee '{}' does not resolve to a function",
                        identifier.name()
                    );
                };
                Self::IdentifierFunctionCall(function_definition)
            }
            SlangExpression::MemberAccessExpression(access) => {
                match access.member().resolve_to_built_in() {
                    Some(BuiltIn::AbiDecode) => Self::BuiltinCall(BuiltIn::AbiDecode),
                    _ => Self::MemberBuiltinCall(access),
                }
            }
            callee => unimplemented!(
                "unsupported callee expression: {:?}",
                std::mem::discriminant(&callee)
            ),
        }
    }

    /// Emits the classified call, returning its results in declaration order; statement-style
    /// built-ins yield an empty list.
    pub fn emit<'context>(
        self,
        call: &SlangFunctionCallExpression,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = &call.arguments() else {
            unreachable!("only positional arguments supported");
        };
        match self {
            Self::StructConstruction(struct_definition) => {
                let result_type = codegen!(@result_type FunctionCallExpression, call, scope);
                let struct_address = Place::malloc(result_type, scope);
                for (index, (member, argument)) in struct_definition
                    .members()
                    .iter()
                    .zip(arguments.iter())
                    .enumerate()
                {
                    let field_type = Type::resolve(
                        &member.get_type().expect("slang types every struct member"),
                        Some(solx_utils::DataLocation::Memory),
                        scope,
                    );
                    let field_address = struct_address.gep_field(index, field_type, scope);
                    let stored = Expression::emit(&argument, scope).coerce(field_type, scope);
                    field_address.store(stored, scope);
                }
                vec![struct_address.into()]
            }
            Self::TypeConversion => {
                let operand = arguments
                    .iter()
                    .next()
                    .expect("classification admits exactly one argument");
                let target_type = codegen!(@result_type FunctionCallExpression, call, scope);
                vec![Expression::emit(&operand, scope).coerce(target_type, scope)]
            }
            Self::BuiltinCall(built_in) => Self::builtin(built_in, call, arguments, scope)
                .into_iter()
                .collect(),
            Self::MemberBuiltinCall(access) => Self::member_builtin(&access, arguments, scope)
                .into_iter()
                .collect(),
            Self::IdentifierFunctionCall(function_definition) => {
                Self::call_function(&function_definition, arguments, scope)
            }
        }
    }

    /// Statement-style built-ins (`assert`, `require`) produce no value. `abi.decode` takes its
    /// result type from `call` rather than from its operands, which is why the full call
    /// expression is passed alongside the arguments.
    ///
    /// A literal `require` message lowers to the string form of `sol.require`; a non-literal
    /// message evaluates at runtime and is ABI-encoded under the `Error(string)` selector via
    /// its call form.
    pub fn builtin<'context>(
        built_in: BuiltIn,
        call: &SlangFunctionCallExpression,
        arguments: &SlangPositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Option<Value<'context>> {
        match built_in {
            BuiltIn::Assert => {
                let condition_expression = arguments
                    .iter()
                    .next()
                    .expect("slang validates the arity of assert");
                let condition = Expression::emit(&condition_expression, scope).is_nonzero(scope);
                scope.current_block().assert(condition, scope);
                None
            }
            BuiltIn::Require => {
                let mut iter = arguments.iter();
                let condition_expression =
                    iter.next().expect("slang validates the arity of require");
                let condition = Expression::emit(&condition_expression, scope).is_nonzero(scope);
                let (values, message, custom) = match iter.next() {
                    Some(SlangExpression::StringExpression(string_expression)) => (
                        Vec::new(),
                        Some(StringExpression::text(&string_expression)),
                        false,
                    ),
                    Some(expression) => {
                        let string_memory_type =
                            MlirType::string(scope.melior, solx_utils::DataLocation::Memory);
                        let message_value =
                            Expression::emit(&expression, scope).coerce(string_memory_type, scope);
                        (vec![message_value], Some("Error(string)".to_owned()), true)
                    }
                    None => (Vec::new(), None, false),
                };
                (if custom {
                    solx_mlir::Block::require_custom
                } else {
                    solx_mlir::Block::require
                })(
                    scope.current_block(),
                    condition,
                    &values,
                    message.as_deref(),
                    scope,
                );
                None
            }
            BuiltIn::Gasleft => Some(Value::gas_left(scope)),
            BuiltIn::Keccak256 => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::keccak256(values[0], scope))
            }
            BuiltIn::Sha256 => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::sha256(values[0], scope))
            }
            BuiltIn::Ripemd160 => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::ripemd160(values[0], scope))
            }
            BuiltIn::Ecrecover => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::ecrecover(
                    values[0], values[1], values[2], values[3], scope,
                ))
            }
            BuiltIn::Addmod => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::addmod(values[0], values[1], values[2], scope))
            }
            BuiltIn::Mulmod => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::mulmod(values[0], values[1], values[2], scope))
            }
            BuiltIn::AbiDecode => {
                let payload_expression = arguments
                    .iter()
                    .next()
                    .expect("slang validates the payload argument");
                let payload = Expression::emit(&payload_expression, scope);
                let return_slang_type = call
                    .get_type()
                    .expect("abi.decode call is typed by the binder");
                if matches!(return_slang_type, SlangType::Tuple(_)) {
                    unimplemented!("abi.decode returning multiple values is not yet supported");
                }
                let result_type = Type::resolve(&return_slang_type, None, scope);
                Some(Value::decode(payload, result_type, scope))
            }
            _ => unimplemented!("built-in {built_in:?} is not yet supported in call position"),
        }
    }

    /// Resolves the member to its built-in and lowers it: the array mutators lower to `sol.pop` and
    /// `sol.push`, where the no-argument `arr.push()` yields the new element's slot reference while
    /// `arr.push(x)` stores the coerced value into that slot and, like `arr.pop()` and `transfer`,
    /// produces no value. A member resolving to no built-in, or to one not lowered yet, is the sole
    /// unsupported-member-call site.
    pub fn member_builtin<'context>(
        access: &MemberAccessExpression,
        arguments: &SlangPositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Option<Value<'context>> {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressSend) => {
                let address = Expression::emit(&access.operand(), scope);
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::send(address, values[0], scope))
            }
            Some(BuiltIn::AddressTransfer) => {
                let address = Expression::emit(&access.operand(), scope);
                let values = PositionalArguments::emit_values(arguments, scope);
                Value::transfer(address, values[0], scope);
                None
            }
            Some(BuiltIn::AbiEncode) => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::encode(&values, None, scope))
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let values = PositionalArguments::emit_values(arguments, scope);
                Some(Value::encode_packed(&values, None, scope))
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let values = PositionalArguments::emit_values(arguments, scope);
                let selector = values[0].cast(MlirType::fixed_bytes(scope.melior, 4), scope);
                Some(Value::encode(&values[1..], Some(selector), scope))
            }
            Some(BuiltIn::AbiEncodeWithSignature) => {
                let mut iter = arguments.iter();
                let signature_expression =
                    iter.next().expect("slang validates non-empty arguments");
                let SlangExpression::StringExpression(string_expression) = signature_expression
                else {
                    unimplemented!(
                        "abi.encodeWithSignature with a non-literal signature is not yet supported"
                    );
                };
                let hash = solx_utils::Keccak256Hash::from_slice(&string_expression.value());
                let selector_word = u32::from_be_bytes(
                    hash.as_bytes()[..4]
                        .try_into()
                        .expect("keccak256 always yields 32 bytes"),
                );
                let selector = Value::constant(
                    i64::from(selector_word),
                    MlirType::unsigned(scope.melior, solx_utils::BIT_LENGTH_X32),
                    scope,
                )
                .bytes_cast(MlirType::fixed_bytes(scope.melior, 4), scope);
                let values = iter
                    .map(|argument| Expression::emit(&argument, scope))
                    .collect::<Vec<_>>();
                Some(Value::encode(&values, Some(selector), scope))
            }
            Some(BuiltIn::ArrayPop) => {
                Expression::emit(&access.operand(), scope).pop(scope);
                None
            }
            Some(BuiltIn::ArrayPush) => {
                let base = access.operand();
                let base_slang_type = base
                    .get_type()
                    .expect("base of array push has a resolved type");
                let value_argument = arguments.iter().next();
                if value_argument.is_some() && matches!(&base_slang_type, SlangType::Bytes(_)) {
                    unimplemented!(
                        "bytes.push(x) lowers to sol.push_string, which is not yet wired"
                    );
                }
                let (element_type, slang_location) = match &base_slang_type {
                    SlangType::Array(array_type) => (
                        Type::resolve(&array_type.element_type(), None, scope),
                        array_type.location(),
                    ),
                    SlangType::Bytes(bytes_type) => (
                        MlirType::fixed_bytes(scope.melior, 1),
                        bytes_type.location(),
                    ),
                    other => unreachable!(
                        "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                        std::mem::discriminant(other)
                    ),
                };
                let base_location = solx_utils::DataLocation::from_slang(slang_location, None);
                let array_value = Expression::emit(&base, scope);
                let address_type = MlirType::pointer(scope.melior, element_type, base_location);
                let new_slot = array_value.push(address_type, scope);
                let Some(value_argument) = value_argument else {
                    return Some(new_slot);
                };
                let stored = Expression::emit(&value_argument, scope).coerce(element_type, scope);
                Place::from(new_slot).store(stored, scope);
                None
            }
            _ => unimplemented!("unsupported member call: {}", access.member().name()),
        }
    }

    /// Resolves the callee's pre-registered MLIR signature by node id and coerces each argument to
    /// its declared parameter type before `sol.call`.
    pub fn call_function<'context>(
        function_definition: &FunctionDefinition,
        arguments: &SlangPositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let argument_values = PositionalArguments::emit_values(arguments, scope);
        let signature = scope
            .contract()
            .source_unit()
            .function_signature(function_definition.node_id());
        let coerced: Vec<Value<'context>> = argument_values
            .iter()
            .zip(&signature.parameter_types)
            .map(|(value, &parameter_type)| value.coerce(parameter_type, scope))
            .collect();
        MlirFunction::call(
            &signature.mlir_name,
            &coerced,
            &signature.return_types,
            scope,
        )
        .expect("sol.call yields its declared results")
    }
}
