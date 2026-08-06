//!
//! Function call emission: the one construct whose lowering is resolution-directed rather than
//! syntax-directed, classified into [`Call`] kinds.
//!

pub mod arguments;
pub mod options;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptions;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::FunctionType;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type;

use solx_mlir::Function;
use solx_mlir::FunctionType as MlirFunctionType;
use solx_mlir::Place;
use solx_mlir::StateMutability;
use solx_mlir::Type as MlirType;
use solx_mlir::Value;
use solx_utils::FunctionReferenceKind;

use crate::contract::function::expression::call::options::Options;
use crate::scope::function::FunctionScope;
use crate::scope::source_unit::SourceUnitScope;

/// The one emission kind a function call's callee resolves to, owning both the classification and
/// the emission of each kind. The variants are mutually exclusive: `from_call` resolves a callee to
/// exactly one.
pub enum Call {
    /// The callee names a struct, so the call builds a struct value from its members.
    StructConstruction(StructDefinition),
    /// A one-argument elementary or user-defined-value-type conversion.
    TypeConversion,
    /// A built-in invoked by bare identifier (`require`, `keccak256`).
    Builtin(BuiltIn),
    /// A call to another contract's function (`instance.f(x)`, `this.f(x)`), dispatched by the
    /// callee's ABI selector. The callee's type is the operand's, which is where an overload is
    /// disambiguated and a decorated callee carries its partial application.
    External(MemberAccessExpression, FunctionDefinition, FunctionType),
    /// A member-access callee (`address.send`, `abi.encode`, `abi.decode`). The member is resolved
    /// at emission, so a member resolving to no built-in or to one not lowered yet is rejected in
    /// one place rather than at both classification and emission.
    Member(MemberAccessExpression),
    /// A direct call to a named function.
    Function(FunctionDefinition),
    /// A call through a function-typed value, internal or external.
    FunctionPointer(Expression, FunctionType),
}

impl Call {
    /// The canonical signature ABI-encoding a runtime `require` or `revert` message.
    const ERROR_STRING_SIGNATURE: &'static str = "Error(string)";

    /// Classifies and emits `node`, routing each kind to its emission and returning its results in
    /// declaration order; statement-style built-ins yield an empty list.
    pub fn emit<'context>(
        node: &FunctionCallExpression,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let (callee, options) = match node.operand() {
            Expression::CallOptionsExpression(decorated) => {
                (decorated.operand(), Some(decorated.options()))
            }
            callee => (callee, None),
        };
        let kind = Self::from_call(node, callee);
        let arguments = kind.arguments(node);
        match kind {
            Self::StructConstruction(struct_definition) => {
                Self::struct_construction(&struct_definition, node, &arguments, scope)
            }
            Self::TypeConversion => vec![Self::type_conversion(node, &arguments, scope)],
            Self::Builtin(built_in) => Self::builtin(built_in, &arguments, scope)
                .into_iter()
                .collect(),
            Self::External(access, function_definition, function_type) => Self::external(
                &access,
                &function_definition,
                &function_type,
                &arguments,
                options.as_ref(),
                scope,
            ),
            Self::Member(access) => {
                Self::member(&access, node, &arguments, options.as_ref(), scope)
            }
            Self::Function(function_definition) => scope.call(&function_definition, &arguments),
            Self::FunctionPointer(callee, function_type) => {
                Self::function_pointer(&callee, &function_type, &arguments, options.as_ref(), scope)
            }
        }
    }

    /// Classifies `call`'s callee into the single kind that emits it. A type conversion is probed
    /// before the callee's shape, its callee may be an elementary type or `payable` keyword as well
    /// as a named type, and its one-argument arity is part of the classification, per the variant's
    /// definition.
    fn from_call(call: &FunctionCallExpression, callee: Expression) -> Self {
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
                if let Some(Definition::Function(function_definition)) =
                    identifier.resolve_to_definition()
                {
                    return Self::Function(function_definition);
                }
                if let Some(Type::Function(function_type)) = identifier.get_type() {
                    return Self::FunctionPointer(
                        Expression::Identifier(identifier),
                        function_type,
                    );
                }
                unimplemented!("unsupported callee '{}'", identifier.name())
            }
            Expression::MemberAccessExpression(access) => {
                if matches!(
                    access.member().resolve_to_definition(),
                    Some(Definition::StructMember(_))
                ) && let Some(Type::Function(function_type)) = access.get_type()
                {
                    return Self::FunctionPointer(
                        Expression::MemberAccessExpression(access),
                        function_type,
                    );
                }
                if let Some(Definition::Function(function_definition)) =
                    access.member().resolve_to_definition()
                    && function_definition.compute_selector().is_some()
                    && let Some(Type::Function(function_type)) = call.operand().get_type()
                {
                    return Self::External(access, function_definition, function_type);
                }
                Self::Member(access)
            }
            callee => match callee.get_type() {
                Some(Type::Function(function_type)) => Self::FunctionPointer(callee, function_type),
                _ => unimplemented!(
                    "unsupported callee expression: {:?}",
                    std::mem::discriminant(&callee)
                ),
            },
        }
    }

    /// The call's arguments in the callee's declaration order, which is the order the named form
    /// evaluates in. A kind that declares no parameters orders against nothing, so the empty
    /// braces of `f({})` are its only named form.
    fn arguments(&self, call: &FunctionCallExpression) -> Vec<Expression> {
        match call.arguments() {
            ArgumentsDeclaration::PositionalArguments(positional) => positional.iter().collect(),
            ArgumentsDeclaration::NamedArguments(named) => match self {
                Self::StructConstruction(struct_definition) => FunctionScope::named_arguments(
                    &named,
                    struct_definition
                        .members()
                        .iter()
                        .map(|member| member.node_id()),
                ),
                Self::External(_, function_definition, _) | Self::Function(function_definition) => {
                    FunctionScope::named_arguments(
                        &named,
                        function_definition
                            .parameters()
                            .iter()
                            .map(|parameter| parameter.node_id()),
                    )
                }
                Self::FunctionPointer(_, function_type) => {
                    match function_type.associated_definition() {
                        Some(Definition::Function(function_definition)) => {
                            FunctionScope::named_arguments(
                                &named,
                                function_definition
                                    .parameters()
                                    .iter()
                                    .map(|parameter| parameter.node_id()),
                            )
                        }
                        _ if named.is_empty() => Vec::new(),
                        _ => unreachable!("a function value declares no labeled parameter"),
                    }
                }
                Self::Member(_) | Self::Builtin(_) if named.is_empty() => Vec::new(),
                Self::Member(_) => unimplemented!("named arguments on a member callee"),
                Self::Builtin(_) => unreachable!("a built-in declares no labeled parameter"),
                Self::TypeConversion => {
                    unreachable!("a type conversion classifies only positional arguments")
                }
            },
        }
    }

    /// Builds the struct value in memory: allocates the call's result type and stores each
    /// argument, converted to its field type, through the field's address.
    fn struct_construction<'context>(
        struct_definition: &StructDefinition,
        call: &FunctionCallExpression,
        arguments: &[Expression],
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let struct_address = Place::malloc(scope.typing(call.get_type()), scope);
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments)
            .enumerate()
        {
            let field_type = scope.resolve_type(
                &member.get_type().expect("slang types every struct member"),
                Some(solx_utils::DataLocation::Memory),
            );
            let field_address = struct_address.gep_field(index, field_type, scope);
            field_address.store(scope.converted(argument, field_type), scope);
        }
        vec![struct_address.into()]
    }

    /// Converts the conversion's one operand to the call's result type through an explicit `T(x)`
    /// cast.
    fn type_conversion<'context>(
        call: &FunctionCallExpression,
        arguments: &[Expression],
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Value<'context> {
        let operand = arguments
            .first()
            .expect("classification admits exactly one argument");
        let target_type = scope.typing(call.get_type());
        scope.converted(operand, target_type)
    }

    /// Statement-style built-ins (`assert`, `require`, `revert`) produce no value.
    ///
    /// A literal `require` message lowers to the string form of `sol.require`; a non-literal message
    /// evaluates at runtime and is ABI-encoded under the `Error(string)` selector via its call form.
    fn builtin<'context>(
        built_in: BuiltIn,
        arguments: &[Expression],
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Option<Value<'context>> {
        match built_in {
            BuiltIn::Assert => {
                let condition_expression = arguments
                    .first()
                    .expect("slang validates the arity of assert");
                let condition = scope.expression(condition_expression).is_nonzero(scope);
                scope.current_block().assert(condition, scope);
                None
            }
            BuiltIn::Require => {
                let mut iter = arguments.iter();
                let condition_expression =
                    iter.next().expect("slang validates the arity of require");
                let condition = scope.expression(condition_expression).is_nonzero(scope);
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
                        let message_value = scope.converted(expression, string_memory_type);
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
                match arguments.first() {
                    Some(Expression::StringExpression(string_expression)) => {
                        let message = String::from_utf8(string_expression.value())
                            .expect("slang validates string literals are UTF-8");
                        scope.current_block().revert(Some(&message), &[], scope);
                    }
                    Some(expression) => {
                        let string_type =
                            MlirType::string(scope.melior, solx_utils::DataLocation::Memory);
                        let message = scope.converted(expression, string_type);
                        scope.current_block().revert_custom(
                            Some(Self::ERROR_STRING_SIGNATURE),
                            &[message],
                            scope,
                        );
                    }
                    None => scope.current_block().revert(None, &[], scope),
                }
                None
            }
            BuiltIn::Gasleft => Some(Value::gas_left(scope)),
            BuiltIn::Keccak256 => {
                let data = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                Some(Value::keccak256(data, scope))
            }
            BuiltIn::Sha256 => {
                let data = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                Some(Value::sha256(data, scope))
            }
            BuiltIn::Ripemd160 => {
                let data = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                Some(Value::ripemd160(data, scope))
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

    /// Bridges the receiver to an address, converts each argument to its declared parameter type,
    /// and dispatches on the callee's ABI selector. The status the op yields is dropped: a failed
    /// call reverts, so only the callee's results reach the expression.
    fn external<'context>(
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        function_type: &FunctionType,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let address = scope.converted(&access.operand(), MlirType::address(scope.melior, false));
        let options = options
            .map(|options| Options::new(options, scope))
            .unwrap_or_default();
        let callee = Function::new(
            SourceUnitScope::symbol(function_definition),
            scope.contract.source_unit.function_type(function_type),
        );
        let converted = scope.external_arguments(arguments, &callee.function_type.parameters);
        let selector = Value::selector(
            function_definition
                .compute_selector()
                .expect("classification admits only callees with a selector"),
            MlirType::field(scope.melior),
            scope,
        );
        let gas = options.gas(scope);
        let amount = options.amount(scope);
        let call = if Self::is_static(function_type) {
            Function::external_static_call
        } else {
            Function::external_call
        };
        let (_status, values) = call(&callee, &converted, address, selector, gas, amount, scope);
        values
    }

    /// Resolves the member to its built-in and lowers it. `abi.decode` takes its result type from
    /// `call` rather than from its operands, which is why the full call expression is passed
    /// alongside the arguments. A member resolving to no built-in, or to one not lowered yet, is
    /// the sole unsupported-member-call site.
    fn member<'context>(
        access: &MemberAccessExpression,
        call: &FunctionCallExpression,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        match access.member().resolve_to_built_in() {
            Some(BuiltIn::AddressCall) => {
                let address =
                    scope.converted(&access.operand(), MlirType::address(scope.melior, false));
                let options = options
                    .map(|options| Options::new(options, scope))
                    .unwrap_or_default();
                let input = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                let gas = options.gas(scope);
                let amount = options.amount(scope);
                Value::bare_call(address, gas, amount, input, scope)
            }
            Some(BuiltIn::AddressDelegatecall) => {
                let address =
                    scope.converted(&access.operand(), MlirType::address(scope.melior, false));
                let options = options
                    .map(|options| Options::new(options, scope))
                    .unwrap_or_default();
                let input = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                let gas = options.gas(scope);
                Value::bare_delegate_call(address, gas, input, scope)
            }
            Some(BuiltIn::AddressStaticcall) => {
                let address =
                    scope.converted(&access.operand(), MlirType::address(scope.melior, false));
                let options = options
                    .map(|options| Options::new(options, scope))
                    .unwrap_or_default();
                let input = scope.converted(
                    &arguments[0],
                    MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                );
                let gas = options.gas(scope);
                Value::bare_static_call(address, gas, input, scope)
            }
            Some(BuiltIn::AddressSend) => {
                let address =
                    scope.converted(&access.operand(), MlirType::address(scope.melior, false));
                let values = scope.positional_arguments(arguments);
                vec![Value::send(address, values[0], scope)]
            }
            Some(BuiltIn::AddressTransfer) => {
                let address =
                    scope.converted(&access.operand(), MlirType::address(scope.melior, false));
                let values = scope.positional_arguments(arguments);
                Value::transfer(address, values[0], scope);
                Vec::new()
            }
            Some(BuiltIn::AbiEncode) => {
                let values = scope.positional_arguments(arguments);
                vec![Value::encode(&values, None, scope)]
            }
            Some(BuiltIn::AbiEncodePacked) => {
                let values = scope.positional_arguments(arguments);
                vec![Value::encode_packed(&values, None, scope)]
            }
            Some(BuiltIn::AbiEncodeWithSelector) => {
                let mut iter = arguments.iter();
                let selector_type = MlirType::selector(scope.melior);
                let selector = scope.converted(
                    iter.next().expect("slang validates non-empty arguments"),
                    selector_type,
                );
                let values = iter
                    .map(|argument| scope.expression(argument))
                    .collect::<Vec<_>>();
                vec![Value::encode(&values, Some(selector), scope)]
            }
            Some(BuiltIn::AbiEncodeWithSignature) => {
                let mut iter = arguments.iter();
                let selector_type = MlirType::selector(scope.melior);
                let selector = match iter.next().expect("slang validates non-empty arguments") {
                    Expression::StringExpression(signature) => Value::left_aligned_bytes(
                        solx_utils::Keccak256Hash::from_slice(&signature.value()).to_vec(),
                        selector_type,
                        scope,
                    ),
                    signature => {
                        let signature = scope.converted(
                            signature,
                            MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                        );
                        Value::keccak256(signature, scope).bytes_cast(selector_type, scope)
                    }
                };
                let values = iter
                    .map(|argument| scope.expression(argument))
                    .collect::<Vec<_>>();
                vec![Value::encode(&values, Some(selector), scope)]
            }
            Some(BuiltIn::AbiEncodeCall) => {
                let mut iter = arguments.iter();
                let callee = iter.next().expect("slang validates the callee argument");
                let callee_definition = match callee {
                    Expression::MemberAccessExpression(access) => {
                        access.member().resolve_to_definition()
                    }
                    _ => None,
                };
                let parameters: Vec<MlirType<'context>> = match callee_definition {
                    Some(Definition::Function(function_definition)) => function_definition
                        .parameters()
                        .iter()
                        .map(|parameter| scope.typing(parameter.get_type()))
                        .collect(),
                    _ => {
                        let Some(Type::Function(function_type)) = callee.get_type() else {
                            unreachable!("abi.encodeCall dispatches on an external function");
                        };
                        scope
                            .contract
                            .source_unit
                            .function_type(&function_type)
                            .parameters
                    }
                };
                let selector = scope.external_selector(callee);
                let values: Vec<Value<'context>> =
                    match iter.next().expect("slang validates the argument list") {
                        Expression::TupleExpression(tuple) => parameters
                            .into_iter()
                            .zip(tuple.items().iter())
                            .map(|(parameter_type, item)| {
                                scope.converted(
                                    &item.expression().expect("slang validates tuple elements"),
                                    parameter_type,
                                )
                            })
                            .collect(),
                        argument => {
                            let [parameter_type] = parameters[..] else {
                                unreachable!("an untupled argument list names one parameter");
                            };
                            vec![scope.converted(argument, parameter_type)]
                        }
                    };
                vec![Value::encode(&values, Some(selector), scope)]
            }
            Some(BuiltIn::AbiDecode) => {
                let payload_expression = arguments
                    .first()
                    .expect("slang validates the payload argument");
                let return_slang_type = call
                    .get_type()
                    .expect("abi.decode call is typed by the binder");
                let payload = scope.expression(payload_expression);
                let payload =
                    if payload.r#type().data_location() == solx_utils::DataLocation::CallData {
                        payload
                    } else {
                        payload.convert(
                            MlirType::string(scope.melior, solx_utils::DataLocation::Memory),
                            scope,
                        )
                    };
                let result_types: Vec<MlirType<'context>> = match return_slang_type {
                    Type::Tuple(tuple_type) => tuple_type
                        .types()
                        .iter()
                        .map(|element_type| scope.resolve_type(element_type, None))
                        .collect(),
                    other => vec![scope.resolve_type(&other, None)],
                };
                Value::decode(payload, &result_types, scope)
            }
            Some(BuiltIn::ArrayPop) => {
                scope.expression_place(&access.operand()).0.pop(scope);
                Vec::new()
            }
            Some(BuiltIn::ArrayPush) => {
                let base = access.operand();
                let base_slang_type = base
                    .get_type()
                    .expect("base of array push has a resolved type");
                let value_argument = arguments.first();
                let (place, _) = scope.expression_place(&base);

                if let Type::Bytes(_) = &base_slang_type
                    && let Some(value_argument) = value_argument
                {
                    let appended = scope.converted(
                        value_argument,
                        MlirType::fixed_bytes(scope.melior, solx_utils::BYTE_LENGTH_BYTE),
                    );
                    place.push_string(appended, scope);
                    return Vec::new();
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
                    return vec![new_slot];
                };
                Place::from(new_slot).store(scope.converted(value_argument, element_type), scope);
                Vec::new()
            }
            Some(BuiltIn::BytesConcat | BuiltIn::StringConcat) => {
                vec![Value::concat(&scope.positional_arguments(arguments), scope)]
            }
            Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                vec![Self::type_conversion(call, arguments, scope)]
            }
            _ => unimplemented!("unsupported member call: {}", access.member().name()),
        }
    }

    /// The signature comes from the callee's binder type: a pointer callee names no definition to
    /// look a registered signature up by. An internal pointer takes its arguments before the
    /// callee, an external one the callee first, then its options, then the arguments. An external
    /// pointer dispatches through `sol.ext_icall`, whose status the caller drops as a named
    /// external call does, and an internal function type admits no call options, so only the
    /// external path forwards them.
    fn function_pointer<'context>(
        callee: &Expression,
        function_type: &FunctionType,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        let MlirFunctionType {
            parameters,
            results,
        } = scope.contract.source_unit.function_type(function_type);
        match FunctionReferenceKind::from(function_type.visibility()) {
            FunctionReferenceKind::Internal => {
                let converted = scope.converted_arguments(arguments, &parameters);
                scope
                    .expression(callee)
                    .indirect_call(&converted, &results, scope)
            }
            FunctionReferenceKind::External => {
                let pointer = scope.expression(callee);
                let options = options
                    .map(|options| Options::new(options, scope))
                    .unwrap_or_default();
                let converted = scope.external_arguments(arguments, &parameters);
                let gas = options.gas(scope);
                let amount = options.amount(scope);
                let call = if Self::is_static(function_type) {
                    Value::external_static_call
                } else {
                    Value::external_call
                };
                let (_status, values) = call(pointer, &converted, &results, gas, amount, scope);
                values
            }
        }
    }

    /// Whether a callee dispatches through a static call, read from the callee's mutability
    /// rather than from the call syntax.
    fn is_static(function_type: &FunctionType) -> bool {
        match StateMutability::from(function_type.mutability()) {
            StateMutability::Pure | StateMutability::View => true,
            StateMutability::NonPayable | StateMutability::Payable => false,
        }
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

    /// Calls the function a `using {f as op} for T global;` directive binds an operator to. Being a
    /// call, its operands evaluate left-first, not in a built-in operator's right-first order.
    pub fn bound_operator(
        &mut self,
        function_definition: &FunctionDefinition,
        operands: &[Expression],
    ) -> Value<'context> {
        self.call(function_definition, operands)
            .into_iter()
            .next()
            .expect("a user-defined operator's function returns one value")
    }

    /// Resolves the callee's pre-registered MLIR signature by node id and converts each argument to
    /// its declared parameter type before `sol.call`.
    fn call(
        &mut self,
        function_definition: &FunctionDefinition,
        arguments: &[Expression],
    ) -> Vec<Value<'context>> {
        let signature = self
            .contract
            .source_unit
            .function_signature(function_definition.node_id());
        let converted = self.converted_arguments(arguments, &signature.function_type.parameters);
        Function::call(&signature, &converted, self)
    }
}
