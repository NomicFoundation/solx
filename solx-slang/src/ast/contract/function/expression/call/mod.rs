//!
//! Function call and member access expression emission.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod built_in;
pub mod external_call;
pub mod library_call;
pub mod positional_arguments;
pub mod try_external_call;

use melior::ir::BlockLike;
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
use slang_solidity_v2::ast::IndexAccessKind;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::NodeId;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::Function;
use solx_mlir::ods::sol::ExtICallOperation;
use solx_mlir::ods::sol::ICallOperation;
use solx_mlir::ods::sol::MallocOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Materialize;
use crate::ast::Pointer;
use crate::ast::contract::function::expression::ExpressionContext;

/// The shared call-emission primitives the call kinds dispatch through
/// (argument coercion, call-options capture, indirect calls, struct
/// construction, external-library link resolution).
impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// Evaluates a `{value: …, gas: …, salt: …}` option list in source order
    /// (each value emitted for its side effects) and returns the captured
    /// `value` (as `msg.value`, coerced to `ui256`) and `salt` (the CREATE2 salt
    /// for `new`, cast from `bytes32`). The option KIND comes from slang's typed
    /// `BuiltIn::CallOption*` classification, never from comparing the option
    /// name as text. The `{gas: …}` option is not yet threaded into the
    /// call op and is deferred loudly.
    fn capture_call_options(
        &self,
        call_options: &CallOptionsExpression,
        block: BlockRef<'context, 'block>,
    ) -> (
        Option<Value<'context, 'block>>,
        Option<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        let mut value = None;
        let mut salt = None;
        let mut current_block = block;
        for option in call_options.options().iter() {
            // Emit each option toward the type that option expects, so a literal
            // folds correctly: `value`/`gas` are `ui256`, the CREATE2 `salt` is
            // `bytes32` (a hex/string literal `salt: hex"00"` must fold to a
            // fixedbytes constant, NOT a memory string the salt bridge can't take).
            match option.name().resolve_to_built_in() {
                Some(BuiltIn::CallOptionValue) => {
                    let BlockAnd {
                        value: option_value,
                        block: next_block,
                    } = option.value().emit(self, current_block);
                    current_block = next_block;
                    let builder = &self.state.builder;
                    value = Some(
                        option_value
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionSalt) => {
                    let bytes32 = AstType::fixed_bytes(self.state.builder.context, 32).into_mlir();
                    let salt_expression = option.value();
                    let BlockAnd {
                        value: salt_bytes,
                        block: next_block,
                    } = if let Expression::StringExpression(string_literal) = &salt_expression {
                        string_literal.materialize(bytes32, self, current_block)
                    } else {
                        salt_expression.emit(self, current_block)
                    };
                    current_block = next_block;
                    let builder = &self.state.builder;
                    salt = Some(
                        salt_bytes
                            .cast(
                                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                                builder,
                                &current_block,
                            )
                            .into_mlir(),
                    );
                }
                Some(BuiltIn::CallOptionGas) => {
                    // The gas limit is evaluated for its side effects but not
                    // threaded into the call: the call forwards all remaining gas
                    // (the `sol.ext_icall` default). A `{gas: …}`
                    // that must actually cap the forwarded gas is not yet modelled.
                    let BlockAnd {
                        value: _gas,
                        block: next_block,
                    } = option.value().emit(self, current_block);
                    current_block = next_block;
                }
                _ => unreachable!("a call option resolves to a value, gas, or salt built-in"),
            }
        }
        (value, salt, current_block)
    }

    /// Resolves the callee's MLIR signature and evaluates/coerces its arguments,
    /// already in parameter-declaration order. The expression-keyed core of the
    /// direct-call path, shared by the positional and named-argument forms.
    fn emit_call_setup_expressions<'call>(
        &'call self,
        function_definition: &FunctionDefinition,
        arguments: &[Expression],
        block: BlockRef<'context, 'block>,
    ) -> (
        &'call Function<'context>,
        Vec<Value<'context, 'block>>,
        BlockRef<'context, 'block>,
    ) {
        // Virtual dispatch: a bare internal call resolving (lexically) to an
        // overridden base function is routed to the most-derived override of its
        // signature, so a base-body `g()` reaches the derived `g`. The redirect
        // holds only shadowed-override nodes, so a non-virtual callee passes
        // through unchanged. (`super`/`Base.f` bypass this — they resolve the
        // exact linearised target by id through `super_redirect`.)
        let node_id = function_definition.node_id();
        let call_id = self
            .state
            .virtual_redirect
            .get(&node_id)
            .copied()
            .unwrap_or(node_id);
        let function = self.state.resolve_function(call_id);
        let BlockAnd {
            value: argument_values,
            block: current_block,
        } = arguments.materialize(&function.parameter_types, self, block);
        (function, argument_values, current_block)
    }

    /// Resolves an external library call's link target from its member-access
    /// callee: the library's [`solx_utils::ContractName`], the callee function,
    /// and the `self` receiver (`None` for a namespace-qualified `L.f`, the
    /// operand value for a `using for` `x.f`). Shared by the positional and named
    /// paths.
    fn resolve_external_library(
        &self,
        access: &MemberAccessExpression,
    ) -> (
        solx_utils::ContractName,
        FunctionDefinition,
        Option<Expression>,
    ) {
        let Some(Definition::Function(library_function)) = access.member().resolve_to_definition()
        else {
            unreachable!("an external library call resolves to a function");
        };
        let Some(Definition::Library(library)) = library_function.enclosing_definition() else {
            unreachable!("an external library call's target is a library member");
        };
        let operand = access.operand();
        let self_receiver = (!self.is_namespace_qualifier(&operand)).then_some(operand);
        let name = solx_utils::ContractName::new(
            library.get_file_id().to_owned(),
            Some(library.name().name()),
        );
        (name, library_function, self_receiver)
    }

    /// Emits an indirect call through the function-pointer value `callee`
    /// yields, returning the result values. Parameter and result types come
    /// from the pointer's function type (a void return is zero results; a tuple
    /// return expands per element). Internal pointers dispatch through
    /// `sol.icall`; external ones through `sol.ext_icall`, forwarding
    /// `call_value` (or zero) as `msg.value`.
    fn emit_indirect_call_results(
        &self,
        callee: &Expression,
        function_slang_type: &SlangType,
        positional_arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let BlockAnd {
            value: callee_value,
            block,
        } = callee.emit(self, block);
        let SlangType::Function(function_type) = function_slang_type else {
            unreachable!("an indirect-call callee is always a function type");
        };
        let builder = &self.state.builder;
        let parameter_types: Vec<Type<'context>> = function_type
            .parameter_types()
            .iter()
            .map(|parameter_type| {
                AstType::resolve(parameter_type, LocationPolicy::Declared(None), builder)
            })
            .collect();
        let result_types: Vec<Type<'context>> = match function_type.return_type() {
            SlangType::Void(_) => Vec::new(),
            SlangType::Tuple(tuple_type) => tuple_type
                .types()
                .iter()
                .map(|element_type| {
                    AstType::resolve(element_type, LocationPolicy::Declared(None), builder)
                })
                .collect(),
            other => vec![AstType::resolve(
                &other,
                LocationPolicy::Declared(None),
                builder,
            )],
        };
        let arguments: Vec<Expression> = positional_arguments.iter().collect();
        let BlockAnd {
            value: argument_values,
            block: current_block,
        } = arguments.materialize(&parameter_types, self, block);
        let builder = &self.state.builder;
        // Dispatch internal (`sol.icall`) vs external (`sol.ext_icall`) on the
        // callee value's actual reference kind, not slang's
        // `is_externally_visible`: a bare function name used as a value is an
        // INTERNAL pointer (`func_ref`) even for a `public` function, but slang
        // reports the function type as externally visible — so an inline
        // `(cond ? g : h)(args)` over public functions yields an internal
        // `func_ref` value that an `ext_icall` would mis-cast to `ext_func_ref`.
        let results = if callee_value.r#type().is_ext_function_ref() {
            // `fp{value: v}(args)` forwards `v`; a plain `fp(args)` sends zero.
            let value = call_value.unwrap_or_else(|| {
                AstValue::constant(
                    0,
                    AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                    builder,
                    &current_block,
                )
                .into_mlir()
            });
            // `sol.ext_icall` returns `(i1 status, decoded returns…)`; the status
            // is dropped (a non-`try` call reverts internally on failure). An
            // external function-pointer value carries no `view`/`pure` mutability,
            // so the call is never a STATICCALL.
            let mut out_types = Vec::with_capacity(result_types.len() + 1);
            out_types.push(
                AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir(),
            );
            out_types.extend_from_slice(&result_types);
            let operation = current_block.append_operation(sol_op_build!(
                builder,
                ExtICallOperation
                    .outs(&out_types)
                    .callee(callee_value)
                    .callee_operands(&argument_values)
                    .gas(AstValue::gas_left(builder, &current_block))
                    .value(value)
            ));
            (0..result_types.len())
                .map(|index| {
                    operation
                        .result(index + 1)
                        .expect("sol.ext_icall produces a status plus its declared results")
                        .into()
                })
                .collect()
        } else {
            let operation = current_block.append_operation(sol_op_build!(
                builder,
                ICallOperation
                    .outs(&result_types)
                    .callee(callee_value)
                    .callee_operands(&argument_values)
            ));
            (0..result_types.len())
                .map(|index| {
                    operation
                        .result(index)
                        .expect("sol.icall produces its declared result count")
                        .into()
                })
                .collect()
        };
        (results, current_block)
    }

    /// Whether a member-access operand `x` in `x.f(...)` is a namespace qualifier
    /// — a library or import alias (`L.f` / `M.f`), which is not a value — rather
    /// than a `using for` receiver, which becomes the implicit `self` argument.
    fn is_namespace_qualifier(&self, operand: &Expression) -> bool {
        let Expression::Identifier(identifier) = operand else {
            return false;
        };
        matches!(
            identifier.resolve_to_definition(),
            Some(Definition::Library(_) | Definition::Import(_) | Definition::ImportedSymbol(_))
        )
    }
}

impl<'state, 'context, 'block, 'scope> Emit<'context, 'block, 'state, 'scope>
    for FunctionCallExpression
where
    'context: 'block,
    'context: 'state,
    'block: 'state,
    'state: 'scope,
{
    type Context = &'scope ExpressionContext<'state, 'context, 'block>;
    type Output = (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>);

    /// Emits a function call, yielding its result values in declaration order —
    /// none for a void callee, one for the common case, several for a
    /// tuple-returning call. The callee, resolved through slang's binder, selects
    /// the shape directly: a single match over the callee expression and its
    /// resolved definition, no intermediate kind enum.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        // `recv.f{value: v}(args)` / `new C{value, salt}(args)`: evaluate the
        // option list (each for its side effects, in source order) before the
        // arguments, forwarding `value` as msg.value and `salt` as the CREATE2
        // salt. The inner callee drives the dispatch below.
        let (call_value, salt, block, callee) = match self.operand().unwrap_parentheses() {
            Expression::CallOptionsExpression(options) => {
                let (value, salt, block) = context.capture_call_options(&options, block);
                (value, salt, block, options.operand().unwrap_parentheses())
            }
            other => (None, None, block, other),
        };
        let arguments = self.arguments();

        // A callee resolving to a struct definition is a struct constructor —
        // `S(a, b)` / `S({…})` / `Lib.S(...)`, in any argument spelling: allocate
        // the struct in memory, order the field initialisers by member
        // declaration, and store each coerced to its field type.
        let struct_callee = match &callee {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(access) => access.member().resolve_to_definition(),
            _ => None,
        };
        if let Some(Definition::Struct(struct_definition)) = struct_callee {
            let result_type = AstType::resolve_optional(self.get_type(), &context.state.builder)
                .expect("slang validated");
            let member_ids: Vec<NodeId> = struct_definition
                .members()
                .iter()
                .map(|member| member.node_id())
                .collect();
            let arguments = arguments.ordered_by(&member_ids);
            let builder = &context.state.builder;
            let struct_address = sol_op!(builder, &block, MallocOperation.addr(result_type));
            let struct_pointer = Pointer::new(struct_address);
            let mut block = block;
            for (index, (member, argument)) in struct_definition
                .members()
                .iter()
                .zip(arguments.iter())
                .enumerate()
            {
                let field_slang_type = member.get_type().expect("slang validated");
                let field_type = AstType::resolve(
                    &field_slang_type,
                    LocationPolicy::Declared(Some(DataLocation::Memory)),
                    builder,
                );
                let index_value = AstValue::constant(
                    index as i64,
                    AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_X64),
                    builder,
                    &block,
                );
                let field_address =
                    struct_pointer.gep(index_value, AstType::new(field_type), builder, &block);
                let BlockAnd {
                    value: argument_value,
                    block: next_block,
                } = argument.emit(context, block);
                block = next_block;
                let stored = argument_value.cast(AstType::new(field_type), builder, &block);
                field_address.store(stored, builder, &block);
            }
            return (vec![struct_address], block);
        }

        // `T(x)` / `bytesN("…")`: an explicit 1-argument type conversion coerces
        // the argument to the call's own type.
        if self.is_type_conversion()
            && let ArgumentsDeclaration::PositionalArguments(positional) = &arguments
            && positional.len() == 1
        {
            let first = positional.iter().next().expect("slang validated");
            let target_type = AstType::resolve_optional(self.get_type(), &context.state.builder)
                .expect("slang validated");
            let BlockAnd { value, block } = first.materialize(target_type, context, block);
            return (vec![value.into_mlir()], block);
        }

        // An identifier-callee built-in (`keccak256`, `require`, …).
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
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("a built-in takes positional arguments only");
            };
            let (value, block) = context.emit_built_in_call(built_in, positional, block);
            return (value.into_iter().collect(), block);
        }

        // A member-access callee: a call-position built-in, a namespace-qualified
        // struct constructor, or a member call `x.f(...)`.
        if let Expression::MemberAccessExpression(access) = &callee {
            match access.member().resolve_to_built_in() {
                // `addr.call/delegatecall/staticcall(data)` → (success, returndata).
                Some(
                    kind @ (BuiltIn::AddressCall
                    | BuiltIn::AddressDelegatecall
                    | BuiltIn::AddressStaticcall),
                ) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("a bare low-level call takes positional arguments only");
                    };
                    let (status, ret_data, block) =
                        context.emit_bare_call(access, kind, positional, call_value, block);
                    return (vec![status, ret_data], block);
                }
                // `abi.decode(payload, (T))` — result type from the call.
                Some(BuiltIn::AbiDecode) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("abi.decode takes positional arguments only");
                    };
                    return context.emit_abi_decode(self, positional, block);
                }
                // `T.wrap(x)` / `T.unwrap(x)`: a single conversion to the result type.
                Some(BuiltIn::Wrap | BuiltIn::Unwrap) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("a UDVT wrap/unwrap takes one positional argument");
                    };
                    let argument = positional.iter().next().expect("slang validated");
                    let BlockAnd { value, block } = argument.emit(context, block);
                    let result =
                        match AstType::resolve_optional(self.get_type(), &context.state.builder) {
                            Some(result_type) => value
                                .cast(AstType::new(result_type), &context.state.builder, &block)
                                .into_mlir(),
                            None => value.into_mlir(),
                        };
                    return (vec![result], block);
                }
                // Any other member built-in in call position (`abi.encode`, `arr.push`, …).
                Some(_) => {
                    let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                        unimplemented!("a built-in member takes positional arguments only");
                    };
                    let (value, block) =
                        context.emit_built_in_member_access(access, Some(positional), block);
                    return (value.into_iter().collect(), block);
                }
                None => {}
            }

            // A member call `x.f(...)`, classified by operand and member resolution.
            let operand = access.operand();
            // `super.f` / a recorded base redirect: an internal call up the C3 chain.
            if matches!(operand, Expression::SuperKeyword(_))
                || context.state.super_redirect.contains_key(&access.node_id())
            {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unimplemented!("named arguments on a super call are not supported");
                };
                let target_id = context
                    .state
                    .super_redirect
                    .get(&access.node_id())
                    .copied()
                    .expect("a super/base call has a recorded redirect target");
                let argument_expressions: Vec<Expression> = positional.iter().collect();
                let function = context.state.resolve_function(target_id);
                let BlockAnd {
                    value: argument_values,
                    block,
                } = argument_expressions.materialize(&function.parameter_types, context, block);
                let results = function.call(&argument_values, &context.state.builder, &block);
                return (results, block);
            }

            let member_definition = access.member().resolve_to_definition();
            // An external library call (`L.f` namespace or `using for` onto a
            // selector-bearing library function) delegatecalls — the only member
            // call that accepts named arguments.
            if let Some(Definition::Function(function)) = &member_definition
                && function.compute_selector().is_some()
                && (matches!(&operand, Expression::Identifier(identifier)
                        if matches!(identifier.resolve_to_definition(), Some(Definition::Library(_))))
                    || matches!(
                        function.enclosing_definition(),
                        Some(Definition::Library(_))
                    ))
            {
                let (library_name, library_function, self_receiver) =
                    context.resolve_external_library(access);
                let parameter_ids: Vec<NodeId> = library_function
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let explicit_parameter_ids = if self_receiver.is_some() {
                    &parameter_ids[1..]
                } else {
                    &parameter_ids[..]
                };
                let argument_expressions = arguments.ordered_by(explicit_parameter_ids);
                return context.emit_library_external_call(
                    &library_name,
                    &library_function,
                    &argument_expressions,
                    self_receiver.as_ref(),
                    block,
                );
            }

            // Every other member call is positional.
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("named arguments on this member call are not supported");
            };
            return match member_definition {
                // `using for` / `L.f` onto an internal (no-selector) library fn,
                // inlined like an ordinary internal call; a selector-bearing one is
                // a `this.f` / `instance.f` external call.
                Some(Definition::Function(function)) if function.compute_selector().is_none() => {
                    let resolved = context.state.resolve_function(function.node_id());
                    // A namespace qualifier (`L.f` / `M.f`) is not a value, so only
                    // the explicit arguments pass; a `using for` receiver becomes the
                    // implicit `self` first parameter.
                    if context.is_namespace_qualifier(&operand) {
                        let arguments: Vec<Expression> = positional.iter().collect();
                        let BlockAnd {
                            value: argument_values,
                            block,
                        } = arguments.materialize(&resolved.parameter_types, context, block);
                        let results =
                            resolved.call(&argument_values, &context.state.builder, &block);
                        (results, block)
                    } else {
                        let (parameter_self, parameter_rest) = resolved
                            .parameter_types
                            .split_first()
                            .expect("slang validated");
                        let BlockAnd {
                            value: self_value,
                            block,
                        } = operand.emit(context, block);
                        let self_value = self_value
                            .cast(
                                AstType::new(*parameter_self),
                                &context.state.builder,
                                &block,
                            )
                            .into_mlir();
                        let arguments: Vec<Expression> = positional.iter().collect();
                        let BlockAnd {
                            value: mut argument_values,
                            block,
                        } = arguments.materialize(parameter_rest, context, block);
                        argument_values.insert(0, self_value);
                        let results =
                            resolved.call(&argument_values, &context.state.builder, &block);
                        (results, block)
                    }
                }
                Some(Definition::Function(_)) => {
                    // `this.f` / `instance.f`: an external call.
                    context.emit_external(access, call_value, positional, block)
                }
                // `C.x(args)`: a function-pointer state variable read then called;
                // `this.x` / `instance.x`: a getter (an external call).
                Some(Definition::StateVariable(_)) => {
                    if matches!(&operand, Expression::Identifier(identifier)
                        if matches!(identifier.resolve_to_definition(), Some(Definition::Contract(_))))
                        && matches!(access.get_type(), Some(SlangType::Function(_)))
                    {
                        let callee = Expression::MemberAccessExpression(access.clone());
                        let function_slang_type = access.get_type().expect("slang validated");
                        context.emit_indirect_call_results(
                            &callee,
                            &function_slang_type,
                            positional,
                            call_value,
                            block,
                        )
                    } else {
                        context.emit_external(access, call_value, positional, block)
                    }
                }
                // `s.f(...)` through a function-pointer struct field.
                Some(Definition::StructMember(_))
                    if matches!(access.get_type(), Some(SlangType::Function(_))) =>
                {
                    let callee = Expression::MemberAccessExpression(access.clone());
                    let function_slang_type = access.get_type().expect("slang validated");
                    context.emit_indirect_call_results(
                        &callee,
                        &function_slang_type,
                        positional,
                        call_value,
                        block,
                    )
                }
                other => unimplemented!(
                    "unsupported member call: {:?}",
                    other.map(|definition| definition.node_id())
                ),
            };
        }

        // `new T[](n)` / `new bytes(n)` / `new C(args)`.
        if let Expression::NewExpression(_) = &callee {
            let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                unimplemented!("named arguments on a new expression are not supported");
            };
            let (value, block) = context.emit_new(self, positional, call_value, salt, block);
            return (value.into_iter().collect(), block);
        }

        let Expression::Identifier(identifier) = &callee else {
            // `T[](x)`: an empty-bracket array type used as a data-location cast.
            if let Expression::IndexAccessExpression(array_type) = &callee
                && array_type.start().is_none()
                && array_type.end().is_none()
                && !matches!(array_type.kind(), IndexAccessKind::Slice)
            {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unimplemented!("named arguments on an array-type cast are not supported");
                };
                let first = positional.iter().next().expect("slang validated");
                let target_type =
                    AstType::resolve_optional(self.get_type(), &context.state.builder)
                        .expect("slang validated");
                let BlockAnd { value, block } = first.materialize(target_type, context, block);
                return (vec![value.into_mlir()], block);
            }
            // `arr[i](args)` / `(cond ? f : g)(args)`: a call through a pointer value.
            if matches!(callee.get_type(), Some(SlangType::Function(_))) {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unimplemented!("named arguments on an indirect call are not supported");
                };
                let function_slang_type = callee.get_type().expect("slang validated");
                return context.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    positional,
                    call_value,
                    block,
                );
            }
            unimplemented!("unsupported callee expression");
        };
        match identifier.resolve_to_definition() {
            // A direct call passes its arguments by position or by name; ordering
            // them against the parameter ids collapses both into one path.
            Some(Definition::Function(function_definition)) => {
                let parameter_ids: Vec<NodeId> = function_definition
                    .parameters()
                    .iter()
                    .map(|parameter| parameter.node_id())
                    .collect();
                let ordered = arguments.ordered_by(&parameter_ids);
                let (function, argument_values, block) =
                    context.emit_call_setup_expressions(&function_definition, &ordered, block);
                let results = function.call(&argument_values, &context.state.builder, &block);
                (results, block)
            }
            // A function-typed variable / parameter / state variable calls through
            // its stored pointer.
            Some(
                Definition::Variable(_) | Definition::Parameter(_) | Definition::StateVariable(_),
            ) if matches!(identifier.get_type(), Some(SlangType::Function(_))) => {
                let ArgumentsDeclaration::PositionalArguments(positional) = &arguments else {
                    unimplemented!("named arguments on an indirect call are not supported");
                };
                let function_slang_type = callee.get_type().expect("slang validated");
                context.emit_indirect_call_results(
                    &callee,
                    &function_slang_type,
                    positional,
                    call_value,
                    block,
                )
            }
            _ => unimplemented!(
                "callee '{}' does not resolve to a function",
                identifier.name()
            ),
        }
    }
}
