//!
//! Function call emission: the one construct whose lowering is resolution-directed rather than
//! syntax-directed, classified into [`Call`] kinds.
//!

pub mod arguments;
pub mod options;

use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptions;
use slang_solidity_v2::ast::ContractDefinition;
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
    /// `new C(..)` deploying a contract, its declared constructor ordering and typing the arguments.
    Creation(ContractDefinition, Option<FunctionDefinition>),
    /// `new T[](n)` / `new bytes(n)` allocating a dynamically sized value in memory.
    Allocation,
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
        let (callee, options) = Self::callee(node);
        let kind = Self::from_call(node, callee);
        let arguments = kind.arguments(node);
        match kind {
            Self::StructConstruction(struct_definition) => {
                Self::struct_construction(&struct_definition, node, &arguments, scope)
            }
            Self::Creation(contract_definition, constructor) => vec![Self::creation(
                &contract_definition,
                constructor.as_ref(),
                node,
                &arguments,
                options.as_ref(),
                false,
                scope,
            )],
            Self::Allocation => vec![Self::allocation(node, &arguments, scope)],
            Self::TypeConversion => vec![Self::type_conversion(node, &arguments, scope)],
            Self::Builtin(built_in) => Self::builtin(built_in, &arguments, scope)
                .into_iter()
                .collect(),
            Self::External(access, function_definition, function_type) => {
                let (_status, values) = Self::external(
                    &access,
                    &function_definition,
                    &function_type,
                    &arguments,
                    options.as_ref(),
                    false,
                    scope,
                );
                values
            }
            Self::Member(access) => {
                Self::member(&access, node, &arguments, options.as_ref(), scope)
            }
            Self::Function(function_definition) => scope.call(&function_definition, &arguments),
            Self::FunctionPointer(callee, function_type) => {
                Self::function_pointer(&callee, &function_type, &arguments, options.as_ref(), scope)
            }
        }
    }

    /// Classifies and emits the call a `try` statement guards, keeping the status the unguarded path
    /// drops and selecting the op's guarded wrapper. A creation carries no status of its own, so one
    /// comes from the address it produced: `CREATE` reports failure by returning zero.
    pub fn try_call<'context>(
        node: &FunctionCallExpression,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> (Value<'context>, Vec<Value<'context>>) {
        let (callee, options) = Self::callee(node);
        let kind = Self::from_call(node, callee);
        let arguments = kind.arguments(node);
        match kind {
            Self::Creation(contract_definition, constructor) => {
                let contract = Self::creation(
                    &contract_definition,
                    constructor.as_ref(),
                    node,
                    &arguments,
                    options.as_ref(),
                    true,
                    scope,
                );
                let address = contract
                    .address_cast(MlirType::address(scope.melior, false), scope)
                    .address_cast(
                        MlirType::unsigned(scope.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                        scope,
                    );
                (address.is_nonzero(scope), vec![contract])
            }
            Self::External(access, function_definition, function_type) => Self::external(
                &access,
                &function_definition,
                &function_type,
                &arguments,
                options.as_ref(),
                true,
                scope,
            ),
            Self::FunctionPointer(callee, function_type) => Self::external_pointer(
                &callee,
                &function_type,
                &arguments,
                options.as_ref(),
                true,
                scope,
            ),
            Self::Member(access) => {
                unimplemented!("unsupported member call: {}", access.member().name())
            }
            Self::StructConstruction(_)
            | Self::Allocation
            | Self::TypeConversion
            | Self::Builtin(_)
            | Self::Function(_) => {
                unreachable!("a guarded call dispatches externally or creates a contract")
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
            Expression::NewExpression(_) => match call.get_type() {
                Some(Type::Contract(contract_type)) => {
                    let Definition::Contract(contract_definition) = contract_type.definition()
                    else {
                        unreachable!("slang ContractType always references a Contract definition");
                    };
                    let constructor = contract_definition.constructor();
                    Self::Creation(contract_definition, constructor)
                }
                _ => Self::Allocation,
            },
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

    /// The callee under the parentheses and call-option layers wrapping it, and the options those
    /// layers carry. Both nest freely and in either order, so a peel that stops at one of them
    /// leaves the other in a position no emitter admits.
    fn callee(call: &FunctionCallExpression) -> (Expression, Option<CallOptions>) {
        let mut callee = call.operand();
        let mut options = None;
        loop {
            match callee {
                Expression::CallOptionsExpression(decorated) => {
                    options = Some(decorated.options());
                    callee = decorated.operand();
                }
                Expression::TupleExpression(inner) => match inner
                    .items()
                    .iter()
                    .next()
                    .and_then(|item| item.expression())
                    .expect("a parenthesized callee wraps a single operand")
                {
                    Expression::Identifier(_) | Expression::MemberAccessExpression(_) => {
                        return (Expression::TupleExpression(inner), options);
                    }
                    wrapped => callee = wrapped,
                },
                resolved => return (resolved, options),
            }
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
                Self::Creation(_, Some(function_definition))
                | Self::External(_, function_definition, _)
                | Self::Function(function_definition) => FunctionScope::named_arguments(
                    &named,
                    function_definition
                        .parameters()
                        .iter()
                        .map(|parameter| parameter.node_id()),
                ),
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
                Self::Creation(_, None) | Self::Member(_) | Self::Builtin(_)
                    if named.is_empty() =>
                {
                    Vec::new()
                }
                Self::Creation(_, None) => {
                    unreachable!("a contract without a constructor declares no parameter")
                }
                Self::Member(_) => unimplemented!("named arguments on a member callee"),
                Self::Builtin(_) => unreachable!("a built-in declares no labeled parameter"),
                Self::Allocation => {
                    unreachable!("an allocation takes its length argument positionally")
                }
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
        let struct_address = Place::malloc(scope.typing(call.get_type()), None, scope);
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

    /// Deploys a contract, ABI-encoding the constructor arguments after the bytecode it copies.
    /// A creation is an external dispatch in that respect, so a reference-typed argument is
    /// encoded out of the location it already lives in and only a scalar converts.
    fn creation<'context>(
        contract_definition: &ContractDefinition,
        constructor: Option<&FunctionDefinition>,
        call: &FunctionCallExpression,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        guarded: bool,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Value<'context> {
        let result_type = scope.typing(call.get_type());
        let options = options
            .map(|options| Options::new(options, scope))
            .unwrap_or_default();
        let parameter_types: Vec<MlirType<'context>> = constructor
            .map(|constructor| {
                constructor
                    .parameters()
                    .iter()
                    .map(|parameter| scope.typing(parameter.get_type()))
                    .collect()
            })
            .unwrap_or_default();
        let converted = scope.external_arguments(arguments, &parameter_types);
        let amount = options.amount(scope);
        let object = SourceUnitScope::object_identifier(
            contract_definition.get_file_id(),
            contract_definition.name().name(),
        );
        let create = if guarded {
            Value::create_contract_try
        } else {
            Value::create_contract
        };
        create(
            object.as_str(),
            amount,
            options.salt(),
            &converted,
            result_type,
            scope,
        )
    }

    /// Allocates a dynamically sized value in memory. The size reaches `sol.malloc` in its declared
    /// type, which the lowering masks it against, so widening it here would change that cleanup.
    fn allocation<'context>(
        call: &FunctionCallExpression,
        arguments: &[Expression],
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Value<'context> {
        let result_type = scope.typing(call.get_type());
        let size = scope.expression(
            arguments
                .first()
                .expect("slang validates that an allocation names its size"),
        );
        Place::malloc_zeroed(result_type, Some(size), scope).into()
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
    /// and dispatches on the callee's ABI selector. Unguarded, the caller drops the status: a failed
    /// call reverts, so only the callee's results reach the expression.
    fn external<'context>(
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        function_type: &FunctionType,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        guarded: bool,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> (Value<'context>, Vec<Value<'context>>) {
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
        let call = match (Self::is_static(function_type), guarded) {
            (false, false) => Function::external_call,
            (true, false) => Function::external_static_call,
            (false, true) => Function::external_try_call,
            (true, true) => Function::external_static_try_call,
        };
        call(&callee, &converted, address, selector, gas, amount, scope)
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

                let (element_type, slot_type) = match &base_slang_type {
                    Type::Array(array_type) => {
                        let element_slang_type = array_type.element_type();
                        let element_type = scope.resolve_type(&element_slang_type, None);
                        (
                            element_type,
                            scope.pointer_type(
                                &element_slang_type,
                                element_type,
                                solx_utils::DataLocation::from_slang(array_type.location(), None),
                            ),
                        )
                    }
                    Type::Bytes(bytes_type) => {
                        let element_type = MlirType::byte(scope.melior);
                        (
                            element_type,
                            MlirType::pointer(
                                scope.melior,
                                element_type,
                                solx_utils::DataLocation::from_slang(bytes_type.location(), None),
                            ),
                        )
                    }
                    other => unreachable!(
                        "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                        std::mem::discriminant(other)
                    ),
                };
                let new_slot = place.push(slot_type, scope);

                let Some(value_argument) = value_argument else {
                    return vec![new_slot];
                };
                let place = Place::from(new_slot);
                if let Expression::StringExpression(_) = value_argument
                    && element_type.is_bytes_like()
                {
                    let value = scope.converted(value_argument, element_type);
                    place.assign(value, element_type, scope);
                    return Vec::new();
                }
                let value = scope.expression(value_argument);
                place.assign(value, element_type, scope);
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

    /// A call through a function-typed value. An internal pointer goes through `sol.icall` and takes
    /// its arguments before the callee; an external one is an external call in every respect and
    /// drops its status as one does.
    fn function_pointer<'context>(
        callee: &Expression,
        function_type: &FunctionType,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> Vec<Value<'context>> {
        match FunctionReferenceKind::from(function_type.visibility()) {
            FunctionReferenceKind::Internal => {
                let MlirFunctionType {
                    parameters,
                    results,
                } = scope.contract.source_unit.function_type(function_type);
                let converted = scope.converted_arguments(arguments, &parameters);
                scope
                    .expression(callee)
                    .indirect_call(&converted, &results, scope)
            }
            FunctionReferenceKind::External => {
                let (_status, values) =
                    Self::external_pointer(callee, function_type, arguments, options, false, scope);
                values
            }
        }
    }

    /// A call through an external function pointer: the callee, then its options, then the
    /// arguments. The signature comes from the callee's binder type, since a pointer names no
    /// definition to look a registered one up by.
    fn external_pointer<'context>(
        callee: &Expression,
        function_type: &FunctionType,
        arguments: &[Expression],
        options: Option<&CallOptions>,
        guarded: bool,
        scope: &mut FunctionScope<'_, '_, 'context>,
    ) -> (Value<'context>, Vec<Value<'context>>) {
        let MlirFunctionType {
            parameters,
            results,
        } = scope.contract.source_unit.function_type(function_type);
        let pointer = scope.expression(callee);
        let options = options
            .map(|options| Options::new(options, scope))
            .unwrap_or_default();
        let converted = scope.external_arguments(arguments, &parameters);
        let gas = options.gas(scope);
        let amount = options.amount(scope);
        let call = match (Self::is_static(function_type), guarded) {
            (false, false) => Value::external_call,
            (true, false) => Value::external_static_call,
            (false, true) => Value::external_try_call,
            (true, true) => Value::external_static_try_call,
        };
        call(pointer, &converted, &results, gas, amount, scope)
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
    /// push emits. The binder types a `bytes` element `bytes1` where the dialect stores a byte.
    pub fn function_call_place(
        &mut self,
        node: &FunctionCallExpression,
    ) -> (Place<'context>, MlirType<'context>) {
        let slot = Call::emit(node, self)
            .into_iter()
            .next()
            .expect("an array push in place position yields the new element's slot");
        let slang_type = node.get_type().expect("the binder types every expression");
        let element_type = if slang_type.is_reference_type() {
            self.resolve_type(&slang_type, None)
        } else {
            slot.r#type().element_type(0)
        };
        (Place::from(slot), element_type)
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
