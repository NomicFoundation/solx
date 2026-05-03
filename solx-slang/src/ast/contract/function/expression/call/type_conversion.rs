//!
//! Solidity type conversion classification and dispatch.
//!

use melior::ir::Type;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use slang_solidity::backend::ir::ast::ContractMember;
use slang_solidity::backend::ir::ast::Definition;
use slang_solidity::backend::ir::ast::FunctionDefinition;
use slang_solidity::backend::ir::ast::FunctionKind;
use slang_solidity::backend::ir::ast::FunctionMutability;
use slang_solidity::backend::ir::ast::LiteralKind;
use slang_solidity::backend::ir::ast::Parameter;
use slang_solidity::backend::ir::ast::StateVariableDefinition;
use slang_solidity::backend::ir::ast::Type as SlangType;

/// Classification of Solidity type conversions.
///
/// Used for both explicit conversions (`uint256(x)`, `address(x)`, `bool(x)`)
/// and implicit operand widening in arithmetic, assignment, and comparison.
pub enum TypeConversion<'context> {
    /// `bool(x)` — comparison against zero, not bit-truncation.
    Bool,
    /// `address(x)` / `payable(x)` — `sol.cast` to ui160 then `sol.address_cast`.
    Address,
    /// Integer type cast — `sol.cast` to target.
    Cast(Type<'context>),
}

impl<'context> TypeConversion<'context> {
    /// Maps a Slang semantic type to an MLIR type.
    ///
    /// `inherited_location` is the dialect data location to substitute when a
    /// type's Slang location is `Inherited` (struct-field-relative). Top-level
    /// callers pass `None`; the `Struct` arm sets it to the parent struct's
    /// location for the duration of member resolution.
    pub fn resolve_slang_type(
        slang_type: &SlangType,
        inherited_location: Option<solx_utils::DataLocation>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Type<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.signed() {
                    Type::from(IntegerType::signed(builder.context, bits))
                } else {
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
            }
            SlangType::Boolean(_) => Type::from(IntegerType::new(
                builder.context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => builder.types.sol_address,
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Zero => Type::from(IntegerType::unsigned(
                    builder.context,
                    solx_utils::BIT_LENGTH_BYTE as u32,
                )),
                LiteralKind::Address => builder.types.sol_address,
                LiteralKind::DecimalInteger {
                    bytes,
                    signed: true,
                } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    Type::from(IntegerType::signed(builder.context, bits))
                }
                LiteralKind::DecimalInteger {
                    bytes,
                    signed: false,
                }
                | LiteralKind::HexInteger { bytes } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
                kind @ (LiteralKind::Rational
                | LiteralKind::HexString { .. }
                | LiteralKind::String { .. }) => {
                    unimplemented!(
                        "MLIR type resolution is not yet implemented for literal kind {kind:?}"
                    )
                }
            },
            SlangType::String(string_type) => {
                let location = solx_utils::DataLocation::from_slang(string_type.location(), inherited_location);
                builder.types.string(location)
            }
            SlangType::Bytes(bytes_type) => {
                let location = solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
                builder.types.string(location)
            }
            SlangType::ByteArray(byte_array_type) => {
                builder.types.fixed_bytes(byte_array_type.width())
            }
            SlangType::Array(array_type) => {
                let element_type = Self::resolve_slang_type(
                    &array_type.element_type(),
                    inherited_location,
                    builder,
                );
                let location = solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                builder.types.array(solx_mlir::ArraySize::Dynamic, element_type, location)
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = Self::resolve_slang_type(
                    &fixed_array_type.element_type(),
                    inherited_location,
                    builder,
                );
                let location = solx_utils::DataLocation::from_slang(fixed_array_type.location(), inherited_location);
                builder.types.array(
                    solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Self::resolve_slang_type(
                    &mapping_type.key_type(),
                    inherited_location,
                    builder,
                );
                let value_type = Self::resolve_slang_type(
                    &mapping_type.value_type(),
                    inherited_location,
                    builder,
                );
                builder.types.mapping(key_type, value_type)
            }
            SlangType::Struct(struct_type) => {
                let struct_location = solx_utils::DataLocation::from_slang(struct_type.location(), inherited_location);
                let struct_definition = match struct_type.definition() {
                    Definition::Struct(definition) => definition,
                    _ => unreachable!(
                        "Slang StructType always references a Struct definition"
                    ),
                };
                let mut member_types = Vec::new();
                for member in struct_definition.members().iter() {
                    let member_slang_type = member
                        .get_type()
                        .expect("struct member type resolved by semantic analysis");
                    member_types.push(Self::resolve_slang_type(
                        &member_slang_type,
                        Some(struct_location),
                        builder,
                    ));
                }
                builder.types.structure(&member_types, struct_location)
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!(
                        "Slang ContractType always references a Contract definition"
                    ),
                };
                let payable = contract_definition.members().iter().any(|member| {
                    let ContractMember::FunctionDefinition(function) = member else {
                        return false;
                    };
                    match function.kind() {
                        FunctionKind::Receive => true,
                        FunctionKind::Fallback => {
                            matches!(function.mutability(), FunctionMutability::Payable)
                        }
                        _ => false,
                    }
                });
                builder
                    .types
                    .contract(contract_definition.name().name().as_str(), payable)
            }
            SlangType::Interface(interface_type) => {
                let interface_definition = match interface_type.definition() {
                    Definition::Interface(definition) => definition,
                    _ => unreachable!(
                        "Slang InterfaceType always references an Interface definition"
                    ),
                };
                // Interfaces are never `payable` themselves; payability lives
                // on the address-cast at the call site.
                builder
                    .types
                    .contract(interface_definition.name().name().as_str(), false)
            }
            SlangType::Enum(enum_type) => {
                let enum_definition = match enum_type.definition() {
                    Definition::Enum(definition) => definition,
                    _ => unreachable!(
                        "Slang EnumType always references an Enum definition"
                    ),
                };
                let member_count = enum_definition.members().iter().count();
                let max = u32::try_from(member_count - 1).expect("enum member count fits in u32");
                builder.types.enumeration(max)
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Classifies a target type into the appropriate conversion variant.
    pub fn from_target_type(
        target_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Self {
        if target_type == builder.types.i1 {
            Self::Bool
        } else if target_type == builder.types.sol_address {
            Self::Address
        } else {
            Self::Cast(target_type)
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let name = state_variable.name().name();
        let slang_type = state_variable
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("unresolved type for state variable: {name}"))?;
        Ok(Self::resolve_slang_type(&slang_type, None, builder))
    }

    /// Resolves a function's parameter and return types from Slang AST to MLIR.
    pub fn resolve_function_types(
        function: &FunctionDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let resolve = |parameter: Parameter| {
            Self::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                builder,
            )
        };
        let parameter_types = function.parameters().iter().map(&resolve).collect();
        let return_types = function
            .returns()
            .map(|returns| returns.iter().map(&resolve).collect())
            .unwrap_or_default();
        (parameter_types, return_types)
    }

    /// Emits the conversion, returning the cast value.
    pub fn emit<'block>(
        self,
        value: melior::ir::Value<'context, 'block>,
        builder: &solx_mlir::Builder<'context>,
        block: &melior::ir::BlockRef<'context, 'block>,
    ) -> melior::ir::Value<'context, 'block>
    where
        'context: 'block,
    {
        match self {
            Self::Bool => {
                let zero = builder.emit_sol_constant(0, value.r#type(), block);
                builder.emit_sol_cmp(value, zero, solx_mlir::CmpPredicate::Ne, block)
            }
            Self::Address => {
                let address_type = builder.types.sol_address;
                let truncated = if melior::ir::r#type::IntegerType::try_from(value.r#type()).is_ok()
                {
                    let ui160 = builder.types.ui160;
                    builder.emit_sol_cast(value, ui160, block)
                } else {
                    value
                };
                builder.emit_sol_address_cast(truncated, address_type, block)
            }
            Self::Cast(target_type) => builder.emit_sol_cast(value, target_type, block),
        }
    }
}
