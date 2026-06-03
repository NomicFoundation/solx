//!
//! `type(T)` introspection built-ins: `type(T).min` / `.max`, the enum forms
//! `type(E).min` / `.max`, `type(I).interfaceId`, `type(C).creationCode` /
//! `.runtimeCode`, and `type(C).name`.
//!

use super::*;

impl<'emitter, 'state, 'context, 'block> CallEmitter<'emitter, 'state, 'context, 'block> {
    /// `type(E).min` / `type(E).max` for an enum `E` are the first / last
    /// enumerator (ordinals `0` and `member_count - 1`), as an enum value.
    pub(crate) fn try_emit_type_enum_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(builtin @ (BuiltIn::TypeEnumMin | BuiltIn::TypeEnumMax)) =
            access.member().resolve_to_built_in()
            && let Expression::TypeExpression(type_expression) = access.operand()
            && let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            && let Some(Definition::Enum(enum_definition)) =
                identifier_path.resolve_to_definition()
            && let Some(result_type) = self
                .expression_emitter
                .resolve_slang_type(access.get_type())
        {
            let member_count = enum_definition.members().iter().count();
            let ordinal = match builtin {
                BuiltIn::TypeEnumMin => 0,
                BuiltIn::TypeEnumMax => member_count.saturating_sub(1) as i64,
                _ => unreachable!("matched TypeEnumMin/TypeEnumMax above"),
            };
            let builder = &self.expression_emitter.state.builder;
            let int_value = builder.emit_sol_constant(ordinal, builder.types.ui256, &block);
            let enum_value = builder.emit_sol_enum_cast(int_value, result_type, &block);
            return Ok(Some((Some(enum_value), block)));
        }
        Ok(None)
    }

    /// `type(T).min` / `type(T).max` are compile-time integer constants.
    pub(crate) fn try_emit_type_min_max(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(builtin @ (BuiltIn::TypeMin | BuiltIn::TypeMax)) =
            access.member().resolve_to_built_in()
            && let Some(result_type) = self
                .expression_emitter
                .resolve_slang_type(access.get_type())
            && let Ok(integer_type) = melior::ir::r#type::IntegerType::try_from(result_type)
        {
            let bits = solx_mlir::TypeFactory::integer_bit_width(result_type);
            let signed = integer_type.is_signed();
            let value = match (builtin, signed) {
                (BuiltIn::TypeMin, false) => num_bigint::BigInt::ZERO,
                (BuiltIn::TypeMin, true) => {
                    -(num_bigint::BigInt::from(1) << (bits as usize - 1))
                }
                (BuiltIn::TypeMax, false) => {
                    (num_bigint::BigInt::from(1) << bits as usize) - 1
                }
                (BuiltIn::TypeMax, true) => {
                    (num_bigint::BigInt::from(1) << (bits as usize - 1)) - 1
                }
                _ => unreachable!("matched TypeMin/TypeMax above"),
            };
            let value =
                self.expression_emitter
                    .state
                    .builder
                    .emit_constant(&value, result_type, &block);
            return Ok(Some((Some(value), block)));
        }
        Ok(None)
    }

    /// `type(I).interfaceId` is a compile-time `bytes4` constant: the
    /// EIP-165 interface identifier, defined as the XOR of the selectors of
    /// the functions declared *directly* within the interface `I`. Inherited
    /// functions are deliberately excluded (matching solc), so we iterate
    /// the interface's own members rather than its linearised functions.
    pub(crate) fn try_emit_type_interface_id(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(BuiltIn::TypeInterfaceId) = access.member().resolve_to_built_in()
            && let Expression::TypeExpression(type_expression) = access.operand()
            && let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            && let Some(Definition::Interface(interface_definition)) =
                identifier_path.resolve_to_definition()
        {
            let mut interface_id: u32 = 0;
            let members = interface_definition.members();
            for member in members.iter() {
                if let slang_solidity_v2::ast::ContractMember::FunctionDefinition(function) =
                    member
                    && let Some(selector) = function.compute_selector()
                {
                    interface_id ^= selector;
                }
            }
            // `!sol.fixedbytes<4>` rejects a bare integer attribute, so emit the
            // identifier as a `uint32` constant and bridge to `bytes4` via
            // `sol.bytes_cast` (same pattern as `f.selector`).
            let builder = &self.expression_emitter.state.builder;
            let integer_type = Type::from(IntegerType::unsigned(builder.context, 32));
            let integer = builder.emit_constant(
                &num_bigint::BigInt::from(interface_id),
                integer_type,
                &block,
            );
            let value =
                builder.emit_sol_bytes_cast(integer, builder.types.fixed_bytes(4), &block);
            return Ok(Some((Some(value), block)));
        }
        Ok(None)
    }

    /// `type(C).creationCode` / `type(C).runtimeCode` yield the contract's
    /// deploy / deployed bytecode as `bytes memory`, lowered to
    /// `sol.object_code` referencing the object by name (`C` for creation,
    /// `C_deployed` for runtime). The reference is registered as a linker
    /// dependency so the assembler pulls the object in (as `new C()` does).
    pub(crate) fn try_emit_type_code(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(builtin @ (BuiltIn::TypeCreationCode | BuiltIn::TypeRuntimeCode)) =
            access.member().resolve_to_built_in()
            && let Expression::TypeExpression(type_expression) = access.operand()
            && let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            && let Some(Definition::Contract(contract_definition)) =
                identifier_path.resolve_to_definition()
        {
            let contract_name = contract_definition.name().name();
            let object_name = match builtin {
                BuiltIn::TypeRuntimeCode => {
                    format!("{contract_name}{}", solx_codegen_evm::DEPLOYED_OBJECT_SUFFIX)
                }
                _ => contract_name,
            };
            // Depend on the *object* actually referenced — `C` for creation
            // code, `C_deployed` for runtime code. The deployed object is a
            // distinct top-level linker object; depending on `C` alone leaves
            // `runtimeCode`'s `__datasize__`/`__dataoffset__` symbols
            // unresolved.
            self.expression_emitter
                .state
                .add_dependency(object_name.clone());
            let result_type = self
                .expression_emitter
                .resolve_slang_type(access.get_type())
                .unwrap_or_else(|| {
                    self.expression_emitter
                        .state
                        .builder
                        .types
                        .string(solx_utils::DataLocation::Memory)
                });
            let builder = &self.expression_emitter.state.builder;
            let value = block
                .append_operation(
                    ObjectCodeOperation::builder(builder.context, builder.unknown_location)
                        .obj_name(StringAttribute::new(builder.context, &object_name))
                        .out(result_type)
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.object_code always produces one result")
                .into();
            return Ok(Some((Some(value), block)));
        }
        Ok(None)
    }

    /// `type(C).name` yields the contract/interface name as a `string memory`
    /// constant.
    pub(crate) fn try_emit_type_name(
        &self,
        access: &MemberAccessExpression,
        block: BlockRef<'context, 'block>,
    ) -> anyhow::Result<Option<(Option<Value<'context, 'block>>, BlockRef<'context, 'block>)>> {
        if let Some(BuiltIn::TypeName) = access.member().resolve_to_built_in()
            && let Expression::TypeExpression(type_expression) = access.operand()
            && let SlangTypeName::IdentifierPath(identifier_path) = type_expression.type_name()
            && let Some(type_name) = match identifier_path.resolve_to_definition() {
                Some(Definition::Contract(contract)) => Some(contract.name().name()),
                Some(Definition::Interface(interface)) => Some(interface.name().name()),
                _ => None,
            }
        {
            let value = self
                .expression_emitter
                .state
                .builder
                .emit_sol_string_lit(&type_name, &block);
            return Ok(Some((Some(value), block)));
        }
        Ok(None)
    }
}
