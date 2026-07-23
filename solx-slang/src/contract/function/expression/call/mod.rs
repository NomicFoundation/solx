//!
//! Function call emission: the one construct whose lowering is resolution-directed rather than
//! syntax-directed, classified into [`Call`] kinds.
//!

pub mod arguments;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type;

use solx_mlir::Function;
use solx_mlir::Place;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;

use crate::scope::function::FunctionScope;

/// The one emission kind a function call's callee resolves to, owning both the classification and
/// the emission of each kind. The variants are mutually exclusive and tested in declaration order,
/// so an earlier match wins.
pub enum Call {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A built-in invoked by bare identifier (`require`, `keccak256`).
    Builtin(BuiltIn),
    /// A member-access callee (`address.send`, `abi.encode`, `abi.decode`). The member is resolved
    /// at emission, so a member resolving to no built-in or to one not lowered yet is rejected in
    /// one place rather than at both classification and emission.
    Member(MemberAccessExpression),
    /// A direct call to a named function.
    Function(FunctionDefinition),
}

impl Call {
    /// The canonical signature ABI-encoding a runtime `require` message.
    const ERROR_STRING_SIGNATURE: &'static str = "Error(string)";

    /// Classifies and emits `node`, routing each kind to its emission and returning its results in
    /// declaration order; statement-style built-ins yield an empty list.
    pub fn emit<'context>(
        node: &FunctionCallExpression,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let ArgumentsDeclaration::PositionalArguments(arguments) = &node.arguments() else {
            unreachable!("only positional arguments supported");
        };
        match Self::from_call(node) {
            Self::StructConstruction(struct_definition) => {
                Self::struct_construction(&struct_definition, node, arguments, scope)
            }
            Self::TypeConversion => Self::type_conversion(node, arguments, scope),
            Self::Builtin(built_in) => Self::builtin(built_in, arguments, scope)
                .into_iter()
                .collect(),
            Self::Member(access) => Self::member(&access, node, arguments, scope)
                .into_iter()
                .collect(),
            Self::Function(function_definition) => {
                Self::function(&function_definition, arguments, scope)
            }
        }
    }

    /// Classifies `call`'s callee into the single kind that emits it. A type conversion is probed
    /// before the callee's shape, its callee may be an elementary type or `payable` keyword as well
    /// as a named type, and its one-argument arity is part of the classification, per the variant's
    /// definition.
    fn from_call(call: &FunctionCallExpression) -> Self {
        let callee = call.operand();
        if let Expression::Identifier(identifier) = &callee
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
            Expression::Identifier(identifier) => {
                if let Some(built_in) = identifier.resolve_to_built_in() {
                    return Self::Builtin(built_in);
                }
                let Some(Definition::Function(function_definition)) =
                    identifier.resolve_to_definition()
                else {
                    unimplemented!(
                        "callee '{}' does not resolve to a function",
                        identifier.name()
                    );
                };
                Self::Function(function_definition)
            }
            Expression::MemberAccessExpression(access) => Self::Member(access),
            callee => unimplemented!(
                "unsupported callee expression: {:?}",
                std::mem::discriminant(&callee)
            ),
        }
    }

    /// Builds the struct value in memory: allocates the call's result type and stores each
    /// argument, converted to its field type, through the field's address.
    fn struct_construction<'context>(
        struct_definition: &StructDefinition,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let struct_address = Place::malloc(scope.typing(call.get_type()), scope);
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let field_type = scope.resolve_type(
                &member.get_type().expect("slang types every struct member"),
                Some(solx_utils::DataLocation::Memory),
            );
            let field_address = struct_address.gep_field(index, field_type, scope);
            field_address.store(scope.converted(&argument, field_type), scope);
        }
        vec![struct_address.into()]
    }

    /// Converts the conversion's one operand to the call's result type through an explicit `T(x)`
    /// cast.
    fn type_conversion<'context>(
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let operand = arguments
            .iter()
            .next()
            .expect("classification admits exactly one argument");
        let target_type = scope.typing(call.get_type());
        vec![scope.converted(&operand, target_type)]
    }

    /// Statement-style built-ins (`assert`, `require`, `revert`) produce no value.
    ///
    /// A literal `require` message lowers to the string form of `sol.require`; a non-literal message
    /// evaluates at runtime and is ABI-encoded under the `Error(string)` selector via its call form.
    fn builtin<'context>(
        built_in: BuiltIn,
        arguments: &PositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Option<Value<'context>> {
        match built_in {
            BuiltIn::Assert => {
                let condition_expression = arguments
                    .iter()
                    .next()
                    .expect("slang validates the arity of assert");
                let condition = scope.expression(&condition_expression).is_nonzero(scope);
                scope.current_block().assert(condition, scope);
                None
            }
            BuiltIn::Require => {
                let mut iter = arguments.iter();
                let condition_expression =
                    iter.next().expect("slang validates the arity of require");
                let condition = scope.expression(&condition_expression).is_nonzero(scope);
                let (values, message, custom) = match iter.next() {
                    Some(Expression::StringExpression(string_expression)) => (
                        Vec::new(),
                        Some(
                            String::from_utf8(string_expression.value())
                                .expect("slang validates string literals are UTF-8"),
                        ),
                        false,
                    ),
                    Some(expression) => {
                        let string_memory_type =
                            MlirType::string(scope.melior, solx_utils::DataLocation::Memory);
                        let message_value = scope.converted(&expression, string_memory_type);
                        (
                            vec![message_value],
                            Some(Self::ERROR_STRING_SIGNATURE.to_owned()),
                            true,
                        )
                    }
                    None => (Vec::new(), None, false),
                };
                let require = if custom {
                    solx_mlir::Block::require_custom
                } else {
                    solx_mlir::Block::require
                };
                require(
                    scope.current_block(),
                    condition,
                    &values,
                    message.as_deref(),
                    scope,
                );
                None
            }
            BuiltIn::Revert => {
                let signature: String = match arguments.iter().next() {
                    Some(Expression::StringExpression(string_expression)) => {
                        let message = String::from_utf8(string_expression.value())
                            .expect("slang validates string lals are UTF-8");
                        if message.is_empty() {
                            unimplemented!(
                                "revert with an empty reason is not yet supported; use revert() for a no-data revert"
                            );
                        }
                        message
                    }
                    Some(_) => unreachable!("revert message is a string lal"),
                    None => String::new(),
                };
                scope.current_block().revert(&signature, &[], scope);
                None
            }
            BuiltIn::Gasleft => Some(Value::gas_left(scope)),
            BuiltIn::Keccak256 => {
                let values = scope.positional_arguments(arguments);
                Some(Value::keccak256(values[0], scope))
            }
            BuiltIn::Sha256 => {
                let values = scope.positional_arguments(arguments);
                Some(Value::sha256(values[0], scope))
            }
            BuiltIn::Ripemd160 => {
                let values = scope.positional_arguments(arguments);
                Some(Value::ripemd160(values[0], scope))
            }
            BuiltIn::Ecrecover => {
                let values = scope.positional_arguments(arguments);
                Some(Value::ecrecover(
                    values[0], values[1], values[2], values[3], scope,
                ))
            }
            BuiltIn::Addmod => Some(scope.modular(arguments, Value::addmod)),
            BuiltIn::Mulmod => Some(scope.modular(arguments, Value::mulmod)),
            BuiltIn::Blockhash => {
                let field = MlirType::field(scope.melior);
                let values = scope.positional_arguments(arguments);
                Some(Value::blockhash(values[0].convert(field, scope), scope))
            }
            BuiltIn::Blobhash => {
                let field = MlirType::field(scope.melior);
                let values = scope.positional_arguments(arguments);
                Some(Value::blobhash(values[0].convert(field, scope), scope))
            }
            BuiltIn::Selfdestruct => {
                let values = scope.positional_arguments(arguments);
                Value::selfdestruct(values[0], scope);
                None
            }
            _ => unimplemented!("built-in {built_in:?} is not yet supported in call position"),
        }
    }

    /// Resolves the member to its built-in and lowers it: the array mutators lower to `sol.pop` and
    /// `sol.push`, where the no-argument `arr.push()` yields the new element's slot reference while
    /// `arr.push(x)` stores the converted value into that slot and, like `arr.pop()` and `transfer`,
    /// produces no value. `abi.decode` takes its result type from `call` rather than from its
    /// operands, which is why the full call expression is passed alongside the arguments. A member
    /// resolving to no built-in, or to one not lowered yet, is the sole unsupported-member-call site.
    fn member<'context>(
        access: &MemberAccessExpression,
        call: &FunctionCallExpression,
        arguments: &PositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Option<Value<'context>> {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressSend) => {
                let address = scope.expression(&access.operand());
                let values = scope.positional_arguments(arguments);
                Some(Value::send(address, values[0], scope))
            }
            Some(BuiltIn::AddressTransfer) => {
                let address = scope.expression(&access.operand());
                let values = scope.positional_arguments(arguments);
                Value::transfer(address, values[0], scope);
                None
            }
            Some(BuiltIn::AbiEncode) => {
                let values = scope.positional_arguments(arguments);
                Some(Value::encode(&values, None, scope))
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let values = scope.positional_arguments(arguments);
                Some(Value::encode_packed(&values, None, scope))
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let values = scope.positional_arguments(arguments);
                let selector = values[0].cast(
                    MlirType::fixed_bytes(scope.melior, MlirType::SELECTOR_BYTE_WIDTH),
                    scope,
                );
                Some(Value::encode(&values[1..], Some(selector), scope))
            }
            Some(BuiltIn::AbiEncodeWithSignature) => {
                let mut iter = arguments.iter();
                let signature_expression =
                    iter.next().expect("slang validates non-empty arguments");
                let Expression::StringExpression(string_expression) = signature_expression else {
                    unimplemented!(
                        "abi.encodeWithSignature with a non-literal signature is not yet supported"
                    );
                };
                let selector_word = u32::from_be_bytes(
                    solx_utils::Keccak256Hash::from_slice(&string_expression.value()).as_bytes()
                        [..solx_utils::BYTE_LENGTH_X32]
                        .try_into()
                        .expect("keccak256 always yields 32 bytes"),
                );
                let selector = Value::constant(
                    i64::from(selector_word),
                    MlirType::unsigned(scope.melior, solx_utils::BIT_LENGTH_X32),
                    scope,
                )
                .bytes_cast(
                    MlirType::fixed_bytes(scope.melior, MlirType::SELECTOR_BYTE_WIDTH),
                    scope,
                );
                let values = iter
                    .map(|argument| scope.expression(&argument))
                    .collect::<Vec<_>>();
                Some(Value::encode(&values, Some(selector), scope))
            }
            Some(BuiltIn::AbiDecode) => {
                let payload_expression = arguments
                    .iter()
                    .next()
                    .expect("slang validates the payload argument");
                let return_slang_type = call
                    .get_type()
                    .expect("abi.decode call is typed by the binder");
                if matches!(return_slang_type, Type::Tuple(_)) {
                    unimplemented!("abi.decode returning multiple values is not yet supported");
                }
                Some(Value::decode(
                    scope.expression(&payload_expression),
                    scope.resolve_type(&return_slang_type, None),
                    scope,
                ))
            }
            Some(BuiltIn::ArrayPop) => {
                scope.expression_place(&access.operand()).0.pop(scope);
                None
            }
            Some(BuiltIn::ArrayPush) => {
                let base = access.operand();
                let base_slang_type = base
                    .get_type()
                    .expect("base of array push has a resolved type");
                let value_argument = arguments.iter().next();
                let (place, _) = scope.expression_place(&base);

                if let Type::Bytes(_) = &base_slang_type
                    && let Some(value_argument) = &value_argument
                {
                    let appended = scope.converted(
                        value_argument,
                        MlirType::fixed_bytes(scope.melior, solx_utils::BYTE_LENGTH_BYTE as u32),
                    );
                    place.push_string(appended, scope);
                    return None;
                }

                let (element_type, slang_location) = match &base_slang_type {
                    Type::Array(array_type) => (
                        scope.resolve_type(&array_type.element_type(), None),
                        array_type.location(),
                    ),
                    Type::Bytes(bytes_type) => {
                        (MlirType::byte(scope.melior), bytes_type.location())
                    }
                    other => unreachable!(
                        "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                        std::mem::discriminant(other)
                    ),
                };
                let new_slot = place.push(
                    MlirType::pointer(
                        scope.melior,
                        element_type,
                        solx_utils::DataLocation::from_slang(slang_location, None),
                    ),
                    scope,
                );

                let Some(value_argument) = value_argument else {
                    return Some(new_slot);
                };
                Place::from(new_slot).store(scope.converted(&value_argument, element_type), scope);
                None
            }
            Some(BuiltIn::BytesConcat | BuiltIn::StringConcat) => {
                Some(Value::concat(&scope.positional_arguments(arguments), scope))
            }
            _ => unimplemented!("unsupported member call: {}", access.member().name()),
        }
    }

    /// Resolves the callee's pre-registered MLIR signature by node id and converts each argument to
    /// its declared parameter type before `sol.call`.
    fn function<'context>(
        function_definition: &FunctionDefinition,
        arguments: &PositionalArguments,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let signature = scope
            .contract
            .source_unit
            .function_signature(function_definition.node_id());
        let converted: Vec<Value<'context>> = arguments
            .iter()
            .zip(&signature.parameter_types)
            .map(|(argument, &parameter_type)| scope.converted(&argument, parameter_type))
            .collect();
        Function::call(
            &signature.mlir_name,
            &converted,
            &signature.return_types,
            scope,
        )
        .expect("sol.call yields its declared results")
    }
}

impl<'contract, 'source_unit, 'context> FunctionScope<'contract, 'source_unit, 'context> {
    /// `arr.push()` is the sole call assignable in place position; the slot it grows is the value the
    /// push emits, and its element type is that slot's pointee.
    pub fn function_call_place(
        &mut self,
        node: &FunctionCallExpression,
    ) -> (Place<'context>, MlirType<'context>) {
        let slot = Call::emit(node, self)
            .into_iter()
            .next()
            .expect("an array push in place position yields the new element's slot");
        (Place::from(slot), slot.r#type().element_type(0))
    }
}
