//!
//! Function call and member access expression emission.
//!

use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
pub mod built_in;
pub mod call_kind;
pub mod external_call;
pub mod library_call;
pub mod library_visibility;
pub mod member_call_kind;
pub mod positional_arguments;
pub mod static_mode;
pub mod try_external_call;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::CallOptionsExpression;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionCallExpression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::Function;
use solx_mlir::ods::sol::ExtICallOperation;
use solx_mlir::ods::sol::ICallOperation;

use self::call_kind::CallKind;
use self::member_call_kind::MemberCallKind;
use self::static_mode::StaticMode;
use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Materialize;
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
        let self_receiver = (!MemberCallKind::is_namespace_qualifier(&operand)).then_some(operand);
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
            self.emit_ext_icall(
                callee_value.into_mlir(),
                &argument_values,
                &result_types,
                value,
                StaticMode::Call,
                &current_block,
            )
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

    /// Emits a `sol.ext_icall` through the external-function-pointer `callee`,
    /// forwarding all remaining gas and `value` as msg.value, and returns the
    /// decoded results. The `i1` status is the first result and is dropped — a
    /// non-`try` call reverts internally on failure. `static_mode` selects a
    /// STATICCALL for a `view`/`pure` callee (which reverts on a state change,
    /// matching solc). The shared `ext_icall` sink for the direct external call
    /// (`emit_external_call`) and the external-function-pointer call above.
    fn emit_ext_icall(
        &self,
        callee: Value<'context, 'block>,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        value: Value<'context, 'block>,
        static_mode: StaticMode,
        block: &BlockRef<'context, 'block>,
    ) -> Vec<Value<'context, 'block>> {
        let builder = &self.state.builder;
        // `sol.ext_icall` results are `(i1 status, decoded-returns...)`; the status
        // is prepended here and dropped from the values handed back.
        let mut out_types = Vec::with_capacity(result_types.len() + 1);
        out_types
            .push(AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
        out_types.extend_from_slice(result_types);
        let mut operation_builder =
            ExtICallOperation::builder(builder.context, builder.unknown_location)
                .outs(&out_types)
                .callee(callee)
                .callee_operands(operands)
                .gas(AstValue::gas_left(builder, block).into_mlir())
                .value(value);
        if matches!(static_mode, StaticMode::Static) {
            operation_builder = operation_builder.static_call(Attribute::unit(builder.context));
        }
        let operation = block.append_operation(operation_builder.build().into());
        (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_icall produces a status plus its declared results")
                    .into()
            })
            .collect()
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
    /// tuple-returning call (`(a, b) = f(...)`). In value position the sole result
    /// is taken through [`Expression`]'s emit; a statement-position discard keeps
    /// only the continuation block.
    fn emit(&self, context: Self::Context, block: BlockRef<'context, 'block>) -> Self::Output {
        CallKind::new(context, self).emit(context, self, block)
    }
}
