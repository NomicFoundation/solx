//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod call_kind;
pub mod external_call;
pub mod library_call;
pub mod member_call_kind;
pub mod static_mode;

use std::collections::HashMap;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NamedArguments;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::DataLocation;

use self::call_kind::CallKind;
use self::member_call_kind::MemberCallKind;
use crate::ast::contract::function::expression::ExpressionEmitter;
use crate::ast::expression_ext::ExpressionExt;
use crate::ast::type_conversion::TypeConversion;

/// Lowers function call and member access expressions to MLIR.
pub struct CallEmitter<'emitter, 'state, 'context, 'block> {
    /// The parent expression emitter for recursive subexpression emission.
    expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>,
}

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Creates a new call emitter.
    pub fn new(expression_emitter: &'emitter ExpressionEmitter<'state, 'context, 'block>) -> Self {
        Self { expression_emitter }
    }

    /// Emits a function call expression.
    ///
    /// Handles type conversions and built-in dispatch, then resolves
    /// user-defined callees through slang's binder to a function definition
    /// node id and looks up the registered MLIR signature.
    ///
    /// # Errors
    ///
    /// Returns an error if the callee is unsupported, arguments contain
    /// unsupported constructs, or the callee does not resolve to a registered
    /// function definition.
    pub fn emit_function_call(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let arguments = call.arguments();
        if let ArgumentsDeclaration::NamedArguments(named_arguments) = &arguments {
            return self.emit_named_call(call, named_arguments, block);
        }
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &arguments else {
            unreachable!("call arguments are either positional or named");
        };

        // A `recv.f{value: v}(args)` / `new C{value, salt}(args)` callee is a
        // `CallOptionsExpression`; capture the options and dispatch the inner
        // callee, taking the single result.
        if let Expression::CallOptionsExpression(call_options) = call.operand().unwrap_parens() {
            let (results, block) =
                self.emit_call_with_options(call, &call_options, positional_arguments, block)?;
            return Ok((results.into_iter().next(), block));
        }

        match self.classify_call(call, positional_arguments) {
            CallKind::TypeConversion | CallKind::RefFieldStructAsConversion => {
                // Both lower a single argument cast to the call's own type: a
                // genuine `T(x)` conversion and a reference-field struct `S(ref)`
                // that slang reports as a conversion and the backend cannot yet
                // construct (its `sol.copy` into a struct destination is NYI).
                let first = positional_arguments
                    .iter()
                    .next()
                    .expect("a type conversion has exactly one argument");
                let (value, block) = self.expression_emitter.emit_value(&first, block)?;
                let builder = &self.expression_emitter.state.builder;
                let target_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("slang types a type-conversion call");
                let result = TypeConversion::from_target_type(target_type, builder)
                    .emit(value, builder, &block);
                Ok((Some(result), block))
            }
            CallKind::BuiltInIdentifier(built_in) => {
                self.emit_built_in_call(built_in, positional_arguments, block)
            }
            CallKind::AbiDecode => {
                let (value, block) = self.emit_abi_decode(call, positional_arguments, block)?;
                Ok((Some(value), block))
            }
            CallKind::UdvtWrapUnwrap => {
                let argument = positional_arguments
                    .iter()
                    .next()
                    .expect("wrap/unwrap takes exactly one argument");
                let (value, block) = self.expression_emitter.emit_value(&argument, block)?;
                // A UDVT shares its underlying type's representation, so wrap/unwrap
                // is a single conversion of the argument to the call's result type
                // (the UDVT for `wrap`, its underlying type for `unwrap`). With no
                // resolved call type the value already has the right
                // representation, so it passes through unchanged.
                let result = match self.expression_emitter.resolve_slang_type(call.get_type()) {
                    Some(result_type) => {
                        let builder = &self.expression_emitter.state.builder;
                        TypeConversion::from_target_type(result_type, builder)
                            .emit(value, builder, &block)
                    }
                    None => value,
                };
                Ok((Some(result), block))
            }
            CallKind::BuiltInMemberAccess(access) => {
                self.emit_built_in_member_access(&access, Some(positional_arguments), block)
            }
            CallKind::LocalFunction(function_definition) => {
                let callee_name = function_definition
                    .name()
                    .map(|identifier| identifier.name())
                    .unwrap_or_default();
                let (mlir_name, argument_values, return_types, current_block) = self
                    .emit_call_setup(&function_definition, positional_arguments, block)
                    .with_context(|| format!("resolving callee '{callee_name}'"))?;

                if return_types.is_empty() {
                    self.expression_emitter.state.builder.emit_sol_call(
                        mlir_name,
                        &argument_values,
                        &[],
                        &current_block,
                    )?;
                    Ok((None, current_block))
                } else {
                    let result = self
                        .expression_emitter
                        .state
                        .builder
                        .emit_sol_call(mlir_name, &argument_values, return_types, &current_block)?
                        .expect("function call always produces at least one result");
                    Ok((Some(result), current_block))
                }
            }
            CallKind::StructConstructor(struct_definition) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("slang types a struct constructor call");
                self.emit_struct_constructor(
                    &struct_definition,
                    result_type,
                    positional_arguments,
                    block,
                )
                .map(|(value, block)| (Some(value), block))
            }
            CallKind::New => {
                self.expression_emitter
                    .emit_new(call, positional_arguments, None, None, block)
            }
            CallKind::WithOptions(_) => {
                unimplemented!("call dispatch: call with options")
            }
            CallKind::ArrayTypeConversion => {
                unimplemented!("call dispatch: array type conversion")
            }
            CallKind::IndirectPointer => {
                let callee = call.operand().unwrap_parens();
                let function_slang_type = callee
                    .get_type()
                    .expect("slang types every indirect-call callee");
                let (results, block) = self.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    positional_arguments,
                    None,
                    block,
                )?;
                Ok((results.into_iter().next(), block))
            }
            CallKind::Member(member_kind) => {
                let Expression::MemberAccessExpression(access) = call.operand().unwrap_parens()
                else {
                    unreachable!("a member call classifies only a member-access callee");
                };
                // Single-result position takes the first decoded value; the full
                // tuple flows through the one shared member-call dispatcher.
                let (results, block) = self.emit_member_call_results(
                    member_kind,
                    &access,
                    None,
                    positional_arguments,
                    block,
                )?;
                Ok((results.into_iter().next(), block))
            }
        }
    }

    /// Evaluates a `{value: …, gas: …, salt: …}` option list in source order
    /// (each value emitted for its side effects) and returns the captured
    /// `value` (as `msg.value`, coerced to `ui256`) and `salt` (the CREATE2 salt
    /// for `new`, cast from `bytes32`). The option KIND comes from slang's typed
    /// `BuiltIn::CallOption*` classification, never from comparing the option
    /// name as text (Rule-7). The `{gas: …}` option is not yet threaded into the
    /// call op and is deferred loudly.
    fn capture_call_options(
        &self,
        call_options: &CallOptionsExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        Option<Value<'context, 'block>>,
        Option<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    )> {
        let mut value = None;
        let mut salt = None;
        let mut current_block = block;
        for option in call_options.options().iter() {
            let (option_value, next_block) = self
                .expression_emitter
                .emit_value(&option.value(), current_block)?;
            current_block = next_block;
            let builder = &self.expression_emitter.state.builder;
            match option.name().resolve_to_built_in() {
                Some(BuiltIn::CallOptionValue) => {
                    value = Some(
                        TypeConversion::from_target_type(builder.types.ui256, builder).emit(
                            option_value,
                            builder,
                            &current_block,
                        ),
                    );
                }
                Some(BuiltIn::CallOptionSalt) => {
                    salt = Some(builder.emit_sol_bytes_cast(
                        option_value,
                        builder.types.ui256,
                        &current_block,
                    ));
                }
                Some(BuiltIn::CallOptionGas) => {
                    unimplemented!("the `{{gas: …}}` call option is not yet threaded into the call")
                }
                _ => unreachable!("a call option resolves to a value, gas, or salt built-in"),
            }
        }
        Ok((value, salt, current_block))
    }

    /// Lowers a call whose callee is a `CallOptionsExpression`
    /// (`recv.f{value: v}(args)`, `addr.call{value: v}(data)`,
    /// `new C{value: v, salt: s}(args)`, `fp{value: v}(args)`): captures the
    /// options, then dispatches the inner callee, forwarding the `value` as
    /// `msg.value` (and, for contract creation, the `salt`). Returns the full
    /// result tuple.
    fn emit_call_with_options(
        &self,
        call: &FunctionCallExpression,
        call_options: &CallOptionsExpression,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (value, salt, block) = self.capture_call_options(call_options, block)?;
        let callee = call_options.operand();
        match &callee {
            Expression::NewExpression(_) => {
                let (result, block) = self.expression_emitter.emit_new(
                    call,
                    positional_arguments,
                    value,
                    salt,
                    block,
                )?;
                Ok((result.into_iter().collect(), block))
            }
            Expression::MemberAccessExpression(access) => {
                if let Some(
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall),
                ) = access.member().resolve_to_built_in()
                {
                    return self.emit_bare_call_results(
                        access,
                        kind,
                        value,
                        positional_arguments,
                        block,
                    );
                }
                let member_kind = self.classify_member(access);
                self.emit_member_call_results(
                    member_kind,
                    access,
                    value,
                    positional_arguments,
                    block,
                )
            }
            _ if matches!(callee.get_type(), Some(SlangType::Function(_))) => {
                let function_slang_type = callee
                    .get_type()
                    .expect("an indirect call-options callee is function-typed");
                self.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    positional_arguments,
                    value,
                    block,
                )
            }
            _ => unimplemented!("unsupported call-options callee"),
        }
    }

    /// Classifies a call expression into its [`CallKind`] ahead of emission,
    /// from positive, mutually-exclusive resolution facts rather than a
    /// speculative chain of fallible attempts.
    ///
    /// The arms are ordered to preserve the original dispatch precedence:
    /// type conversion, then identifier built-in, then the `abi.decode`
    /// member built-in, then other member built-ins, then a user-defined
    /// function or struct constructor. An unsupported callee shape is a loud
    /// `unimplemented!`.
    fn classify_call(
        &self,
        call: &FunctionCallExpression,
        positional_arguments: &PositionalArguments,
    ) -> CallKind {
        let callee = call.operand().unwrap_parens();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
            // A single-argument struct constructor `S(x)` is reported as a type
            // conversion by slang (the struct name types as a `UserMetaType`),
            // yet it must build a struct, not cast its argument to the struct
            // type (`sol.cast` is integer-only and rejects a struct result).
            // Multi-field constructors carry more than one argument and never
            // reach here. When the struct's sole field is a reference type the
            // construction would `sol.copy` an aggregate into a struct
            // destination the backend cannot yet lower, so it stays on the
            // conversion path as `RefFieldStructAsConversion`.
            if let Expression::Identifier(identifier) = &callee
                && let Some(Definition::Struct(struct_definition)) =
                    identifier.resolve_to_definition()
            {
                let first_field_is_value = struct_definition
                    .members()
                    .iter()
                    .next()
                    .and_then(|member| member.get_type())
                    .is_some_and(|field_type| !field_type.is_reference_type());
                return if first_field_is_value {
                    CallKind::StructConstructor(struct_definition)
                } else {
                    CallKind::RefFieldStructAsConversion
                };
            }
            return CallKind::TypeConversion;
        }

        if let Expression::Identifier(identifier) = &callee
            && let Some(built_in) = identifier.resolve_to_built_in()
            && Self::is_emittable_identifier_built_in(built_in, positional_arguments.len())
        {
            return CallKind::BuiltInIdentifier(built_in);
        }

        if let Expression::MemberAccessExpression(access) = &callee {
            return match access.member().resolve_to_built_in() {
                Some(BuiltIn::AbiDecode) => CallKind::AbiDecode,
                // `T.wrap(x)` / `T.unwrap(x)` take their result type from the call
                // itself (the UDVT or its underlying type), so they classify apart
                // from the member-access built-ins whose result follows from their
                // operands — like `abi.decode` above.
                Some(BuiltIn::Wrap | BuiltIn::Unwrap) => CallKind::UdvtWrapUnwrap,
                // A member resolving to any other built-in is an intrinsic
                // (`abi.encode`, `arr.push`, `a.transfer`, `msg.sender`); one
                // resolving to a definition (a function or state variable) is a
                // user / external member call dispatched by `classify_member`.
                Some(_) => CallKind::BuiltInMemberAccess(access.clone()),
                None => CallKind::Member(self.classify_member(access)),
            };
        }

        if let Expression::NewExpression(_) = &callee {
            return CallKind::New;
        }

        let Expression::Identifier(callee_identifier) = &callee else {
            // A non-identifier callee of function type — `arr[i](args)`,
            // `(cond ? f : g)(args)` — is an indirect call through the pointer
            // value the callee evaluates to.
            if matches!(callee.get_type(), Some(SlangType::Function(_))) {
                return CallKind::IndirectPointer;
            }
            unimplemented!("unsupported callee expression");
        };
        match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => {
                CallKind::LocalFunction(function_definition)
            }
            Some(Definition::Struct(struct_definition)) => {
                CallKind::StructConstructor(struct_definition)
            }
            // A function-typed variable / parameter / state variable named as a
            // callee (`fp(args)`) is an indirect call through its stored pointer
            // value, distinct from a direct call to a function definition above.
            Some(
                Definition::Variable(_) | Definition::Parameter(_) | Definition::StateVariable(_),
            ) if matches!(callee_identifier.get_type(), Some(SlangType::Function(_))) => {
                CallKind::IndirectPointer
            }
            _ => unimplemented!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            ),
        }
    }

    /// Classifies a member-access callee (`x.f(...)`) into its
    /// [`MemberCallKind`] ahead of emission, mirroring [`Self::classify_call`].
    pub fn classify_member(&self, access: &MemberAccessExpression) -> MemberCallKind {
        let operand = access.operand();
        // `super.f()` dispatches up the C3 linearisation (inheritance cluster),
        // distinct from an external call on a contract instance.
        if matches!(operand, Expression::SuperKeyword(_)) {
            return MemberCallKind::Super;
        }
        // `L.f()` on a library lowers to an internal or delegatecall library
        // call, not an external instance call. An external/public library
        // function (one with a selector) is delegatecalled; an internal one is
        // inlined.
        if let Expression::Identifier(identifier) = &operand
            && matches!(
                identifier.resolve_to_definition(),
                Some(Definition::Library(_))
            )
        {
            let external = matches!(
                access.member().resolve_to_definition(),
                Some(Definition::Function(function)) if function.compute_selector().is_some()
            );
            return MemberCallKind::Library { external };
        }
        let is_this = matches!(operand, Expression::ThisKeyword(_));
        match access.member().resolve_to_definition() {
            Some(Definition::Function(function)) => {
                if is_this {
                    MemberCallKind::SelfExternal
                } else if function.compute_selector().is_none() {
                    // `x.f(...)` using-for on an internal (no-selector) library
                    // function: an internal contract function cannot be reached
                    // via member access, so this is a library call with `x` as
                    // the implicit `self`.
                    MemberCallKind::Library { external: false }
                } else {
                    MemberCallKind::ExternalInstance
                }
            }
            Some(Definition::StateVariable(_)) => {
                if is_this {
                    MemberCallKind::SelfGetter
                } else {
                    MemberCallKind::ExternalGetter
                }
            }
            // `s.f(...)` where `f` is a function-pointer struct field: a call
            // through the member-resolved pointer, classified by the member's
            // function type (never by the member name).
            Some(Definition::StructMember(_))
                if matches!(access.get_type(), Some(SlangType::Function(_))) =>
            {
                MemberCallKind::FunctionPointer
            }
            other => unimplemented!(
                "unsupported member call: {:?}",
                other.map(|definition| definition.node_id())
            ),
        }
    }

    /// The single member-call dispatcher: lowers a member-access callee
    /// (`recv.f(args)` / `this.f(args)` / `L.f(args)` / a public getter) to its
    /// full result tuple. [`Self::emit_function_call`] takes the first value for
    /// single-result position and [`Self::emit_function_call_results`] takes the
    /// whole tuple — both route here, so classify-and-dispatch lives in one place.
    fn emit_member_call_results(
        &self,
        member_kind: MemberCallKind,
        access: &MemberAccessExpression,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        match member_kind {
            // `this.f(args)` and `c.f(args)` are both genuine external calls; the
            // single results helper computes the signature from the callee
            // definition for either receiver (self or foreign instance).
            MemberCallKind::SelfExternal | MemberCallKind::ExternalInstance => {
                let Some(Definition::Function(function_definition)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("an external member call resolves to a function");
                };
                self.emit_external_call_results(
                    access,
                    &function_definition,
                    call_value,
                    arguments,
                    block,
                )
            }
            MemberCallKind::SelfGetter => {
                let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("a self getter call resolves to a state variable");
                };
                self.emit_self_getter_call(access, &state_variable, arguments, call_value, block)
            }
            MemberCallKind::ExternalGetter => {
                let Some(Definition::StateVariable(state_variable)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("an external getter call resolves to a state variable");
                };
                let (value, block) =
                    self.emit_external_getter_call(access, &state_variable, arguments, block)?;
                Ok((value.into_iter().collect(), block))
            }
            MemberCallKind::Library { external: false } => {
                let Some(Definition::Function(library_function)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("a library call resolves to a function");
                };
                self.emit_library_call(access, &library_function, arguments, block)
            }
            MemberCallKind::Library { external: true } => {
                // `L.f(args)` on a library whose `f` is external/public: a
                // `delegatecall` to the deployed library, whose link symbol is
                // the fully-qualified `"<file>:<Library>"`.
                let Expression::Identifier(identifier) = access.operand() else {
                    unreachable!("a direct external library call has a library-name operand");
                };
                let Some(Definition::Library(library)) = identifier.resolve_to_definition() else {
                    unreachable!("an external library call's operand resolves to a library");
                };
                let Some(Definition::Function(library_function)) =
                    access.member().resolve_to_definition()
                else {
                    unreachable!("an external library call resolves to a function");
                };
                let library_name = format!("{}:{}", library.get_file_id(), library.name().name());
                self.emit_library_external_call(
                    &library_name,
                    &library_function,
                    arguments,
                    None,
                    block,
                )
            }
            MemberCallKind::Super => unimplemented!("super call"),
            MemberCallKind::FunctionPointer => {
                // `s.f(args)` through a function-pointer struct field: evaluate
                // the member access (gep + load of the `func_ref`) as the callee
                // value, then `sol.icall` / `sol.ext_icall` per the pointer's
                // visibility — exactly the indirect-call path used for a plain
                // function-pointer identifier callee.
                let callee = Expression::MemberAccessExpression(access.clone());
                let function_slang_type = access
                    .get_type()
                    .expect("a function-pointer member call is function-typed");
                self.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    arguments,
                    call_value,
                    block,
                )
            }
        }
    }

    /// Returns whether an identifier-callee built-in is lowered directly with
    /// the given argument count. Built-ins not listed here (or with a
    /// mismatched arity) fall through to user-defined function resolution,
    /// preserving the original dispatch behavior.
    fn is_emittable_identifier_built_in(built_in: BuiltIn, argument_count: usize) -> bool {
        matches!(
            (built_in, argument_count),
            (BuiltIn::Assert, 1)
                | (BuiltIn::Require, 1 | 2)
                | (BuiltIn::Gasleft, 0)
                | (BuiltIn::Blockhash, 1)
                | (BuiltIn::Keccak256, 1)
                | (BuiltIn::Sha256, 1)
                | (BuiltIn::Ripemd160, 1)
                | (BuiltIn::Ecrecover, 4)
                | (BuiltIn::Addmod, 3)
                | (BuiltIn::Mulmod, 3)
        )
    }

    /// Emits a struct-literal constructor `S(a, b, c)` in memory.
    fn emit_struct_constructor(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let arguments: Vec<Expression> = positional_arguments.iter().collect();
        self.emit_struct_constructor_expressions(struct_definition, result_type, &arguments, block)
    }

    /// Emits a struct-literal constructor from its field initializers already in
    /// member-declaration order, storing each into the malloc'd struct. Shared by
    /// positional `S(a, b)` and named `S({b: …, a: …})` construction (the latter
    /// reorders its arguments to member order first).
    fn emit_struct_constructor_expressions(
        &self,
        struct_definition: &StructDefinition,
        result_type: Type<'context>,
        arguments: &[Expression],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(arguments.iter())
            .enumerate()
        {
            let field_slang_type = member.get_type().expect("slang types every struct member");
            let field_type = TypeConversion::resolve_slang_type(
                &field_slang_type,
                Some(DataLocation::Memory),
                builder,
            );
            let index_value = builder.emit_sol_constant(index as i64, builder.types.ui64, &block);
            let field_address =
                builder.emit_sol_gep(struct_address, index_value, field_type, &block);

            let (argument_value, next_block) =
                self.expression_emitter.emit_value(argument, block)?;
            block = next_block;
            let stored = TypeConversion::from_target_type(field_type, builder).emit(
                argument_value,
                builder,
                &block,
            );
            builder.emit_sol_store(stored, field_address, &block);
        }

        Ok((struct_address, block))
    }

    /// Emits a direct, named function call and returns all of its result
    /// values in declaration order.
    ///
    /// Unlike [`Self::emit_function_call`], this entry point does not handle
    /// explicit type conversions or built-in dispatch — it is intended for
    /// callers that need the full result tuple (e.g. tuple deconstruction).
    ///
    /// # Errors
    ///
    /// Returns an error if the call uses non-positional arguments, if the
    /// callee is not a named identifier, or if name resolution fails.
    pub fn emit_function_call_results(
        &self,
        call: &FunctionCallExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let arguments = call.arguments();
        if let ArgumentsDeclaration::NamedArguments(named_arguments) = &arguments {
            return self.emit_named_call_results(call, named_arguments, block);
        }
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &arguments else {
            unreachable!("call arguments are either positional or named");
        };

        // A `recv.f{value: v}(args)` / `new C{value, salt}(args)` callee in
        // result-binding position dispatches through the same options handler.
        if let Expression::CallOptionsExpression(call_options) = call.operand().unwrap_parens() {
            return self.emit_call_with_options(call, &call_options, positional_arguments, block);
        }

        // A member-access callee yields its full result tuple through the same
        // dispatchers as the single-result path: a bare low-level call (member
        // resolves to a built-in) via `emit_bare_call_results`, any other member
        // via the shared `emit_member_call_results`.
        if let Expression::MemberAccessExpression(access) = call.operand().unwrap_parens() {
            if let Some(built_in) = access.member().resolve_to_built_in() {
                return match built_in {
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall) => self.emit_bare_call_results(
                        &access,
                        kind,
                        None,
                        positional_arguments,
                        block,
                    ),
                    _ => unimplemented!("multi-result built-in member call is not yet supported"),
                };
            }
            let member_kind = self.classify_member(&access);
            return self.emit_member_call_results(
                member_kind,
                &access,
                None,
                positional_arguments,
                block,
            );
        }

        let Expression::Identifier(callee_identifier) = call.operand().unwrap_parens() else {
            unimplemented!("multi-result calls only support direct named or member callees");
        };
        let Some(Definition::Function(function_definition)) =
            callee_identifier.resolve_to_definition()
        else {
            unimplemented!(
                "callee '{}' does not resolve to a function",
                callee_identifier.name()
            );
        };

        let (mlir_name, argument_values, return_types, current_block) = self
            .emit_call_setup(&function_definition, positional_arguments, block)
            .with_context(|| format!("resolving callee '{}'", callee_identifier.name()))?;

        let results = self
            .expression_emitter
            .state
            .builder
            .emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
        Ok((results, current_block))
    }

    /// Evaluates `arguments` left-to-right (via
    /// [`CallEmitter::emit_argument_values`]) and coerces each resulting value to
    /// its declared parameter type, returning the materialised argument values
    /// and the continuation block.
    ///
    /// The single argument eval-and-coerce primitive: every call site (internal,
    /// external, library, struct-constructor) delegates here rather than
    /// re-implementing the evaluation and zip-coerce loops. `pub` so the call
    /// fills in sibling modules reuse it.
    pub fn emit_coerced_arguments(
        &self,
        arguments: &PositionalArguments,
        parameter_types: &[melior::ir::Type<'context>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let arguments: Vec<Expression> = arguments.iter().collect();
        self.emit_coerced_argument_expressions(&arguments, parameter_types, block)
    }

    /// Evaluates `arguments` left-to-right and coerces each value to its declared
    /// parameter type. The expression-keyed core of [`Self::emit_coerced_arguments`]:
    /// named calls feed it a reordered argument list, positional calls the source
    /// order.
    pub fn emit_coerced_argument_expressions(
        &self,
        arguments: &[Expression],
        parameter_types: &[melior::ir::Type<'context>],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let mut argument_values = Vec::with_capacity(arguments.len());
        let mut block = block;
        for argument in arguments {
            let (value, next_block) = self.expression_emitter.emit_value(argument, block)?;
            argument_values.push(value);
            block = next_block;
        }
        let builder = &self.expression_emitter.state.builder;
        for (value, &parameter_type) in argument_values.iter_mut().zip(parameter_types) {
            let conversion = TypeConversion::from_target_type(parameter_type, builder);
            *value = conversion.emit(*value, builder, &block);
        }
        Ok((argument_values, block))
    }

    /// Resolves the callee's MLIR signature, then evaluates and coerces the
    /// arguments to its declared parameter types.
    ///
    /// Returns the resolved MLIR name, the coerced argument values, the
    /// declared return types, and the block in which the call should be
    /// emitted.
    fn emit_call_setup<'a>(
        &'a self,
        function_definition: &FunctionDefinition,
        positional_arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [melior::ir::Type<'context>],
        BlockRef<'context, 'block>,
    )> {
        let arguments: Vec<Expression> = positional_arguments.iter().collect();
        self.emit_call_setup_expressions(function_definition, &arguments, block)
    }

    /// Resolves the callee's MLIR signature and evaluates/coerces its arguments,
    /// already in parameter-declaration order. The expression-keyed core of
    /// [`Self::emit_call_setup`], shared with the named-argument call path.
    fn emit_call_setup_expressions<'a>(
        &'a self,
        function_definition: &FunctionDefinition,
        arguments: &[Expression],
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        &'a str,
        Vec<Value<'context, 'block>>,
        &'a [melior::ir::Type<'context>],
        BlockRef<'context, 'block>,
    )> {
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id())?;

        let (argument_values, current_block) =
            self.emit_coerced_argument_expressions(arguments, parameter_types, block)?;

        Ok((mlir_name, argument_values, return_types, current_block))
    }

    /// Routes a single-result call with named arguments `f({…})` / `S({…})` to
    /// the matching emitter. Named arguments only apply to a direct function or
    /// struct-constructor identifier; a named `using for` member call is not yet
    /// supported.
    fn emit_named_call(
        &self,
        call: &FunctionCallExpression,
        named_arguments: &NamedArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(callee_identifier) = call.operand().unwrap_parens() else {
            unimplemented!(
                "named arguments are only supported on a direct function or struct call"
            );
        };
        match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => {
                let (results, block) =
                    self.emit_named_function_call(&function_definition, named_arguments, block)?;
                Ok((results.into_iter().next(), block))
            }
            Some(Definition::Struct(struct_definition)) => {
                let result_type = self
                    .expression_emitter
                    .resolve_slang_type(call.get_type())
                    .expect("slang types a struct constructor call");
                let member_ids: Vec<NodeId> = struct_definition
                    .members()
                    .iter()
                    .map(|member| member.node_id())
                    .collect();
                let arguments = Self::order_named_arguments(named_arguments, &member_ids);
                let (value, block) = self.emit_struct_constructor_expressions(
                    &struct_definition,
                    result_type,
                    &arguments,
                    block,
                )?;
                Ok((Some(value), block))
            }
            _ => unimplemented!(
                "named arguments are only supported on a function or struct-constructor callee"
            ),
        }
    }

    /// Routes a multi-result call with named arguments (e.g. tuple
    /// deconstruction `(a, b) = f({…})`) to the named function-call emitter.
    fn emit_named_call_results(
        &self,
        call: &FunctionCallExpression,
        named_arguments: &NamedArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Expression::Identifier(callee_identifier) = call.operand().unwrap_parens() else {
            unimplemented!("named multi-result calls require a direct function callee");
        };
        let Some(Definition::Function(function_definition)) =
            callee_identifier.resolve_to_definition()
        else {
            unimplemented!("named multi-result calls require a function callee");
        };
        self.emit_named_function_call(&function_definition, named_arguments, block)
    }

    /// Emits a direct function call whose arguments are passed by name,
    /// returning all result values. The named arguments are reordered into the
    /// callee's parameter-declaration order, then evaluated and coerced through
    /// the same setup path as a positional call.
    fn emit_named_function_call(
        &self,
        function_definition: &FunctionDefinition,
        named_arguments: &NamedArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let parameter_ids: Vec<NodeId> = function_definition
            .parameters()
            .iter()
            .map(|parameter| parameter.node_id())
            .collect();
        let arguments = Self::order_named_arguments(named_arguments, &parameter_ids);
        let (mlir_name, argument_values, return_types, current_block) =
            self.emit_call_setup_expressions(function_definition, &arguments, block)?;
        let results = self
            .expression_emitter
            .state
            .builder
            .emit_sol_call_results(mlir_name, &argument_values, return_types, &current_block)?;
        Ok((results, current_block))
    }

    /// Reorders named call arguments into the declaration order given by
    /// `ordered_definition_ids` (the callee's parameter or struct-member node
    /// ids). Each argument binds to its target through slang's typed resolution,
    /// keyed by the resolved definition's [`NodeId`], never by comparing name
    /// strings. slang has already validated the binding, so a missing or unknown
    /// name is unreachable.
    fn order_named_arguments(
        named_arguments: &NamedArguments,
        ordered_definition_ids: &[NodeId],
    ) -> Vec<Expression> {
        let mut by_definition: HashMap<NodeId, Expression> = HashMap::new();
        for argument in named_arguments.iter() {
            let definition = argument
                .name()
                .resolve_to_definition()
                .expect("slang resolves every named argument to its target definition");
            by_definition.insert(definition.node_id(), argument.value());
        }
        ordered_definition_ids
            .iter()
            .map(|definition_id| {
                by_definition
                    .remove(definition_id)
                    .expect("slang binds a named argument for every declared name")
            })
            .collect()
    }

    /// Emits an indirect call through the function-pointer value `callee`
    /// yields, returning the result values. Parameter and result types come
    /// from the pointer's function type (a void return is zero results; a tuple
    /// return expands per element). Internal pointers dispatch through
    /// `sol.icall`; external ones through `sol.ext_icall`, forwarding
    /// `call_value` (or zero) as `msg.value`.
    ///
    /// # Errors
    ///
    /// Returns an error if a subexpression or the call op cannot be emitted.
    fn emit_indirect_call_results(
        &self,
        callee: &Expression,
        function_slang_type: &SlangType,
        positional_arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (callee_value, block) = self.expression_emitter.emit_value(callee, block)?;
        let SlangType::Function(function_type) = function_slang_type else {
            unreachable!("an indirect-call callee is always a function type");
        };
        let builder = &self.expression_emitter.state.builder;
        let parameter_types: Vec<Type<'context>> = function_type
            .parameter_types()
            .iter()
            .map(|parameter_type| TypeConversion::resolve_slang_type(parameter_type, None, builder))
            .collect();
        let result_types: Vec<Type<'context>> = match function_type.return_type() {
            SlangType::Void(_) => Vec::new(),
            SlangType::Tuple(tuple_type) => tuple_type
                .types()
                .iter()
                .map(|element_type| TypeConversion::resolve_slang_type(element_type, None, builder))
                .collect(),
            other => vec![TypeConversion::resolve_slang_type(&other, None, builder)],
        };
        let (argument_values, current_block) =
            self.emit_coerced_arguments(positional_arguments, &parameter_types, block)?;
        let builder = &self.expression_emitter.state.builder;
        let results = if function_type.is_externally_visible() {
            // `fp{value: v}(args)` forwards `v`; a plain `fp(args)` sends zero.
            let value = call_value.unwrap_or_else(|| {
                builder.emit_sol_constant(0, builder.types.ui256, &current_block)
            });
            builder.emit_sol_ext_icall(
                callee_value,
                &argument_values,
                &result_types,
                value,
                false,
                &current_block,
            )?
        } else {
            builder.emit_sol_icall(
                callee_value,
                &argument_values,
                &result_types,
                &current_block,
            )?
        };
        Ok((results, current_block))
    }

    /// Emits a bare member access expression (e.g. `tx.origin`, `msg.sender`).
    ///
    /// # Errors
    ///
    /// Returns an error if the member access is not a recognized EVM intrinsic.
    pub fn emit_member_access(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Value<'context, 'block>, BlockRef<'context, 'block>)> {
        let (value, block) = self.emit_built_in_member_access(access, None, block)?;
        Ok((
            value.expect("bare member access always produces a value"),
            block,
        ))
    }
}
