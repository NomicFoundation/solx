//!
//! Function call and member access expression lowering.
//!

pub mod built_in;
pub mod call_kind;
pub mod external_call;
pub mod library_call;
pub mod member_call_kind;
pub mod static_mode;

use anyhow::Context as _;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::ArgumentsDeclaration;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StructDefinition;
use solx_utils::DataLocation;

use self::call_kind::CallKind;
use self::member_call_kind::MemberCallKind;
use crate::ast::contract::function::expression::ExpressionEmitter;
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
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("only positional arguments supported");
        };

        match self.classify_call(call, positional_arguments) {
            CallKind::TypeConversion => {
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
            CallKind::New => unimplemented!("call dispatch: new expression"),
            CallKind::WithOptions(_) => {
                unimplemented!("call dispatch: call with options")
            }
            CallKind::ArrayTypeConversion => {
                unimplemented!("call dispatch: array type conversion")
            }
            CallKind::RefFieldStructAsConversion => {
                unimplemented!("call dispatch: reference-field struct as conversion")
            }
            CallKind::IndirectPointer => {
                unimplemented!("call dispatch: indirect function pointer")
            }
            CallKind::Member(member_kind) => {
                let Expression::MemberAccessExpression(access) = call.operand() else {
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
        let callee = call.operand();

        if call.is_type_conversion() && positional_arguments.len() == 1 {
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

        let Expression::Identifier(callee_identifier) = &callee else {
            unimplemented!("unsupported callee expression");
        };
        match callee_identifier.resolve_to_definition() {
            Some(Definition::Function(function_definition)) => {
                CallKind::LocalFunction(function_definition)
            }
            Some(Definition::Struct(struct_definition)) => {
                CallKind::StructConstructor(struct_definition)
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
                unimplemented!("external (delegatecall) library call")
            }
            MemberCallKind::Super => unimplemented!("super call"),
            MemberCallKind::FunctionPointer => unimplemented!("function pointer call"),
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
        let builder = &self.expression_emitter.state.builder;
        let struct_address = builder.emit_sol_malloc(result_type, &block);

        let mut block = block;
        for (index, (member, argument)) in struct_definition
            .members()
            .iter()
            .zip(positional_arguments.iter())
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
                self.expression_emitter.emit_value(&argument, block)?;
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
        let ArgumentsDeclaration::PositionalArguments(positional_arguments) = &call.arguments()
        else {
            unimplemented!("only positional arguments supported");
        };

        // A member-access callee yields its full result tuple through the same
        // dispatchers as the single-result path: a bare low-level call (member
        // resolves to a built-in) via `emit_bare_call_results`, any other member
        // via the shared `emit_member_call_results`.
        if let Expression::MemberAccessExpression(access) = call.operand() {
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

        let Expression::Identifier(callee_identifier) = call.operand() else {
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
        let (mut argument_values, block) = self.emit_argument_values(arguments, block)?;
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
        let (mlir_name, parameter_types, return_types) = self
            .expression_emitter
            .state
            .resolve_function(function_definition.node_id())?;

        let (argument_values, current_block) =
            self.emit_coerced_arguments(positional_arguments, parameter_types, block)?;

        Ok((mlir_name, argument_values, return_types, current_block))
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
