//!
//! The projection from Slang's semantic type tree onto Sol dialect types.
//!

use num_traits::sign::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ArraySize;
use solx_mlir::Context as MlirContext;
use solx_mlir::Type as MlirType;

codegen!(
    Type {
        /// Resolves to an MLIR type.
        ///
        /// `inherited_location` is the dialect data location to substitute when a
        /// type's Slang location is `Inherited` (struct-field-relative). Top-level
        /// callers pass `None`; the `Struct` arm sets it to the parent struct's
        /// location for the duration of member resolution.
        pub fn resolve<'context>(
            node: &SlangType,
            inherited_location: Option<solx_utils::DataLocation>,
            context: &MlirContext<'context>,
        ) -> MlirType<'context> {
            match node {
                SlangType::Integer(integer_type) => {
                    let bits = integer_type.bits() as usize;
                    if integer_type.is_signed() {
                        MlirType::signed(context.melior, bits)
                    } else {
                        MlirType::unsigned(context.melior, bits)
                    }
                }
                SlangType::Boolean(_) => MlirType::boolean(context.melior),
                SlangType::Address(_) => MlirType::address(context.melior, false),
                SlangType::Literal(literal_type) => match literal_type.kind() {
                    LiteralKind::Address { .. } => MlirType::address(context.melior, false),
                    LiteralKind::Integer { value } => {
                        let bits = if value.is_negative() {
                            (-&value - 1u32).bits() + 1
                        } else {
                            value.bits().max(1)
                        };
                        let bits = usize::try_from(bits)
                            .expect("a literal's bit count fits the address width")
                            .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                            .max(solx_utils::BIT_LENGTH_BYTE);
                        if value.is_negative() {
                            MlirType::signed(context.melior, bits)
                        } else {
                            MlirType::unsigned(context.melior, bits)
                        }
                    }
                    LiteralKind::HexInteger { bytes, .. } => {
                        let bits = bytes as usize * solx_utils::BIT_LENGTH_BYTE;
                        MlirType::unsigned(context.melior, bits)
                    }
                    LiteralKind::String { .. } => {
                        MlirType::string(context.melior, solx_utils::DataLocation::Memory)
                    }
                    LiteralKind::HexString { bytes } => MlirType::fixed_bytes(
                        context.melior,
                        bytes.try_into().expect("hex string length fits in u32"),
                    ),
                    LiteralKind::Rational { .. } => unimplemented!(
                        "MLIR type resolution is not yet implemented for rational literals"
                    ),
                },
                SlangType::String(string_type) => {
                    let location = solx_utils::DataLocation::from_slang(
                        string_type.location(),
                        inherited_location,
                    );
                    MlirType::string(context.melior, location)
                }
                SlangType::Bytes(bytes_type) => {
                    let location = solx_utils::DataLocation::from_slang(
                        bytes_type.location(),
                        inherited_location,
                    );
                    MlirType::string(context.melior, location)
                }
                SlangType::ByteArray(byte_array_type) => {
                    MlirType::fixed_bytes(context.melior, byte_array_type.width())
                }
                SlangType::Array(array_type) => {
                    let element_type =
                        Self::resolve(&array_type.element_type(), inherited_location, context);
                    let location = solx_utils::DataLocation::from_slang(
                        array_type.location(),
                        inherited_location,
                    );
                    MlirType::array(context.melior, ArraySize::Dynamic, element_type, location)
                }
                SlangType::FixedSizeArray(fixed_array_type) => {
                    let element_type = Self::resolve(
                        &fixed_array_type.element_type(),
                        inherited_location,
                        context,
                    );
                    let location = solx_utils::DataLocation::from_slang(
                        fixed_array_type.location(),
                        inherited_location,
                    );
                    MlirType::array(
                        context.melior,
                        ArraySize::Fixed(fixed_array_type.size() as u64),
                        element_type,
                        location,
                    )
                }
                SlangType::Mapping(mapping_type) => {
                    let key_type = Self::resolve(
                        &mapping_type.key_type(),
                        Some(solx_utils::DataLocation::Storage),
                        context,
                    );
                    let value_type = Self::resolve(
                        &mapping_type.value_type(),
                        Some(solx_utils::DataLocation::Storage),
                        context,
                    );
                    MlirType::mapping(context.melior, key_type, value_type)
                }
                SlangType::Struct(struct_type) => {
                    let struct_location = solx_utils::DataLocation::from_slang(
                        struct_type.location(),
                        inherited_location,
                    );
                    let Definition::Struct(struct_definition) = struct_type.definition() else {
                        unreachable!("Slang StructType always references a Struct definition");
                    };
                    let member_types: Vec<MlirType<'context>> = struct_definition
                        .members()
                        .iter()
                        .map(|member| {
                            Self::resolve(
                                &member
                                    .get_type()
                                    .expect("struct member type resolved by semantic analysis"),
                                Some(struct_location),
                                context,
                            )
                        })
                        .collect();
                    MlirType::structure(context.melior, &member_types, struct_location)
                }
                SlangType::Contract(contract_type) => {
                    let Definition::Contract(contract_definition) = contract_type.definition()
                    else {
                        unreachable!("Slang ContractType always references a Contract definition");
                    };
                    MlirType::contract(
                        context.melior,
                        contract_definition.name().name().as_str(),
                        contract_definition.is_payable(),
                    )
                }
                SlangType::Interface(interface_type) => {
                    let Definition::Interface(interface_definition) = interface_type.definition()
                    else {
                        unreachable!(
                            "Slang InterfaceType always references an Interface definition"
                        );
                    };
                    MlirType::contract(
                        context.melior,
                        interface_definition.name().name().as_str(),
                        false,
                    )
                }
                SlangType::Enum(enum_type) => {
                    let Definition::Enum(enum_definition) = enum_type.definition() else {
                        unreachable!("Slang EnumType always references an Enum definition");
                    };
                    let member_count = enum_definition.members().len();
                    let max =
                        u8::try_from(member_count - 1).expect("enum member count fits in u8");
                    MlirType::enumeration(context.melior, max.into())
                }
                SlangType::UserDefinedValue(udvt) => {
                    let target_type = udvt
                        .target_type()
                        .expect("UDVT target type resolved by semantic analysis");
                    Self::resolve(&target_type, inherited_location, context)
                }
                _ => unimplemented!("unsupported Slang type"),
            }
        }

        /// The MLIR type of the address a `sol.gep` / `sol.map` / `sol.addr_of` yields for a
        /// value of this slang type: mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule:
        /// a reference-typed element living in `Storage` or `CallData` IS its own address, so
        /// the address type is the element type itself rather than a pointer to it.
        pub fn address_type<'context>(
            node: &SlangType,
            element_type: MlirType<'context>,
            base_location: solx_utils::DataLocation,
            context: &MlirContext<'context>,
        ) -> MlirType<'context> {
            if node.is_reference_type()
                && matches!(
                    base_location,
                    solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
                )
            {
                element_type
            } else {
                MlirType::pointer(context.melior, element_type, base_location)
            }
        }
    }
);
