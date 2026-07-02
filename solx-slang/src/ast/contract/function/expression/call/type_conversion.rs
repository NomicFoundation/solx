//!
//! Solidity type conversion classification and dispatch.
//!

use melior::ir::Type;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_traits::sign::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ArraySize;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Type as AstType;
use solx_mlir::Value as AstValue;

use crate::ast::contract::ContractEmitter;

/// Classification of Solidity type conversions.
///
/// Used for both explicit conversions (`uint256(x)`, `address(x)`, `bool(x)`)
/// and implicit operand widening in arithmetic, assignment, and comparison.
pub enum TypeConversion<'context> {
    /// `bool(x)` — comparison against zero, not bit-truncation.
    Bool,
    /// `address(x)` / `payable(x)` — `sol.cast` to ui160 then `sol.address_cast`.
    Address,
    /// `bytesN(x)` / `byte(x)` and the reverse `uintN(bytesN)` — `sol.bytes_cast` to target.
    BytesCast(Type<'context>),
    /// `bytesN(b)` on a dynamic `bytes` value — `sol.dyn_bytes_to_fixedbytes` to target.
    DynBytesToFixedBytes(Type<'context>),
    /// `bytes(b)` / `string(s)` relocating a reference across data locations — `sol.data_loc_cast`.
    DataLocCast(Type<'context>),
    /// `E(x)` and the reverse `uint8(E)` — `sol.enum_cast` between an enum and its backing integer.
    EnumCast(Type<'context>),
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
        context: &Context<'context>,
    ) -> Type<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.is_signed() {
                    Type::from(IntegerType::signed(context.mlir_context, bits))
                } else {
                    Type::from(IntegerType::unsigned(context.mlir_context, bits))
                }
            }
            SlangType::FixedPointNumber(fixed_point_type) => {
                let bits = fixed_point_type.bits();
                if fixed_point_type.is_signed() {
                    Type::from(IntegerType::signed(context.mlir_context, bits))
                } else {
                    Type::from(IntegerType::unsigned(context.mlir_context, bits))
                }
            }
            SlangType::Boolean(_) => Type::from(IntegerType::new(
                context.mlir_context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => AstType::address(context.mlir_context, false).into_mlir(),
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => {
                    AstType::address(context.mlir_context, false).into_mlir()
                }
                LiteralKind::Integer { value } => {
                    let bits = Self::integer_bits_required(&value) as usize;
                    let bits = bits
                        .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                        .max(solx_utils::BIT_LENGTH_BYTE);
                    let bits = u32::try_from(bits).expect("bit size fits in 32 bits");
                    if value.is_negative() {
                        Type::from(IntegerType::signed(context.mlir_context, bits))
                    } else {
                        Type::from(IntegerType::unsigned(context.mlir_context, bits))
                    }
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    Type::from(IntegerType::unsigned(context.mlir_context, bits))
                }
                LiteralKind::String { .. } => {
                    AstType::string(context.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir()
                }
                LiteralKind::HexString { bytes } => AstType::fixed_bytes(
                    context.mlir_context,
                    bytes.try_into().expect("hex string length fits in u32"),
                )
                .into_mlir(),
                LiteralKind::Rational { .. } => unimplemented!(
                    "MLIR type resolution is not yet implemented for rational literals"
                ),
            },
            SlangType::String(string_type) => {
                let location = solx_utils::DataLocation::from_slang(
                    string_type.location(),
                    inherited_location,
                );
                AstType::string(context.mlir_context, location).into_mlir()
            }
            SlangType::Bytes(bytes_type) => {
                let location =
                    solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
                AstType::string(context.mlir_context, location).into_mlir()
            }
            SlangType::ByteArray(byte_array_type) => {
                AstType::fixed_bytes(context.mlir_context, byte_array_type.width()).into_mlir()
            }
            SlangType::Array(array_type) => {
                let element_type = Self::resolve_slang_type(
                    &array_type.element_type(),
                    inherited_location,
                    context,
                );
                let location =
                    solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                AstType::array(
                    context.mlir_context,
                    ArraySize::Dynamic,
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = Self::resolve_slang_type(
                    &fixed_array_type.element_type(),
                    inherited_location,
                    context,
                );
                let location = solx_utils::DataLocation::from_slang(
                    fixed_array_type.location(),
                    inherited_location,
                );
                AstType::array(
                    context.mlir_context,
                    ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Self::resolve_slang_type(
                    &mapping_type.key_type(),
                    Some(solx_utils::DataLocation::Storage),
                    context,
                );
                let value_type = Self::resolve_slang_type(
                    &mapping_type.value_type(),
                    Some(solx_utils::DataLocation::Storage),
                    context,
                );
                AstType::mapping(context.mlir_context, key_type, value_type).into_mlir()
            }
            SlangType::Struct(struct_type) => {
                let struct_location = solx_utils::DataLocation::from_slang(
                    struct_type.location(),
                    inherited_location,
                );
                let struct_definition = match struct_type.definition() {
                    Definition::Struct(definition) => definition,
                    _ => unreachable!("Slang StructType always references a Struct definition"),
                };
                let mut member_types = Vec::new();
                for member in struct_definition.members().iter() {
                    let member_slang_type = member
                        .get_type()
                        .expect("struct member type resolved by semantic analysis");
                    member_types.push(Self::resolve_slang_type(
                        &member_slang_type,
                        Some(struct_location),
                        context,
                    ));
                }
                AstType::structure(context.mlir_context, &member_types, struct_location).into_mlir()
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                AstType::contract(
                    context.mlir_context,
                    contract_definition.name().name().as_str(),
                    ContractEmitter::is_contract_payable(&contract_definition),
                )
                .into_mlir()
            }
            SlangType::Interface(interface_type) => {
                let interface_definition = match interface_type.definition() {
                    Definition::Interface(definition) => definition,
                    _ => unreachable!(
                        "Slang InterfaceType always references an Interface definition"
                    ),
                };
                AstType::contract(
                    context.mlir_context,
                    interface_definition.name().name().as_str(),
                    false,
                )
                .into_mlir()
            }
            SlangType::Enum(enum_type) => {
                let enum_definition = match enum_type.definition() {
                    Definition::Enum(definition) => definition,
                    _ => unreachable!("Slang EnumType always references an Enum definition"),
                };
                let member_count = enum_definition.members().iter().count();
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                AstType::enumeration(context.mlir_context, max.into()).into_mlir()
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::resolve_slang_type(&target_type, inherited_location, context)
            }
            SlangType::Function(function_type) => {
                if function_type.is_externally_visible() {
                    unimplemented!("external function-pointer types are not yet supported");
                }
                let (parameter_types, result_types) =
                    Self::function_pointer_signature(slang_type, context);
                AstType::func_ref(context.mlir_context, &parameter_types, &result_types).into_mlir()
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Resolves a function-pointer callee type's `(parameter_types, result_types)` from Slang to MLIR:
    /// the declared parameter types, and the flattened result types where `void` is empty and a tuple
    /// return expands per element.
    pub fn function_pointer_signature(
        callee_type: &SlangType,
        context: &Context<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let SlangType::Function(function_type) = callee_type else {
            unreachable!("an indirect-call callee is always a function type");
        };
        let parameter_types = function_type
            .parameter_types()
            .iter()
            .map(|parameter_type| Self::resolve_slang_type(parameter_type, None, context))
            .collect();
        let result_types = match function_type.return_type() {
            SlangType::Void(_) => Vec::new(),
            SlangType::Tuple(tuple_type) => tuple_type
                .types()
                .iter()
                .map(|element_type| Self::resolve_slang_type(element_type, None, context))
                .collect(),
            other => vec![Self::resolve_slang_type(&other, None, context)],
        };
        (parameter_types, result_types)
    }

    fn integer_bits_required(value: &BigInt) -> u32 {
        if value.is_negative() {
            let magnitude_minus_one = -value - 1u32;
            u32::try_from(magnitude_minus_one.bits()).unwrap() + 1
        } else {
            u32::try_from(value.bits()).unwrap().max(1)
        }
    }

    /// Classifies a target type into the appropriate conversion variant.
    ///
    /// A fixed-width byte target routes through `sol.bytes_cast`, so an integer literal coerced to a
    /// `bytesN` return or argument narrows through the byte representation rather than the illegal
    /// integer-to-fixed-bytes `sol.cast`. A dynamic-bytes target routes through `sol.data_loc_cast`,
    /// relocating a `bytes` / `string` value across data locations.
    pub fn from_target_type(target_type: Type<'context>, context: &Context<'context>) -> Self {
        if target_type == AstType::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir()
        {
            Self::Bool
        } else if target_type == AstType::address(context.mlir_context, false).into_mlir() {
            Self::Address
        } else if AstType::new(target_type)
            .fixed_bytes_or_byte_width(context.mlir_context)
            .is_some()
        {
            Self::BytesCast(target_type)
        } else if AstType::new(target_type).is_string(context.mlir_context) {
            Self::DataLocCast(target_type)
        } else {
            Self::Cast(target_type)
        }
    }

    /// Classifies an explicit `T(x)` conversion from its Slang source and target types.
    ///
    /// An enumeration on either side routes through `sol.enum_cast`. A dynamic `bytes` / `string`
    /// source toward a `bytesN` target routes through `sol.dyn_bytes_to_fixedbytes`. A fixed-width
    /// byte type on either side routes through `sol.bytes_cast`; the Slang types disambiguate the
    /// reverse `uintN(bytesN)` and `uint8(E)` directions that the MLIR target alone cannot. Any other
    /// conversion, including a `bytes` / `string` data-location relocation, defers to the target-typed
    /// classification.
    pub fn from_slang_conversion(
        source: &SlangType,
        target: &SlangType,
        target_type: Type<'context>,
        context: &Context<'context>,
    ) -> Self {
        let source_is_reference = matches!(
            source,
            SlangType::Bytes(_) | SlangType::String(_) | SlangType::Array(_)
        );
        if matches!(source, SlangType::Enum(_)) || matches!(target, SlangType::Enum(_)) {
            Self::EnumCast(target_type)
        } else if source_is_reference && matches!(target, SlangType::ByteArray(_)) {
            Self::DynBytesToFixedBytes(target_type)
        } else if matches!(source, SlangType::ByteArray(_))
            || matches!(target, SlangType::ByteArray(_))
        {
            Self::BytesCast(target_type)
        } else {
            Self::from_target_type(target_type, context)
        }
    }

    /// Returns the MLIR target type this conversion produces.
    pub fn to_target_type(&self, context: &Context<'context>) -> Type<'context> {
        match self {
            Self::Bool => {
                AstType::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir()
            }
            Self::Address => AstType::address(context.mlir_context, false).into_mlir(),
            Self::BytesCast(target_type)
            | Self::DynBytesToFixedBytes(target_type)
            | Self::DataLocCast(target_type)
            | Self::EnumCast(target_type)
            | Self::Cast(target_type) => *target_type,
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        context: &Context<'context>,
    ) -> Type<'context> {
        let slang_type = state_variable
            .get_type()
            .expect("binder types every state variable");
        Self::resolve_slang_type(&slang_type, None, context)
    }

    /// Resolves a function's parameter and return types from Slang AST to MLIR.
    pub fn resolve_function_types(
        function: &FunctionDefinition,
        context: &Context<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let resolve = |parameter: Parameter| {
            Self::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                None,
                context,
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
        context: &Context<'context>,
        block: &melior::ir::BlockRef<'context, 'block>,
    ) -> melior::ir::Value<'context, 'block>
    where
        'context: 'block,
    {
        if value.r#type() == self.to_target_type(context) {
            return value;
        }
        match self {
            Self::Bool => {
                let zero = AstValue::constant(
                    0,
                    AstType::new(value.r#type()),
                    context,
                    block,
                );
                AstValue::new(value)
                    .compare(zero, CmpPredicate::Ne, context, block)
                    .into_mlir()
            }
            Self::Address => {
                let address_type = AstType::address(context.mlir_context, false);
                let truncated = if IntegerType::try_from(value.r#type()).is_ok() {
                    let ui160 =
                        AstType::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS);
                    AstValue::new(value).cast(ui160, context, block)
                } else {
                    AstValue::new(value)
                };
                truncated.address_cast(address_type, context, block).into_mlir()
            }
            Self::BytesCast(target_type) => {
                let bridged = match (
                    IntegerType::try_from(value.r#type()),
                    AstType::new(target_type).fixed_bytes_or_byte_width(context.mlir_context),
                ) {
                    (Ok(integer), Some(width)) => {
                        let bridge_bits = width * solx_utils::BIT_LENGTH_BYTE as u32;
                        if integer.width() == bridge_bits {
                            AstValue::new(value)
                        } else {
                            let bridge =
                                AstType::unsigned(context.mlir_context, bridge_bits as usize);
                            AstValue::new(value).cast(bridge, context, block)
                        }
                    }
                    _ => AstValue::new(value),
                };
                bridged
                    .bytes_cast(AstType::new(target_type), context, block)
                    .into_mlir()
            }
            Self::DynBytesToFixedBytes(target_type) => AstValue::new(value)
                .dyn_bytes_to_fixedbytes(AstType::new(target_type), context, block)
                .into_mlir(),
            Self::DataLocCast(target_type) => AstValue::new(value)
                .data_loc_cast(AstType::new(target_type), context, block)
                .into_mlir(),
            Self::EnumCast(target_type) => AstValue::new(value)
                .enum_cast(AstType::new(target_type), context, block)
                .into_mlir(),
            Self::Cast(target_type) => AstValue::new(value)
                .cast(AstType::new(target_type), context, block)
                .into_mlir(),
        }
    }
}
