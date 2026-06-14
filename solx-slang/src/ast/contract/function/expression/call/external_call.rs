//!
//! External / bare-address call lowering.
//!

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
use solx_utils::DataLocation;

use crate::ast::BlockAnd;
use crate::ast::Emit;
use crate::ast::contract::ContractEmitter;
use crate::ast::contract::function::expression::ExpressionContext;
use crate::ast::contract::function::expression::call::static_mode::StaticMode;
use crate::ast::type_conversion::LocationPolicy;
use crate::ast::type_conversion::TypeConversion;

impl<'state, 'context, 'block> ExpressionContext<'state, 'context, 'block> {
    /// The SOLE `ext_icall` sink for `SelfExternal` + `ExternalInstance`.
    ///
    /// The call's inputs are passed flat — a single bundle struct would be a
    /// forbidden second top-level type under §2a — with `static_call` enum-ized
    /// into [`StaticMode`] (R8-4). At 9 args (`&self` + 8) this is the one
    /// signature above the `clippy.toml` `too-many-arguments-threshold`; it is a
    /// deliberate WARN at the skeleton tip, never an `#[allow]` (Rule 11).
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
    ) -> anyhow::Result<Vec<Value<'context, 'block>>> {
        let callee =
            self.emit_external_callee(receiver, selector, parameter_types, return_types, block);
        let builder = &self.state.builder;
        // The call value defaults to zero wei.
        let value = call_value.unwrap_or_else(|| {
            builder.emit_sol_constant(
                0,
                crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir(),
                block,
            )
        });
        builder.emit_sol_ext_icall(
            callee,
            argument_values,
            return_types,
            value,
            matches!(static_mode, StaticMode::Static),
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
        let address = crate::ast::Value::from(receiver)
            .cast(
                crate::ast::Type::address(builder.context, false).into_mlir(),
                builder,
                block,
            )
            .into_mlir();
        let ext_func_ref_type =
            crate::ast::Type::ext_func_ref(builder.context, parameter_types, return_types)
                .into_mlir();
        builder.emit_sol_ext_func_constant(address, selector, ext_func_ref_type, block)
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
                    vec![TypeConversion::resolve_slang_type(
                        &key,
                        LocationPolicy::Declared(None),
                        builder,
                    )],
                    vec![TypeConversion::resolve_slang_type(
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
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![TypeConversion::resolve_slang_type(
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
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ],
                    vec![TypeConversion::resolve_slang_type(
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
                let struct_mlir_type = TypeConversion::resolve_slang_type(
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
                vec![TypeConversion::resolve_slang_type(
                    other,
                    LocationPolicy::Declared(None),
                    builder,
                )],
            )),
            _ => None,
        }
    }

    /// Emits a self getter call (`this.x()` / `this.m(key)`); A4 (#H-M7):
    /// nested / reference-typed getters are a LOUD residual via
    /// [`Self::getter_signature`].
    pub fn emit_self_getter_call(
        &self,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        call_value: Option<Value<'context, 'block>>,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let selector = state_variable
            .compute_selector()
            .expect("a public state variable has a getter selector");
        let Some((parameter_types, return_types)) = self.getter_signature(state_variable) else {
            unimplemented!(
                "self getter of a nested or reference-typed state variable is not yet supported"
            );
        };
        let (argument_values, current_block) =
            self.emit_coerced_arguments(arguments, &parameter_types, block)?;
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(self, current_block)?;
        let results = self.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &argument_values,
            call_value,
            StaticMode::Call,
            &current_block,
        )?;
        Ok((results, current_block))
    }

    /// Emits an external getter call (`instance.value()` scalar); A4
    /// (#H-M10/M11): arg-bearing mapping/array getters are a LOUD residual.
    pub fn emit_external_getter_call(
        &self,
        access: &MemberAccessExpression,
        state_variable: &StateVariableDefinition,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        // The external accessor lowered here is single-valued, so an arg-bearing
        // mapping / array getter on another instance is a LOUD residual
        // (#H-M10/M11); only the no-argument scalar / struct getter lowers here.
        if !arguments.is_empty() {
            unimplemented!("external getter with key/index arguments is not yet supported");
        }
        let selector = state_variable
            .compute_selector()
            .expect("a public state variable has a getter selector");
        let Some((parameter_types, return_types)) = self.getter_signature(state_variable) else {
            unimplemented!(
                "external getter of a nested or reference-typed state variable is not yet supported"
            );
        };
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(self, block)?;
        let results = self.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &[],
            None,
            StaticMode::Call,
            &current_block,
        )?;
        Ok((results.into_iter().next(), current_block))
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
    ) -> anyhow::Result<(
        Value<'context, 'block>,
        Value<'context, 'block>,
        BlockRef<'context, 'block>,
    )> {
        let BlockAnd {
            value: address,
            block,
        } = access.operand().emit(self, block)?;
        let argument = arguments
            .iter()
            .next()
            .expect("a bare call takes one bytes argument");
        let BlockAnd {
            value: input,
            block,
        } = argument.emit(self, block)?;

        let builder = &self.state.builder;
        // `sol.bare_call`'s input rejects a non-memory operand, so an argument
        // sourced from storage / calldata is copied into memory first.
        let input = input
            .coerce_to(
                crate::ast::Type::string(builder.context, solx_utils::DataLocation::Memory)
                    .into_mlir(),
                builder,
                &block,
            )
            .into_mlir();
        let address = address.into_mlir();
        let (status, ret_data) = match kind {
            BuiltIn::AddressCall => {
                let value = call_value.unwrap_or_else(|| {
                    builder.emit_sol_constant(
                        0,
                        crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                        &block,
                    )
                });
                builder.emit_sol_bare_call(address, value, input, &block)
            }
            BuiltIn::AddressDelegatecall => {
                builder.emit_sol_bare_delegate_call(address, input, &block)
            }
            BuiltIn::AddressStaticcall => builder.emit_sol_bare_static_call(address, input, &block),
            _ => unreachable!("bare call kind must be Call, Delegatecall, or Staticcall"),
        };
        Ok((status, ret_data, block))
    }

    /// Emits a bare address call in result-binding position
    /// (`(ok, data) = addr.call{..}(data)`).
    pub fn emit_bare_call_results(
        &self,
        access: &MemberAccessExpression,
        kind: BuiltIn,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (status, ret_data, block) =
            self.emit_bare_call(access, kind, arguments, call_value, block)?;
        Ok((vec![status, ret_data], block))
    }

    /// Emits a multi-result external call (`(a, b) = recv.f(args)`); always a
    /// `sol.ext_icall`.
    pub fn emit_external_call_results(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        call_value: Option<Value<'context, 'block>>,
        arguments: &PositionalArguments,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Vec<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let selector = function_definition
            .compute_selector()
            .expect("an external call resolves to a function with a selector");
        // The signature comes from the callee's definition (valid for both a
        // foreign instance and the current contract's own function), so the
        // unified path never depends on the callee being in the local registry.
        // External calls cross the ABI boundary, so a `calldata` reference
        // parameter is encoded from / decoded to memory — the callee type and
        // argument coercions use the EXTERNAL (memory) representation.
        let (parameter_types, return_types) = TypeConversion::resolve_function_types(
            function_definition,
            LocationPolicy::ForceMemory,
            &self.state.builder,
        );
        // The receiver is the member operand: `this` for a self call, the
        // instance value for an external one — both evaluate to an address.
        let BlockAnd {
            value: receiver,
            block: current_block,
        } = access.operand().emit(self, block)?;
        let (argument_values, current_block) =
            self.emit_coerced_arguments(arguments, &parameter_types, current_block)?;
        let results = self.emit_external_call(
            receiver.into_mlir(),
            selector,
            &parameter_types,
            &return_types,
            &argument_values,
            call_value,
            StaticMode::from_function(function_definition),
            &current_block,
        )?;
        Ok((results, current_block))
    }
}
