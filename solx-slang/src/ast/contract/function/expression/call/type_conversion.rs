//!
//! Solidity type conversion classification and dispatch.
//!

use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;

use solx_mlir::ArraySize;
use solx_mlir::CmpPredicate;
use solx_mlir::Context;
use solx_mlir::Type;
use solx_mlir::Value;

use crate::ast::contract::ContractEmitter;

/// Classification of Solidity type conversions.
///
/// Used for both explicit conversions (`uint256(x)`, `address(x)`, `bool(x)`)
/// and implicit operand widening in arithmetic, assignment, and comparison.
pub enum TypeConversion<'context> {
    /// `bool(x)`: comparison against zero, not bit-truncation.
    Bool,
    /// `address(x)` / `payable(x)`: `sol.cast` to ui160 then `sol.address_cast`.
    Address,
    /// Integer type cast: `sol.cast` to target.
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
                let bits = integer_type.bits() as usize;
                if integer_type.is_signed() {
                    Type::signed(context.melior, bits)
                } else {
                    Type::unsigned(context.melior, bits)
                }
            }
            SlangType::Boolean(_) => Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN),
            SlangType::Address(_) => Type::address(context.melior, false),
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => Type::address(context.melior, false),
                LiteralKind::Integer { .. } => {
                    let mobile_type = literal_type
                        .mobile_type()
                        .expect("integer literal fits in 256 bits");
                    Self::resolve_slang_type(&mobile_type, inherited_location, context)
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes as usize * solx_utils::BIT_LENGTH_BYTE;
                    Type::unsigned(context.melior, bits)
                }
                LiteralKind::String { .. } => {
                    Type::string(context.melior, solx_utils::DataLocation::Memory)
                }
                LiteralKind::HexString { bytes } => Type::fixed_bytes(
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
                Type::string(context.melior, location)
            }
            SlangType::Bytes(bytes_type) => {
                let location =
                    solx_utils::DataLocation::from_slang(bytes_type.location(), inherited_location);
                Type::string(context.melior, location)
            }
            SlangType::ByteArray(byte_array_type) => {
                Type::fixed_bytes(context.melior, byte_array_type.width())
            }
            SlangType::Array(array_type) => {
                let element_type = Self::resolve_slang_type(
                    &array_type.element_type(),
                    inherited_location,
                    context,
                );
                let location =
                    solx_utils::DataLocation::from_slang(array_type.location(), inherited_location);
                Type::array(context.melior, ArraySize::Dynamic, element_type, location)
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
                Type::array(
                    context.melior,
                    ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
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
                Type::mapping(context.melior, key_type, value_type)
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
                Type::structure(context.melior, &member_types, struct_location)
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                Type::contract(
                    context.melior,
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
                Type::contract(
                    context.melior,
                    interface_definition.name().name().as_str(),
                    false,
                )
            }
            SlangType::Enum(enum_type) => {
                let enum_definition = match enum_type.definition() {
                    Definition::Enum(definition) => definition,
                    _ => unreachable!("Slang EnumType always references an Enum definition"),
                };
                let member_count = enum_definition.members().iter().count();
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                Type::enumeration(context.melior, max.into())
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::resolve_slang_type(&target_type, inherited_location, context)
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// Classifies a target type into the appropriate conversion variant.
    pub fn from_target_type(target_type: Type<'context>, context: &Context<'context>) -> Self {
        if target_type == Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN) {
            Self::Bool
        } else if target_type == Type::address(context.melior, false) {
            Self::Address
        } else {
            Self::Cast(target_type)
        }
    }

    /// Returns the MLIR target type this conversion produces.
    pub fn to_target_type(&self, context: &Context<'context>) -> Type<'context> {
        match self {
            Self::Bool => Type::signless(context.melior, solx_utils::BIT_LENGTH_BOOLEAN),
            Self::Address => Type::address(context.melior, false),
            Self::Cast(target_type) => *target_type,
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        context: &Context<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let name = state_variable.name().name();
        let slang_type = state_variable
            .get_type()
            .ok_or_else(|| anyhow::anyhow!("unresolved type for state variable: {name}"))?;
        Ok(Self::resolve_slang_type(&slang_type, None, context))
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
    pub fn emit(self, value: Value<'context>, context: &Context<'context>) -> Value<'context> {
        if value.r#type() == self.to_target_type(context) {
            return value;
        }
        match self {
            Self::Bool => {
                let zero = Value::constant(0, value.r#type(), context);
                value.compare(zero, CmpPredicate::Ne, context)
            }
            Self::Address => {
                let address_type = Type::address(context.melior, false);
                let truncated = if value.r#type().is_integer() {
                    let ui160 = Type::unsigned(context.melior, solx_utils::BIT_LENGTH_ETH_ADDRESS);
                    value.cast(ui160, context)
                } else {
                    value
                };
                truncated.address_cast(address_type, context)
            }
            Self::Cast(target_type) => value.cast(target_type, context),
        }
    }
}
