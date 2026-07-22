//!
//! The projection from Slang's semantic type tree onto Sol dialect types.
//!

use num_traits::sign::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Type;

use solx_mlir::ArraySize;
use solx_mlir::Type as MlirType;

use crate::scope::source_unit::SourceUnitScope;

impl<'context> SourceUnitScope<'context> {
    /// Resolves a Slang semantic type to its Sol dialect MLIR type.
    ///
    /// `inherited_location` is the dialect data location to substitute when a type's Slang location
    /// is `Inherited` (struct-field-relative). Top-level callers pass `None`; the `Struct` arm sets
    /// it to the parent struct's location for the duration of member resolution.
    pub fn resolve(
        &self,
        node: &Type,
        inherited_location: Option<solx_utils::DataLocation>,
    ) -> MlirType<'context> {
        match node {
            Type::Integer(integer_type) => MlirType::integer(
                self.melior,
                integer_type.bits() as usize,
                integer_type.is_signed(),
            ),
            Type::FixedPointNumber(fixed_point_type) => MlirType::integer(
                self.melior,
                fixed_point_type.bits() as usize,
                fixed_point_type.is_signed(),
            ),
            Type::Boolean(_) => MlirType::boolean(self.melior),
            Type::Address(_) => MlirType::address(self.melior, false),
            Type::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => MlirType::address(self.melior, false),
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
                    MlirType::integer(self.melior, bits, value.is_negative())
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes as usize * solx_utils::BIT_LENGTH_BYTE;
                    MlirType::unsigned(self.melior, bits)
                }
                LiteralKind::String { .. } => {
                    MlirType::string(self.melior, solx_utils::DataLocation::Memory)
                }
                LiteralKind::HexString { bytes } => MlirType::fixed_bytes(
                    self.melior,
                    bytes.try_into().expect("hex string length fits in u32"),
                ),
                LiteralKind::Rational { .. } => unimplemented!(
                    "MLIR type resolution is not yet implemented for rational literals"
                ),
            },
            Type::String(string_type) => {
                let location = solx_utils::DataLocation::from_slang(
                    string_type.location(),
                    inherited_location,
                );
                MlirType::string(self.melior, location)
            }
            Type::Bytes(bytes_type) => {
                let location =
                    solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
                MlirType::string(self.melior, location)
            }
            Type::ByteArray(byte_array_type) => {
                MlirType::fixed_bytes(self.melior, byte_array_type.width())
            }
            Type::Array(array_type) => {
                let element_type = self.resolve(&array_type.element_type(), inherited_location);
                let location =
                    solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                MlirType::array(self.melior, ArraySize::Dynamic, element_type, location)
            }
            Type::FixedSizeArray(fixed_array_type) => {
                let element_type =
                    self.resolve(&fixed_array_type.element_type(), inherited_location);
                let location = solx_utils::DataLocation::from_slang(
                    fixed_array_type.location(),
                    inherited_location,
                );
                MlirType::array(
                    self.melior,
                    ArraySize::Fixed(
                        u64::try_from(fixed_array_type.size()).expect("fixed array size fits u64"),
                    ),
                    element_type,
                    location,
                )
            }
            Type::Mapping(mapping_type) => {
                let key_type = self.resolve(
                    &mapping_type.key_type(),
                    Some(solx_utils::DataLocation::Storage),
                );
                let value_type = self.resolve(
                    &mapping_type.value_type(),
                    Some(solx_utils::DataLocation::Storage),
                );
                MlirType::mapping(self.melior, key_type, value_type)
            }
            Type::Struct(struct_type) => {
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
                        self.resolve(
                            &member
                                .get_type()
                                .expect("struct member type resolved by semantic analysis"),
                            Some(struct_location),
                        )
                    })
                    .collect();
                MlirType::structure(self.melior, &member_types, struct_location)
            }
            Type::Contract(contract_type) => {
                let Definition::Contract(contract_definition) = contract_type.definition() else {
                    unreachable!("Slang ContractType always references a Contract definition");
                };
                MlirType::contract(
                    self.melior,
                    contract_definition.name().name().as_str(),
                    contract_definition.is_payable(),
                )
            }
            Type::Interface(interface_type) => {
                let Definition::Interface(interface_definition) = interface_type.definition()
                else {
                    unreachable!("Slang InterfaceType always references an Interface definition");
                };
                MlirType::contract(
                    self.melior,
                    interface_definition.name().name().as_str(),
                    false,
                )
            }
            Type::Enum(enum_type) => {
                let Definition::Enum(enum_definition) = enum_type.definition() else {
                    unreachable!("Slang EnumType always references an Enum definition");
                };
                let member_count = enum_definition.members().len();
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                MlirType::enumeration(self.melior, max.into())
            }
            Type::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                self.resolve(&target_type, inherited_location)
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Resolves the binder's typing of a node to its Sol dialect MLIR type.
    pub fn typing(&self, slang_type: Option<Type>) -> MlirType<'context> {
        self.resolve(
            &slang_type.expect("the binder types every expression"),
            None,
        )
    }

    /// The MLIR pointer type a `sol.gep` / `sol.map` / `sol.addr_of` yields for a value of this
    /// Slang type: mirrors `Sol_GepOp::build`'s non-ptr-ref-in-storage rule, where a
    /// reference-typed element living in `Storage` or `CallData` is its own storage pointer, so
    /// the pointer type is the element type itself.
    pub fn pointer(
        &self,
        node: &Type,
        element_type: MlirType<'context>,
        base_location: solx_utils::DataLocation,
    ) -> MlirType<'context> {
        if node.is_reference_type()
            && matches!(
                base_location,
                solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
            )
        {
            return element_type;
        }
        MlirType::pointer(self.melior, element_type, base_location)
    }
}
