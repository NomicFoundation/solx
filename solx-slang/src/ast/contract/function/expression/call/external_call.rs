//!
//! External / bare-address call emission.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use slang_solidity_v2::ast::BuiltIn;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::BareCallOperation;
use solx_mlir::ods::sol::BareDelegateCallOperation;
use solx_mlir::ods::sol::BareStaticCallOperation;
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::LocationPolicy;
use crate::ast::Type as AstType;
use crate::ast::Value as AstValue;
use crate::ast::contract::ContractEmitter;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::member_call_kind::MemberCallKind;
use crate::ast::contract::function::expression::call::static_mode::StaticMode;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// The `ext_icall` sink for `SelfExternal` + `ExternalInstance`.
    ///
    /// Inputs are passed flat rather than as a bundle struct, with `static_call`
    /// carried as [`StaticMode`]. At 9 arguments this is the one signature above
    /// the `too-many-arguments` clippy threshold — a known WARN, not suppressed.
    pub fn emit_external_call(
        &self,
        receiver: Value<'context, 'block>,
        selector: u32,
        parameter_types: &[Type<'context>],
        return_types: &[Type<'context>],
        argument_values: &[Value<'context, 'block>],
        call_value: Option<Value<'context, 'block>>,
        static_mode: StaticMode,
        block: &BlockRef<'context, 'block>,
    ) -> Vec<Value<'context, 'block>> {
        let callee =
            self.emit_external_callee(receiver, selector, parameter_types, return_types, block);
        let builder = &self.state.builder;
        // The call value defaults to zero wei.
        let value = call_value.unwrap_or_else(|| {
            AstValue::constant(
                0,
                AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                block,
            )
            .into_mlir()
        });
        self.emit_ext_icall(
            callee,
            argument_values,
            return_types,
            value,
            static_mode,
            block,
        )
    }

    /// Packs a receiver address and `selector` into the `!sol.ext_func_ref`
    /// callee an external interaction carries, via `sol.address_cast` +
    /// `sol.ext_func_constant`. The single builder of that representation,
    /// shared by `CALL`/`STATICCALL`, the `try`-call, and a `this.f` /
    /// `instance.f` external function-pointer value.
    pub fn emit_external_callee(
        &self,
        receiver: Value<'context, 'block>,
        selector: u32,
        parameter_types: &[Type<'context>],
        return_types: &[Type<'context>],
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.state.builder;
        let address =
            AstValue::from(receiver).cast(AstType::address(builder.context, false), builder, block);
        let ext_func_ref_type =
            AstType::ext_func_ref(builder.context, parameter_types, return_types);
        AstValue::ext_func_constant(address, selector, ext_func_ref_type, builder, block)
            .into_mlir()
    }

    /// The ABI signature of a `public` state variable's synthesised getter:
    /// the key/index parameter types and the returned value types.
    ///
    /// A scalar variable `T public x` is `() -> (T)`; a mapping `mapping(K => V)`
    /// is `(K) -> (V)`; an array is `(uint256) -> (element)`; a struct is
    /// `() -> (flattened returnable members)` (sharing the synthesised getter's
    /// member layout, so the call decodes exactly what the getter returns).
    /// Single-level only — a nested or reference-typed key / value / element
    /// returns `None`, a LOUD residual at the (already-classified) call site.
    fn getter_signature(
        &self,
        state_variable: &StateVariableDefinition,
    ) -> Option<(Vec<Type<'context>>, Vec<Type<'context>>)> {
        let declared_type = state_variable.get_type()?;
        let builder = &self.state.builder;
        match &declared_type {
            SlangType::Mapping(mapping_type) => {
                let key = mapping_type.key_type();
                let value = mapping_type.value_type();
                if key.is_reference_type() || value.is_reference_type() {
                    return None;
                }
                Some((
                    vec![AstType::resolve(
                        &key,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                    vec![AstType::resolve(
                        &value,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::Array(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![
                        AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![AstType::resolve(
                        &element,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::FixedSizeArray(array_type) => {
                let element = array_type.element_type();
                if element.is_reference_type() {
                    return None;
                }
                Some((
                    vec![
                        AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![AstType::resolve(
                        &element,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                ))
            }
            SlangType::Struct(struct_type) => {
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    return None;
                };
                let struct_mlir_type = AstType::resolve(
                    &declared_type,
                    LocationPolicy::Declared(Some(DataLocation::Storage)),
                    builder,
                );
                let plan = ContractEmitter::struct_getter_layout(
                    &struct_definition,
                    struct_mlir_type,
                    builder,
                )?;
                let return_types = plan
                    .iter()
                    .map(|(_, _, result_type)| *result_type)
                    .collect();
                Some((Vec::new(), return_types))
            }
            other if !other.is_reference_type() => Some((
                Vec::new(),
                vec![AstType::resolve(
                    other,
                    LocationPolicy::Declared(None),
                    builder,
                )],
            )),
            _ => None,
        }
    }

    /// Emits a bare address call (`addr.call`/`delegatecall`/`staticcall`),
    /// returning `(success, returndata-pointer, block)`. Inner
    /// `_ => unreachable!("bare call kind must be Call/Delegatecall/Staticcall")`.
    pub fn emit_bare_call(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    ) {
        let BlockAnd {
            value: address,
            block,
        } = access.operand().emit(self, block);
        let argument = arguments
            .iter()
            .next()
            .expect("slang validated");
        let BlockAnd {
            value: input,
            block,
        } = argument.emit(self, block);

        let builder = &self.state.builder;
        // `sol.bare_call`'s input rejects a non-memory operand, so an argument
        // sourced from storage / calldata is copied into memory first.
        let input = input
            .cast(
                AstType::string(builder.context, solx_utils::DataLocation::Memory),
                builder,
                &block,
            )
            .into_mlir();
        let address = address.into_mlir();
        let status_type =
            AstType::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir();
        let ret_data_type =
            AstType::string(builder.context, solx_utils::DataLocation::Memory).into_mlir();
        let operation = match kind {
            BuiltIn::AddressCall => {
                let value = call_value.unwrap_or_else(|| {
                    AstValue::constant(
                        0,
                        AstType::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                        builder,
                        &block,
                    )
                    .into_mlir()
                });
                sol_op_build!(
                    builder,
                    BareCallOperation
                        .addr(address)
                        .gas(AstValue::gas_left(builder, &block))
                        .val(value)
                        .inp(input)
                        .status(status_type)
                        .ret_data(ret_data_type)
                )
            }
            BuiltIn::AddressDelegatecall => sol_op_build!(
                builder,
                BareDelegateCallOperation
                    .addr(address)
                    .gas(AstValue::gas_left(builder, &block))
                    .inp(input)
                    .status(status_type)
                    .ret_data(ret_data_type)
            ),
            BuiltIn::AddressStaticcall => sol_op_build!(
                builder,
                BareStaticCallOperation
                    .addr(address)
                    .gas(AstValue::gas_left(builder, &block))
                    .inp(input)
                    .status(status_type)
                    .ret_data(ret_data_type)
            ),
            _ => unreachable!("bare call kind must be Call, Delegatecall, or Staticcall"),
        };
        let operation = block.append_operation(operation);
        let status = operation
            .result(0)
            .expect("a bare call always produces a status")
            .into();
        let ret_data = operation
            .result(1)
            .expect("a bare call always produces return data")
            .into();
        (status, ret_data, block)
    }
}

impl MemberCallKind {
    /// Emits a multi-result external call (`(a, b) = recv.f(args)`); always a
    /// `sol.ext_icall`.
    pub fn emit_external_call_results<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let selector = function_definition
            .compute_selector()
            .expect("slang validated");
        // The signature comes from the callee's definition (valid for both a
        // foreign instance and the current contract's own function), so the
        // unified path never depends on the callee being in the local registry.
        // External calls cross the ABI boundary, so a `calldata` reference
        // parameter is encoded from / decoded to memory — the callee type and
        // argument coercions use the EXTERNAL (memory) representation.
        let (parameter_types, return_types) = AstType::resolve_signature(
            function_definition,
            LocationPolicy::ForceMemory,
            &context.state.builder,
        );
        // The receiver is the member operand: `this` for a self call, the
        // instance value for an external one — both evaluate to an address.
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(context, block);
        let (argument_values, current_block) =
            context.emit_coerced_arguments(arguments, &parameter_types, current_block);
        let results = context.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &argument_values,
            call_value,
            StaticMode::from_function(function_definition),
            &current_block,
        );
        (results, current_block)
    }

    /// Emits a self getter call (`this.x()` / `this.m(key)`); A4 (#H-M7):
    /// nested / reference-typed getters are a LOUD residual.
    pub fn emit_self_getter_call<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> (Vec<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        let selector = state_variable
            .compute_selector()
            .expect("slang validated");
        let Some((parameter_types, return_types)) = context.getter_signature(state_variable) else {
            unimplemented!(
                "self getter of a nested or reference-typed state variable is not yet supported"
            );
        };
        let (argument_values, current_block) =
            context.emit_coerced_arguments(arguments, &parameter_types, block);
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(context, current_block);
        let results = context.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &argument_values,
            call_value,
            StaticMode::Call,
            &current_block,
        );
        (results, current_block)
    }

    /// Emits an external getter call (`instance.value()` scalar); A4
    /// (#H-M10/M11): arg-bearing mapping/array getters are a LOUD residual.
    pub fn emit_external_getter_call<'state, 'context, 'block>(
        &self,
        context: &ExpressionContext<'state, 'context, 'block>,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> (Option<Value<'context, 'block>>, BlockRef<'context, 'block>) {
        // The external accessor lowered here is single-valued, so an arg-bearing
        // mapping / array getter on another instance is a LOUD residual
        // (#H-M10/M11); only the no-argument scalar / struct getter lowers here.
        if !arguments.is_empty() {
            unimplemented!("external getter with key/index arguments is not yet supported");
        }
        let selector = state_variable
            .compute_selector()
            .expect("slang validated");
        let Some((parameter_types, return_types)) = context.getter_signature(state_variable) else {
            unimplemented!(
                "external getter of a nested or reference-typed state variable is not yet supported"
            );
        };
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(context, block);
        let results = context.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &[],
            None,
            StaticMode::Call,
            &current_block,
        );
        (results.into_iter().next(), current_block)
    }
}
