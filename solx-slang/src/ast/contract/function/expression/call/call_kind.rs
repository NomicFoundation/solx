//!
//! The kind of a function-call expression, resolved ahead of emission.
//!

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::Toward;
use crate::ast::arguments_declaration_ext::ArgumentsDeclarationExt;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::member_call_kind::MemberCallKind;
use crate::ast::type_conversion::TypeConversion;

/// The resolved kind of a `FunctionCallExpression`, computed once so dispatch is
/// a single match rather than a chain of fallible attempts.
pub enum CallKind {
    /// `T(x)` — an explicit type conversion.
    TypeConversion,
    /// An identifier built-in lowered directly (`keccak256`, `require`, …).
    BuiltInIdentifier(BuiltIn),
    /// `abi.decode(payload, (T))` — result type from the call, not the operands.
    AbiDecode,
    /// `T.wrap(x)` / `T.unwrap(x)` — result type from the call; a single conversion.
    UdvtWrapUnwrap,
    /// Any other member-access built-in (`abi.encode`, `arr.push`, `msg.sender`, …).
    BuiltInMemberAccess(MemberAccessExpression),
    /// A direct call to a user-defined function.
    LocalFunction(FunctionDefinition),
    /// A struct constructor `S(...)`.
    StructConstructor(StructDefinition),
    /// `new T(...)` — contract creation / `new T[](n)` array allocation.
    New,
    /// `T[](x)` — an empty-bracket array type used as a data-location cast.
    ArrayTypeConversion,
    /// A call through a function-pointer value.
    IndirectPointer,
    /// A bare low-level call `addr.call(data)` / `.delegatecall` / `.staticcall`,
    /// yielding `(bool success, bytes data)`.
    BareCall(BuiltIn),
    /// A member call `x.f(...)`, dispatched by [`MemberCallKind`].
    Member(MemberCallKind),
}

impl CallKind {
    /// Resolves a function-call expression to its kind from slang's binder.
    pub fn new<'state, 'context, 'block>(
        context: &ExpressionContext<'state, 'context, 'block>,
        call: &FunctionCallExpression,
    ) -> Self {
        // `recv.f{value: v}(args)` / `new C{value, salt}(args)`: the options wrap
        // the real callee. Classify the inner callee — the value/salt are runtime
        // values captured at emission, not part of the classification.
        let callee = match call.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => options.operand().unwrap_parentheses(),
            other => other,
        };

        // `S(x)` types as a conversion but constructs a struct; `T(x)` converts.
        if call.is_type_conversion()
            && let ArgumentsDeclaration::PositionalArguments(arguments) = call.arguments()
            && arguments.len() == 1
        {
            let struct_callee = match &callee {
                Expression::Identifier(identifier) => identifier.resolve_to_definition(),
                Expression::MemberAccessExpression(access) => {
                    access.member().resolve_to_definition()
                }
                _ => None,
            };
            return match struct_callee {
                Some(Definition::Struct(definition)) => Self::StructConstructor(definition),
                _ => Self::TypeConversion,
            };
        }

        if let Expression::Identifier(identifier) = &callee
            && let Some(built_in) = identifier.resolve_to_built_in()
            && matches!(
                built_in,
                BuiltIn::Assert
                    | BuiltIn::Require
                    | BuiltIn::Gasleft
                    | BuiltIn::Blockhash
                    | BuiltIn::Keccak256
                    | BuiltIn::Sha256
                    | BuiltIn::Ripemd160
                    | BuiltIn::Ecrecover
                    | BuiltIn::Addmod
                    | BuiltIn::Mulmod
            )
        {
            return Self::BuiltInIdentifier(built_in);
        }

        if let Expression::MemberAccessExpression(access) = &callee {
            return match access.member().resolve_to_built_in() {
                Some(
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall),
                ) => Self::BareCall(kind),
                Some(BuiltIn::AbiDecode) => Self::AbiDecode,
                Some(BuiltIn::Wrap | BuiltIn::Unwrap) => Self::UdvtWrapUnwrap,
                Some(_) => Self::BuiltInMemberAccess(access.clone()),
                // A namespace-qualified struct constructor `Lib.S(...)` builds the
                // struct like the bare `S(...)`; the member resolves to the struct
                // regardless of the qualifying contract.
                None => match access.member().resolve_to_definition() {
                    Some(Definition::Struct(definition)) => Self::StructConstructor(definition),
                    _ => Self::Member(MemberCallKind::new(context, access)),
                },
            };
        }

        if let Expression::NewExpression(_) = &callee {
            return Self::New;
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            // `T[](x)`: an empty-bracket array type used as a data-location cast.
            if let Expression::IndexAccessExpression(array_type) = &callee
                && array_type.start().is_none()
                && array_type.end().is_none()
                && !array_type.is_slice()
            {
                return Self::ArrayTypeConversion;
            }
            // `arr[i](args)` / `(cond ? f : g)(args)`: a call through a pointer value.
            if matches!(callee.get_type(), Some(SlangType::Function(_))) {
                return Self::IndirectPointer;
            }
            unimplemented!("unsupported callee expression");
        };
        match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(definition)) => Self::LocalFunction(definition),
            Some(Definition::Struct(definition)) => Self::StructConstructor(definition),
            // A function-typed variable / parameter / state variable calls through
            // its stored pointer.
            Some(
                Definition::Variable(_) | Definition::Parameter(_) | Definition::StateVariable(_),
            ) if matches!(callee_identifier.get_type(), Some(SlangType::Function(_))) => {
                Self::IndirectPointer
            }
            _ => unimplemented!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            ),
        }
    }

    /// Lowers this kind to its result values — zero for a void callee, more than
    /// one for a multi-return call. A single-result position takes the first.
    pub fn emit<'state, 'context, 'block>(
        self,
        context: &ExpressionContext<'state, 'context, 'block>,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // `recv.f{value: v}(args)` / `new C{value, salt}(args)`: evaluate the
        // option list (each for its side effects, in source order) before the
        // arguments, forwarding `value` as msg.value and `salt` as the CREATE2
        // salt. The inner callee drives the dispatch below.
        let (call_value, salt, block, callee) = match call.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => {
                let (value, salt, block) = context.capture_call_options(&options, block)?;
                (value, salt, block, options.operand().unwrap_parentheses())
            }
            other => (None, None, block, other),
        };
        let arguments = call.arguments();

        match self {
            // A direct call passes its arguments by position or by name; ordering
            // them against the parameter ids collapses both into one path.
            Self::LocalFunction(function_definition) => {
                let callee_name = function_definition
                    .name()
                    .map(|identifier| identifier.name())
                    .unwrap_or_default();
                let parameter_ids: Vec<NodeId> = function_definition
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let arguments = arguments.ordered_by(&parameter_ids);
                let (mlir_name, argument_values, return_types, block) = context
                    .emit_call_setup_expressions(&function_definition, &arguments, block)
                    .with_context(|| format!("resolving callee '{callee_name}'"))?;
                let results = context.state.builder.emit_sol_call_results(
                    mlir_name,
                    &argument_values,
                    return_types,
                    &block,
                )?;
                Ok((results, block))
            }
            // `S(a, b)` / `S({b: …, a: …})`: order the field initialisers by the
            // struct's member-declaration order, then store each.
            Self::StructConstructor(struct_definition) => {
                let result_type = TypeConversion::resolve_optional_slang_type(
                    call.get_type(),
                    &context.state.builder,
                )
                .expect("slang types a struct constructor call");
                let member_ids: Vec<NodeId> = struct_definition
                    .members()
                    .iter()
                    .map(|member| member.node_id())
                    .collect();
                let arguments = arguments.ordered_by(&member_ids);
                let (value, block) = context.emit_struct_constructor_expressions(
                    &struct_definition,
                    result_type,
                    &arguments,
                    block,
                )?;
                Ok((vec![value], block))
            }
            // `x.f(...)` / `L.f(...)`: dispatched by the resolved member kind,
            // forwarding any `{value: …}` captured from the options.
            Self::Member(member_kind) => {
                let Expression::MemberAccessExpression(access) = &callee else {
                    unreachable!("a member call has a member-access callee");
                };
                member_kind.emit(context, access, call_value, &arguments, block)
            }
            // Every remaining kind takes positional arguments only.
            positional_kind => {
                let ArgumentsDeclaration::PositionalArguments(arguments) = &arguments else {
                    unimplemented!(
                        "named arguments are only supported on a function, struct, or external-library callee"
                    );
                };
                match positional_kind {
                    Self::TypeConversion | Self::ArrayTypeConversion => {
                        // Cast the single argument to the call's own type (`T(x)`,
                        // or the data-location cast `T[](x)`).
                        let first = arguments
                            .iter()
                            .next()
                            .expect("a type conversion has exactly one argument");
                        let target_type = TypeConversion::resolve_optional_slang_type(
                            call.get_type(),
                            &context.state.builder,
                        )
                        .expect("slang types a type-conversion call");
                        // A `bytesN("…")` literal folds to a fixed-bytes constant.
                        let BlockAnd { value, block } = (Toward {
                            expression: &first,
                            target_type,
                        })
                        .emit(context, block)?;
                        let result = value
                            .coerce_to(
                                crate::ast::Type::new(target_type),
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir();
                        Ok((vec![result], block))
                    }
                    Self::BuiltInIdentifier(built_in) => {
                        let (value, block) =
                            context.emit_built_in_call(built_in, arguments, block)?;
                        Ok((value.into_iter().collect(), block))
                    }
                    Self::AbiDecode => context.emit_abi_decode(call, arguments, block),
                    Self::UdvtWrapUnwrap => {
                        let argument = arguments
                            .iter()
                            .next()
                            .expect("wrap/unwrap takes exactly one argument");
                        let BlockAnd { value, block } = argument.emit(context, block)?;
                        // A UDVT shares its underlying type's representation, so this
                        // is one conversion to the result type (none ⇒ already correct).
                        let result = match TypeConversion::resolve_optional_slang_type(
                            call.get_type(),
                            &context.state.builder,
                        ) {
                            Some(result_type) => value
                                .coerce_to(
                                    crate::ast::Type::new(result_type),
                                    &context.state.builder,
                                    &block,
                                )
                                .into_mlir(),
                            None => value.into_mlir(),
                        };
                        Ok((vec![result], block))
                    }
                    Self::BuiltInMemberAccess(access) => {
                        let (value, block) =
                            context.emit_built_in_member_access(&access, Some(arguments), block)?;
                        Ok((value.into_iter().collect(), block))
                    }
                    Self::New => {
                        let (value, block) =
                            context.emit_new(call, arguments, call_value, salt, block)?;
                        Ok((value.into_iter().collect(), block))
                    }
                    Self::IndirectPointer => {
                        let function_slang_type = callee
                            .get_type()
                            .expect("slang types every indirect-call callee");
                        context.emit_indirect_call_results(
                            &callee,
                            &function_slang_type,
                            arguments,
                            call_value,
                            block,
                        )
                    }
                    Self::BareCall(kind) => {
                        let Expression::MemberAccessExpression(access) = &callee else {
                            unreachable!("a bare low-level call has a member-access callee");
                        };
                        context.emit_bare_call_results(access, kind, call_value, arguments, block)
                    }
                    Self::LocalFunction(_) | Self::StructConstructor(_) | Self::Member(_) => {
                        unreachable!("handled in the outer match")
                    }
                }
            }
        }
    }
}
