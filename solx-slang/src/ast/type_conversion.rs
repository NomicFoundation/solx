//!
//! Solidity type conversion classification and dispatch.
//!

use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
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
                LiteralKind::Address { .. } => builder.types.sol_address,
                LiteralKind::Integer { value } => {
                    let bits = Self::integer_bits_required(&value) as usize;
                    let bits = bits
                        .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                        .max(solx_utils::BIT_LENGTH_BYTE);
                    let bits = u32::try_from(bits).expect("bit size fits in 32 bits");
                    if value.is_negative() {
                        Type::from(IntegerType::signed(builder.context, bits))
                    } else {
                        Type::from(IntegerType::unsigned(builder.context, bits))
                    }
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
                LiteralKind::String { .. } => {
                    builder.types.string(solx_utils::DataLocation::Memory)
                }
                LiteralKind::HexString { bytes } => builder
                    .types
                    .fixed_bytes(bytes.try_into().expect("hex string length fits in u32")),
                LiteralKind::Rational { .. } => unimplemented!(
                    "MLIR type resolution is not yet implemented for rational literals"
                ),
            },
            SlangType::String(string_type) => {
                let location = solx_utils::DataLocation::from_slang(
                    string_type.location(),
                    inherited_location,
                );
                builder.types.string(location)
            }
            SlangType::Bytes(bytes_type) => {
                let location =
                    solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
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
                let location =
                    solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                builder
                    .types
                    .array(solx_mlir::ArraySize::Dynamic, element_type, location)
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = Self::resolve_slang_type(
                    &fixed_array_type.element_type(),
                    inherited_location,
                    builder,
                );
                let location = solx_utils::DataLocation::from_slang(
                    fixed_array_type.location(),
                    inherited_location,
                );
                builder.types.array(
                    solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Self::resolve_slang_type(
                    &mapping_type.key_type(),
                    Some(solx_utils::DataLocation::Storage),
                    builder,
                );
                let value_type = Self::resolve_slang_type(
                    &mapping_type.value_type(),
                    Some(solx_utils::DataLocation::Storage),
                    builder,
                );
                builder.types.mapping(key_type, value_type)
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
                // TODO(v2): move struct-member type resolution into Slang itself
                // so consumers don't have to walk `members()` and propagate the
                // struct's data location by hand.
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
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                builder.types.contract(
                    contract_definition.name().name().as_str(),
                    ContractEmitter::is_contract_payable(&contract_definition),
                )
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
                    _ => unreachable!("Slang EnumType always references an Enum definition"),
                };
                let member_count = enum_definition.members().iter().count();
                // Solidity caps enums at 256 members, so the max enumerator
                // index always fits in a `u8`.
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                builder.types.enumeration(max.into())
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::resolve_slang_type(&target_type, inherited_location, builder)
            }
            SlangType::Function(function_type) => {
                // A function pointer lowers to `!sol.func_ref<fnTy>` (internal)
                // or `!sol.ext_func_ref<fnTy>` (external — address + selector).
                // A void return contributes zero result types; a tuple return
                // expands to one result per element.
                let parameter_types: Vec<_> = function_type
                    .parameter_types()
                    .iter()
                    .map(|parameter_type| Self::resolve_slang_type(parameter_type, None, builder))
                    .collect();
                let result_types: Vec<_> = match function_type.return_type() {
                    SlangType::Void(_) => Vec::new(),
                    SlangType::Tuple(tuple_type) => tuple_type
                        .types()
                        .iter()
                        .map(|element_type| Self::resolve_slang_type(element_type, None, builder))
                        .collect(),
                    other => vec![Self::resolve_slang_type(&other, None, builder)],
                };
                if function_type.is_externally_visible() {
                    builder.types.ext_func_ref(&parameter_types, &result_types)
                } else {
                    builder.types.func_ref(&parameter_types, &result_types)
                }
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Emits the zero value of a scalar value type that is not a plain
    /// integer/bool: `address(0)`, a zero `bytesN`, or an enum's `0` variant
    /// (a UDVT defers to its underlying type). The zero constant is materialised
    /// at the representation's own width and bridged with that type's dedicated
    /// cast — never by narrowing a wider constant, which the `sol.cast` fold
    /// mishandles. Plain integers/bools are zeroed directly by
    /// `Builder::emit_zero_initialized_alloca` and do not reach here.
    pub fn emit_scalar_zero<'block>(
        slang_type: &SlangType,
        mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match slang_type {
            SlangType::Integer(_) | SlangType::Boolean(_) => {
                builder.emit_sol_constant(0, mlir_type, block)
            }
            SlangType::Address(_) => {
                // `sol.address_cast`'s operand is the 160-bit address width;
                // emit the zero at that width directly (no constant narrowing).
                let zero = builder.emit_sol_constant(0, builder.types.ui160, block);
                builder.emit_sol_address_cast(zero, mlir_type, block)
            }
            SlangType::ByteArray(byte_array_type) => {
                // `sol.bytes_cast`'s operand must match the fixed-bytes width
                // (`N * 8` bits), so emit the zero at that width directly.
                let bits = byte_array_type.width() * 8;
                let int_type = Type::from(IntegerType::unsigned(builder.context, bits));
                let zero = builder.emit_sol_constant(0, int_type, block);
                builder.emit_sol_bytes_cast(zero, mlir_type, block)
            }
            SlangType::Enum(_) => {
                let zero = builder.emit_sol_constant(0, builder.types.ui256, block);
                builder.emit_sol_enum_cast(zero, mlir_type, block)
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::emit_scalar_zero(&target_type, mlir_type, builder, block)
            }
            SlangType::Function(function_type) => {
                // The zero value of an external function pointer is a zero
                // address + zero selector packed into an `!sol.ext_func_ref`; of
                // an internal one, the dialect's `default_func_constant` (a
                // pointer that reverts when called).
                if function_type.is_externally_visible() {
                    let zero_address = builder.emit_sol_constant(0, builder.types.ui160, block);
                    let address = builder.emit_sol_address_cast(
                        zero_address,
                        builder.types.sol_address,
                        block,
                    );
                    builder.emit_sol_ext_func_constant(address, 0, mlir_type, block)
                } else {
                    builder.emit_sol_default_func_constant(mlir_type, block)
                }
            }
            _ => unreachable!(
                "emit_scalar_zero handles only address/bytesN/enum/integer/bool/UDVT/function value types"
            ),
        }
    }

    // TODO: Remove when nomicFoundation/slang#1793 is merged and we can instead
    // depend on `LiteralType::mobile_type()` for literal type conversion.
    fn integer_bits_required(value: &BigInt) -> u32 {
        if value.is_negative() {
            let magnitude_minus_one = -value - 1u32;
            u32::try_from(magnitude_minus_one.bits())
                .expect("literal magnitude bit count fits in u32")
                + 1
        } else {
            u32::try_from(value.bits())
                .expect("literal bit count fits in u32")
                .max(1)
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

    /// Returns the MLIR target type this conversion produces.
    pub fn to_target_type(&self, builder: &solx_mlir::Builder<'context>) -> Type<'context> {
        match self {
            Self::Bool => builder.types.i1,
            Self::Address => builder.types.sol_address,
            Self::Cast(target_type) => *target_type,
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let slang_type = state_variable
            .get_type()
            .expect("slang types every state variable");
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
        if value.r#type() == self.to_target_type(builder) {
            return value;
        }
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
