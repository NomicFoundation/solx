//!
//! Member references that resolve to a value rather than a call or EVM
//! intrinsic — currently enum variants (`E.Variant`).
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_bigint::Sign;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::Expression;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::MemberAccessExpression;
use slang_solidity_v2::ast::PositionalArguments;

use crate::ast::contract::function::expression::call::CallEmitter;
use crate::ast::type_conversion::TypeConversion;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// Classifies a member access as an enum-variant reference (`E.Variant` or
    /// qualified `C.E.Variant`), returning the variant's ordinal when it is one
    /// (and not a call). The ordinal is located by NodeId identity against the
    /// enum's members, never by comparing the member name as text (Rule-7).
    pub fn enum_variant_ordinal(
        &self,
        access: &MemberAccessExpression,
        arguments: Option<&PositionalArguments>,
    ) -> Option<usize> {
        if arguments.is_some() {
            return None;
        }
        let Definition::EnumMember(member_definition) = access.member().resolve_to_definition()?
        else {
            return None;
        };
        let Definition::Enum(enum_definition) =
            Self::resolve_member_access_operand(&access.operand())?
        else {
            return None;
        };
        enum_definition
            .members()
            .iter()
            .position(|member| member.node_id() == member_definition.node_id())
    }

    /// Emits an enum-variant reference: the variant's `ordinal` as an integer
    /// constant, bridged to the enum type via `sol.enum_cast`.
    pub fn emit_enum_variant(
        &self,
        access: &MemberAccessExpression,
        ordinal: usize,
        block: BlockRef<'context, 'block>,
    ) -> (Value<'context, 'block>, BlockRef<'context, 'block>) {
        let result_type = self
            .expression_emitter
            .resolve_slang_type(access.get_type())
            .expect("slang types an enum-variant reference as the enum");
        let builder = &self.expression_emitter.state.builder;
        let raw = builder.emit_sol_constant(ordinal as i64, builder.types.ui256, &block);
        let value = builder.emit_sol_enum_cast(raw, result_type, &block);
        (value, block)
    }

    /// Resolves a member-access operand to its definition: a bare type name
    /// (`E.Variant`, whose operand is the `Identifier` `E`) or a qualified path
    /// whose operand is itself a member access (`C.E.Variant`).
    fn resolve_member_access_operand(operand: &Expression) -> Option<Definition> {
        match operand {
            Expression::Identifier(identifier) => identifier.resolve_to_definition(),
            Expression::MemberAccessExpression(member_access) => {
                member_access.member().resolve_to_definition()
            }
            _ => None,
        }
    }

    /// `f.selector` — the 4-byte selector (`bytes4`) of a function. A statically
    /// named function (`this.f`, `i.foo`) or public-getter member folds to a
    /// compile-time constant via its `compute_selector()`; an external
    /// function-pointer VALUE (`(cond ? a : b).selector`, a `function (...)
    /// external` local) pulls its selector at runtime via `sol.ext_func_selector`.
    /// slang already classifies the member as `BuiltIn::FunctionSelector`, so the
    /// member is never recognised by comparing its name as text (Rule-7).
    pub fn emit_function_selector(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let static_selector = match Self::resolve_member_access_operand(&access.operand()) {
            Some(Definition::Function(function)) => function.compute_selector(),
            Some(Definition::StateVariable(state_variable)) => state_variable.compute_selector(),
            _ => None,
        };
        if let Some(selector) = static_selector {
            let block = self.eval_selector_receiver_side_effects(access, block)?;
            let value = self.emit_selector_constant(&BigInt::from(selector), 4, &block);
            return Ok((Some(value), block));
        }
        let (operand_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        assert!(
            solx_mlir::TypeFactory::is_sol_ext_function_ref(operand_value.r#type()),
            "function `.selector` resolves to a named function, a public getter, or an external function-pointer value"
        );
        let selector = self
            .expression_emitter
            .state
            .builder
            .emit_sol_ext_func_selector(operand_value, &block);
        Ok((Some(selector), block))
    }

    /// `f.address` — the address component of an external function-pointer VALUE,
    /// pulled out of its `!sol.ext_func_ref` at runtime via `sol.ext_func_addr`.
    /// slang classifies the member as `BuiltIn::FunctionAddress` (never by name
    /// text, Rule-7).
    pub fn emit_function_address(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let (operand_value, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        assert!(
            solx_mlir::TypeFactory::is_sol_ext_function_ref(operand_value.r#type()),
            "function `.address` requires an external function-pointer value"
        );
        let address = self
            .expression_emitter
            .state
            .builder
            .emit_sol_ext_func_addr(operand_value, &block);
        Ok((Some(address), block))
    }

    /// `this.f` / `instance.f` used as a value (not called) is an external
    /// function pointer: the receiver address packed with the function's
    /// selector into a `!sol.ext_func_ref` via `sol.ext_func_constant` (the same
    /// representation an external call builds for its callee).
    pub fn emit_external_function_pointer(
        &self,
        access: &MemberAccessExpression,
        function_definition: &FunctionDefinition,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let selector = function_definition
            .compute_selector()
            .expect("an external function pointer resolves to a function with a selector");
        // An external function pointer's ABI representation (address + selector)
        // types its reference parameters as `Memory`, not their declared
        // `calldata`/`storage` location — calldata cannot cross the call
        // boundary. solc emits the `ext_func_constant` at this memory signature,
        // so assigning `this.g` (declared `string calldata`) to a
        // `function (string memory) external` pointer needs no cast: both are the
        // same `ext_func_ref<(string<Memory>) -> …>`.
        let (parameter_types, return_types) = TypeConversion::resolve_external_function_types(
            function_definition,
            &self.expression_emitter.state.builder,
        );
        let (receiver, block) = self
            .expression_emitter
            .emit_value(&access.operand(), block)?;
        let builder = &self.expression_emitter.state.builder;
        let address = builder.emit_sol_address_cast(receiver, builder.types.sol_address, &block);
        let ext_ref_type = builder.types.ext_func_ref(&parameter_types, &return_types);
        let value = builder.emit_sol_ext_func_constant(address, selector, ext_ref_type, &block);
        Ok((Some(value), block))
    }

    /// `MyError.selector` — the error's 4-byte selector (`bytes4`) as a
    /// compile-time constant.
    pub fn emit_error_selector(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Some(Definition::Error(error)) = Self::resolve_member_access_operand(&access.operand())
        else {
            unreachable!("slang resolves an error `.selector` base to an error definition");
        };
        let selector = error
            .compute_selector()
            .expect("slang computes a 4-byte selector for an error");
        let block = self.eval_selector_receiver_side_effects(access, block)?;
        let value = self.emit_selector_constant(&BigInt::from(selector), 4, &block);
        Ok((Some(value), block))
    }

    /// `MyEvent.selector` — the event's 32-byte topic hash (`bytes32`), the
    /// keccak256 of its canonical signature, as a compile-time constant.
    pub fn emit_event_selector(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)> {
        let Some(Definition::Event(event)) = Self::resolve_member_access_operand(&access.operand())
        else {
            unreachable!("slang resolves an event `.selector` base to an event definition");
        };
        let signature = event
            .compute_canonical_signature()
            .expect("slang computes a canonical signature for a non-anonymous event");
        let hash = solx_utils::Keccak256Hash::from_slice(signature.as_bytes());
        let topic = BigInt::from_bytes_be(Sign::Plus, hash.as_bytes());
        let block = self.eval_selector_receiver_side_effects(access, block)?;
        let value = self.emit_selector_constant(&topic, 32, &block);
        Ok((Some(value), block))
    }

    /// Emits a compile-time selector value of `width_bytes`: an unsigned integer
    /// constant of the matching width bridged to `!sol.fixedbytes<width_bytes>`
    /// via `sol.bytes_cast` (the fixed-bytes type rejects a bare integer
    /// attribute).
    fn emit_selector_constant(
        &self,
        value: &BigInt,
        width_bytes: u32,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let builder = &self.expression_emitter.state.builder;
        let integer_type = Type::from(IntegerType::unsigned(
            builder.context,
            width_bytes * solx_utils::BIT_LENGTH_BYTE as u32,
        ));
        let integer = builder.emit_constant(value, integer_type, block);
        builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(width_bytes), block)
    }

    /// Evaluates the receiver of a `<receiver>.member.selector` for its side
    /// effects when `<receiver>` is a runtime value (e.g. the call in
    /// `h().f.selector`). A namespace / type qualifier (`C.f.selector`) has no
    /// runtime value, so nothing is evaluated. The selector itself stays a
    /// compile-time constant; this only reproduces the discarded receiver's
    /// evaluation.
    fn eval_selector_receiver_side_effects(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<BlockRef<'context, 'block>> {
        let Expression::MemberAccessExpression(inner) = access.operand() else {
            return Ok(block);
        };
        let receiver = inner.operand();
        if Self::is_namespace_or_type_operand(&receiver) {
            return Ok(block);
        }
        let (_discarded, block) = self.expression_emitter.emit_value(&receiver, block)?;
        Ok(block)
    }

    /// Whether `expression` is a namespace or type reference (a contract /
    /// interface / library / import / enum / struct / user-defined-value-type
    /// name) rather than a runtime value — such an operand carries no side
    /// effects, so a `.selector` taken through it evaluates nothing.
    fn is_namespace_or_type_operand(expression: &Expression) -> bool {
        matches!(
            Self::resolve_member_access_operand(expression),
            Some(
                Definition::Contract(_)
                    | Definition::Interface(_)
                    | Definition::Library(_)
                    | Definition::Import(_)
                    | Definition::ImportedSymbol(_)
                    | Definition::Enum(_)
                    | Definition::Struct(_)
                    | Definition::UserDefinedValueType(_)
            )
        )
    }
}
