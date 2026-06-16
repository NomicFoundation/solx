//!
//! Slang type → MLIR (Sol dialect) type resolution: the type-level analogue of
//! the `Emit` projection.
//!

use melior::ir::Type as MlirType;
use melior::ir::r#type::IntegerType;
use num::BigInt;
use num::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Type as SlangType;

use super::Type;
use crate::ArraySize;
use crate::ContractPayable;
use crate::LocationPolicy;

/// Resolves a Slang semantic type to its MLIR (Sol dialect) type — the
/// type-level analogue of the `Emit` projection, a local trait on the foreign
/// Slang `Type` so a type resolves itself (`slang_type.resolve_type(..)`).
pub trait ResolveType {
    /// Maps this Slang type to an MLIR type.
    ///
    /// `policy` picks each reference type's data location: [`LocationPolicy::Declared`]
    /// uses the type's own Slang location (with an inherited fallback for
    /// struct-field-relative `Inherited`), [`LocationPolicy::ForceMemory`] forces
    /// the external (ABI) representation where `calldata` cannot cross the call
    /// boundary. Top-level callers pass `LocationPolicy::Declared(None)`; the
    /// `Struct` arm carries the parent struct's location into member resolution.
    fn resolve_type<'context>(
        &self,
        policy: LocationPolicy,
        builder: &crate::Builder<'context>,
    ) -> MlirType<'context>;
}

impl ResolveType for SlangType {
    fn resolve_type<'context>(
        &self,
        policy: LocationPolicy,
        builder: &crate::Builder<'context>,
    ) -> MlirType<'context> {
        match self {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.signed() {
                    MlirType::from(IntegerType::signed(builder.context, bits))
                } else {
                    MlirType::from(IntegerType::unsigned(builder.context, bits))
                }
            }
            SlangType::Boolean(_) => MlirType::from(IntegerType::new(
                builder.context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => Type::address(builder.context, false).into_mlir(),
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => Type::address(builder.context, false).into_mlir(),
                LiteralKind::Integer { value } => {
                    let bits = integer_bits_required(&value) as usize;
                    let bits = bits
                        .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                        .max(solx_utils::BIT_LENGTH_BYTE);
                    let bits = u32::try_from(bits).expect("bit size fits in 32 bits");
                    if value.is_negative() {
                        MlirType::from(IntegerType::signed(builder.context, bits))
                    } else {
                        MlirType::from(IntegerType::unsigned(builder.context, bits))
                    }
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    MlirType::from(IntegerType::unsigned(builder.context, bits))
                }
                LiteralKind::String { .. } => {
                    Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir()
                }
                LiteralKind::HexString { bytes } => Type::fixed_bytes(
                    builder.context,
                    bytes.try_into().expect("hex string length fits in u32"),
                )
                .into_mlir(),
                LiteralKind::Rational { .. } => {
                    // Sentinel: a rational appears only as a compile-time
                    // intermediate that constant folding consumes (see the folding
                    // gate in `ExpressionContext::emit`); a rational that survived
                    // to runtime would fail downstream, not at type resolution.
                    Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
                }
            },
            SlangType::String(string_type) => Type::string(
                builder.context,
                policy.data_location(string_type.location()),
            )
            .into_mlir(),
            SlangType::Bytes(bytes_type) => {
                Type::string(builder.context, policy.data_location(bytes_type.location()))
                    .into_mlir()
            }
            SlangType::ByteArray(byte_array_type) => {
                Type::fixed_bytes(builder.context, byte_array_type.width()).into_mlir()
            }
            SlangType::Array(array_type) => {
                let element_type = array_type.element_type().resolve_type(policy, builder);
                let location = policy.data_location(array_type.location());
                Type::array(builder.context, ArraySize::Dynamic, element_type, location).into_mlir()
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = fixed_array_type
                    .element_type()
                    .resolve_type(policy, builder);
                let location = policy.data_location(fixed_array_type.location());
                Type::array(
                    builder.context,
                    ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = mapping_type.key_type().resolve_type(
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    builder,
                );
                let value_type = mapping_type.value_type().resolve_type(
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    builder,
                );
                Type::mapping(builder.context, key_type, value_type).into_mlir()
            }
            SlangType::Struct(struct_type) => {
                let struct_location = policy.data_location(struct_type.location());
                let member_policy = policy.within_struct(struct_location);
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
                    member_types.push(member_slang_type.resolve_type(member_policy, builder));
                }
                Type::structure(builder.context, &member_types, struct_location).into_mlir()
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                Type::contract(
                    builder.context,
                    contract_definition.name().name().as_str(),
                    contract_definition.is_payable(),
                )
                .into_mlir()
            }
            SlangType::Interface(interface_type) => {
                let interface_definition = match interface_type.definition() {
                    Definition::Interface(definition) => definition,
                    _ => {
                        unreachable!(
                            "Slang InterfaceType always references an Interface definition"
                        )
                    }
                };
                // Interfaces are never `payable` themselves; payability lives
                // on the address-cast at the call site.
                Type::contract(
                    builder.context,
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
                // Solidity caps enums at 256 members, so the max enumerator
                // index always fits in a `u8`.
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                Type::enumeration(builder.context, max.into()).into_mlir()
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                target_type.resolve_type(policy, builder)
            }
            SlangType::Function(function_type) => {
                // A function pointer lowers to `!sol.func_ref<fnTy>` (internal)
                // or `!sol.ext_func_ref<fnTy>` (external — address + selector).
                // A void return contributes zero result types; a tuple return
                // expands to one result per element.
                let parameter_types: Vec<_> = function_type
                    .parameter_types()
                    .iter()
                    .map(|parameter_type| {
                        parameter_type.resolve_type(LocationPolicy::Declared(None), builder)
                    })
                    .collect();
                let result_types: Vec<_> = match function_type.return_type() {
                    SlangType::Void(_) => Vec::new(),
                    SlangType::Tuple(tuple_type) => tuple_type
                        .types()
                        .iter()
                        .map(|element_type| {
                            element_type.resolve_type(LocationPolicy::Declared(None), builder)
                        })
                        .collect(),
                    other => {
                        vec![other.resolve_type(LocationPolicy::Declared(None), builder)]
                    }
                };
                if function_type.is_externally_visible() {
                    Type::ext_func_ref(builder.context, &parameter_types, &result_types).into_mlir()
                } else {
                    Type::func_ref(builder.context, &parameter_types, &result_types).into_mlir()
                }
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }
}

// TODO: Remove when nomicFoundation/slang#1793 is merged and we can instead
// depend on `LiteralType::mobile_type()` for literal type conversion.
fn integer_bits_required(value: &BigInt) -> u32 {
    if value.is_negative() {
        let magnitude_minus_one = -value - 1u32;
        u32::try_from(magnitude_minus_one.bits()).expect("literal magnitude bit count fits in u32")
            + 1
    } else {
        u32::try_from(value.bits())
            .expect("literal bit count fits in u32")
            .max(1)
    }
}
