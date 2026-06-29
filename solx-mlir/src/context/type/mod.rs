//!
//! An MLIR type in the Sol dialect: its construction, predicates, and the casts it routes.
//!

pub mod array_size;
pub mod location_policy;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::DataLocation as SlangDataLocation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::Type as SlangType;

use crate::Context;
use crate::IntoOds;
use crate::Value;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::ContractCastOperation;
use crate::ods::sol::DataLocCastOperation;
use crate::ods::sol::DynBytesToFixedBytesOperation;
use crate::ods::sol::EnumCastOperation;

use self::array_size::ArraySize;
use self::location_policy::LocationPolicy;

/// An MLIR type in the Sol dialect: type construction, the kind predicates, and the cast router [`Self::cast`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Type<'context> {
    inner: MlirType<'context>,
}

impl<'context> Type<'context> {
    /// Bit width of a Solidity function selector (4 bytes).
    pub const SELECTOR_BIT_WIDTH: u32 = solx_utils::BIT_LENGTH_X32 as u32;

    /// Wraps a melior type.
    pub fn new(inner: MlirType<'context>) -> Self {
        Self { inner }
    }

    /// The inner melior type, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirType<'context> {
        self.inner
    }

    /// Resolves a possibly-absent Slang type (the `Option`-lift over [`Self::resolve`]).
    pub fn resolve_optional(
        slang_type: Option<SlangType>,
        context: &Context<'context>,
    ) -> Option<MlirType<'context>> {
        Some(Self::resolve(
            &slang_type?,
            LocationPolicy::Declared(None),
            context,
        ))
    }

    /// Resolves a state variable's declared type (Slang always types one) in its
    /// declared location.
    pub fn resolve_state_variable(
        slang_type: &SlangType,
        context: &Context<'context>,
    ) -> MlirType<'context> {
        Self::resolve(slang_type, LocationPolicy::Declared(None), context)
    }

    /// An unsigned integer type of `bits` width (`ui<bits>`).
    pub fn unsigned(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::unsigned(context, bits as u32)))
    }

    /// A signless integer type of `bits` width (`i<bits>`) — the boolean `i1` and the Yul word `i256`.
    pub fn signless(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::new(context, bits as u32)))
    }

    /// The opaque LLVM pointer type (`!llvm.ptr`) — a Yul-local slot.
    pub fn llvm_ptr(context: &'context melior::Context) -> Self {
        Self::new(melior::dialect::llvm::r#type::pointer(context, 0))
    }

    /// A `sol::AddressType` with the given payability.
    pub fn address(context: &'context melior::Context, payable: bool) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateAddressType(context.to_raw(), payable))
        })
    }

    /// A `sol::PointerType` with the given element type and data location.
    pub fn pointer(
        context: &'context melior::Context,
        element_type: MlirType<'context>,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreatePointerType(
                context.to_raw(),
                element_type.to_raw(),
                location as u32,
            ))
        })
    }

    /// A `sol::ContractType` for the named contract with the given payability.
    pub fn contract(context: &'context melior::Context, name: &str, payable: bool) -> Self {
        let name_bytes = name.as_bytes();
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateContractType(
                context.to_raw(),
                name_bytes.as_ptr() as *const std::ffi::c_char,
                name_bytes.len(),
                payable,
            ))
        })
    }

    /// A `sol::StringType` at the given data location (`bytes` and `string`
    /// share `!sol.string`).
    pub fn string(context: &'context melior::Context, location: solx_utils::DataLocation) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateStringType(
                context.to_raw(),
                location as u32,
            ))
        })
    }

    /// A `sol::FixedBytesType` of the given byte width.
    pub fn fixed_bytes(context: &'context melior::Context, width: u32) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateFixedBytesType(
                context.to_raw(),
                width,
            ))
        })
    }

    /// The single `sol::ByteType` (the `bytes`/`string` element type).
    pub fn byte(context: &'context melior::Context) -> Self {
        Self::new(unsafe { MlirType::from_raw(crate::ffi::solxCreateByteType(context.to_raw())) })
    }

    /// A `sol::ArrayType` of `element_type` at `location`.
    pub fn array(
        context: &'context melior::Context,
        size: ArraySize,
        element_type: MlirType<'context>,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateArrayType(
                context.to_raw(),
                size.as_dialect_i64(),
                element_type.to_raw(),
                location as u32,
            ))
        })
    }

    /// A `sol::MappingType` with the given key and value types.
    pub fn mapping(
        context: &'context melior::Context,
        key_type: MlirType<'context>,
        value_type: MlirType<'context>,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateMappingType(
                context.to_raw(),
                key_type.to_raw(),
                value_type.to_raw(),
            ))
        })
    }

    /// A `sol::StructType` from member types and a data location.
    pub fn structure(
        context: &'context melior::Context,
        member_types: &[MlirType<'context>],
        location: solx_utils::DataLocation,
    ) -> Self {
        let raw_types: Vec<mlir_sys::MlirType> = member_types.iter().map(|t| t.to_raw()).collect();
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateStructType(
                context.to_raw(),
                raw_types.as_ptr(),
                raw_types.len(),
                location as u32,
            ))
        })
    }

    /// A `sol::EnumType` whose maximum valid value is `max` (one less than the
    /// number of enum members).
    pub fn enumeration(context: &'context melior::Context, max: u32) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateEnumType(context.to_raw(), max))
        })
    }

    /// A `sol::FuncRefType` — an internal function pointer over `parameter_types -> result_types`.
    pub fn func_ref(
        context: &'context melior::Context,
        parameter_types: &[MlirType<'context>],
        result_types: &[MlirType<'context>],
    ) -> Self {
        let parameters: Vec<_> = parameter_types.iter().map(|t| t.to_raw()).collect();
        let results: Vec<_> = result_types.iter().map(|t| t.to_raw()).collect();
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateFuncRefType(
                context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        })
    }

    /// A `sol::ExtFuncRefType` — an external function reference (address + selector) over `parameter_types -> result_types`.
    pub fn ext_func_ref(
        context: &'context melior::Context,
        parameter_types: &[MlirType<'context>],
        result_types: &[MlirType<'context>],
    ) -> Self {
        let parameters: Vec<_> = parameter_types.iter().map(|t| t.to_raw()).collect();
        let results: Vec<_> = result_types.iter().map(|t| t.to_raw()).collect();
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxCreateExtFuncRefType(
                context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        })
    }

    /// Whether this is a Sol enum type (`!sol.enum<N>`).
    pub fn is_enum(self) -> bool {
        unsafe { crate::ffi::solxIsEnumType(self.inner.to_raw()) }
    }

    /// Whether this is the Sol address type (`!sol.address`).
    pub fn is_address(self) -> bool {
        unsafe { crate::ffi::solxIsAddressType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol contract type (`!sol.contract<…>`).
    pub fn is_contract(self) -> bool {
        unsafe { crate::ffi::solxIsContractType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol fixed-bytes type (`!sol.fixedbytes<N>`).
    pub fn is_fixed_bytes(self) -> bool {
        unsafe { crate::ffi::solxIsFixedBytesType(self.inner.to_raw()) }
    }

    /// Whether this is the single-byte `!sol.byte` (distinct from `!sol.fixedbytes<1>`).
    pub fn is_byte(self) -> bool {
        unsafe { crate::ffi::solxIsByteType(self.inner.to_raw()) }
    }

    /// Whether this is the dynamic-bytes type `!sol.string` (shared by `string` and `bytes`).
    pub fn is_string(self) -> bool {
        unsafe { crate::ffi::solxIsStringType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol array type (`!sol.array<…>`).
    pub fn is_array(self) -> bool {
        unsafe { crate::ffi::solxIsArrayType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol struct type (`!sol.struct<…>`).
    pub fn is_struct(self) -> bool {
        unsafe { crate::ffi::solxIsStructType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol mapping type (`!sol.mapping<…>`).
    pub fn is_mapping(self) -> bool {
        unsafe { crate::ffi::solxIsMappingType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol reference type: array, struct, string/`bytes`, or mapping.
    pub fn is_reference(self) -> bool {
        self.is_string() || self.is_array() || self.is_struct() || self.is_mapping()
    }

    /// Whether this is a Sol function reference of either kind (internal or external).
    pub fn is_function_ref(self) -> bool {
        let raw = self.inner.to_raw();
        unsafe { crate::ffi::solxIsFuncRefType(raw) || crate::ffi::solxIsExtFuncRefType(raw) }
    }

    /// Whether this is a Sol external function reference (`!sol.ext_func_ref<…>`).
    pub fn is_ext_function_ref(self) -> bool {
        unsafe { crate::ffi::solxIsExtFuncRefType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol pointer (`!sol.ptr<T, Loc>`) — a typed place.
    pub fn is_pointer(self) -> bool {
        unsafe { crate::ffi::solxIsPointerType(self.inner.to_raw()) }
    }

    /// The pointee type `T` of a `!sol.ptr<T, Loc>` (the caller must ensure this is a pointer).
    pub fn pointee(self) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::solxPointerTypePointeeType(self.inner.to_raw()))
        })
    }

    /// The data location of a pointer's `Loc` or a string/array/struct's own
    /// location.
    pub fn data_location(self) -> solx_utils::DataLocation {
        let raw = self.inner.to_raw();
        let ordinal = if self.is_pointer() {
            unsafe { crate::ffi::solxPointerTypeDataLocation(raw) }
        } else {
            unsafe { crate::ffi::solxReferenceTypeDataLocation(raw) }
        };
        solx_utils::DataLocation::try_from(ordinal).unwrap_or_else(|ordinal| {
            unreachable!("unexpected !sol.ptr data-location ordinal {ordinal}")
        })
    }

    /// The element / field type reached by stepping into this aggregate (the index is ignored for non-structs).
    pub fn element_type(self, field_index: usize) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(crate::ffi::mlirSolGetEltType(
                self.inner.to_raw(),
                field_index as u64,
            ))
        })
    }

    /// The place type addressing an element of `self` at `location`: a reference element in `Storage` /
    /// `CallData` is its own place, every other element a `!sol.ptr<self, location>`.
    pub fn address_type(
        self,
        location: solx_utils::DataLocation,
        context: &'context melior::Context,
    ) -> Self {
        if self.is_reference()
            && matches!(
                location,
                solx_utils::DataLocation::Storage | solx_utils::DataLocation::CallData
            )
        {
            self
        } else {
            Self::pointer(context, self.inner, location)
        }
    }

    /// The byte width of a fixed-bytes-like type: `N` for `!sol.fixedbytes<N>`,
    /// `1` for the single `!sol.byte`, and `None` for any other type.
    pub fn fixed_bytes_or_byte_width(self) -> Option<u32> {
        if self.is_fixed_bytes() {
            Some(unsafe { crate::ffi::solxFixedBytesTypeSize(self.inner.to_raw()) })
        } else if self.is_byte() {
            Some(1)
        } else {
            None
        }
    }

    /// The bit width of an integer type, or 256 for any non-integer type.
    pub fn integer_bit_width(self) -> u32 {
        IntegerType::try_from(self.inner).map_or(solx_utils::BIT_LENGTH_FIELD as u32, |integer| {
            integer.width()
        })
    }

    /// Casts `value` to this (target) type, returning it unchanged when it already has this type.
    ///
    /// `sol.cast` is integer-only, so each non-integer kind (enum, address, contract, fixed-bytes,
    /// reference) routes to its dedicated cast op; this is the single place that classifies and dispatches.
    pub fn cast<'block>(
        self,
        value: Value<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        let source = value.r#type();
        if source == self {
            return value;
        }
        // Enum ↔ integer (`sol.enum_cast` accepts the integer-backed enum;
        // narrowing to an enum range-checks and may revert).
        if source.is_enum() || self.is_enum() {
            return Value::new(mlir_op!(
                context,
                block,
                EnumCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        // Contract ↔ contract (inheritance up/downcast, interface).
        if source.is_contract() && self.is_contract() {
            return Value::new(mlir_op!(
                context,
                block,
                ContractCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        // address ↔ {integer, contract, fixedbytes<20>}. `sol.address_cast`
        // requires the integer side to be exactly `ui160`, so a wider/narrower
        // integer bridges through `ui160` (then a plain `sol.cast` resizes it).
        if source.is_address() || self.is_address() {
            let ui160 = Self::unsigned(context.mlir(), solx_utils::BIT_LENGTH_ETH_ADDRESS);
            if source.is_address() {
                if self.is_contract() || self.is_fixed_bytes() || self == ui160 {
                    return self.address_cast(value, context, block);
                }
                let as_160 = ui160.address_cast(value, context, block);
                return self.cast(as_160, context, block);
            }
            if source.is_contract() || source.is_fixed_bytes() || source == ui160 {
                return self.address_cast(value, context, block);
            }
            let as_160 = ui160.cast(value, context, block);
            return self.address_cast(as_160, context, block);
        }
        // Dynamic `bytes`/`string` → `bytesN`: take the leading N bytes via the
        // dedicated op (`sol.bytes_cast` rejects a `!sol.string` operand).
        if source.is_reference() && self.is_fixed_bytes() {
            return Value::new(mlir_op!(
                context,
                block,
                DynBytesToFixedBytesOperation
                    .inp(value.into_mlir())
                    .out(self.inner)
            ));
        }
        // byte / bytesN ↔ {byte, bytesN, integer}. `sol.bytes_cast` connects `fixedbytes<N>` ↔ `ui(N*8)`
        // directly; an integer counterpart of a different width is first resized through that partner integer.
        if source.is_fixed_bytes() || source.is_byte() {
            let partner_bits = Self::partner_bits(source);
            if let Ok(integer) = IntegerType::try_from(self.inner)
                && integer.width() != partner_bits
            {
                let partner = Self::unsigned(context.mlir(), partner_bits as usize);
                let as_int = partner.bytes_cast(value, context, block);
                return self.cast(as_int, context, block);
            }
            return self.bytes_cast(value, context, block);
        }
        if self.is_fixed_bytes() || self.is_byte() {
            let partner_bits = Self::partner_bits(self);
            if let Ok(integer) = IntegerType::try_from(source.into_mlir())
                && integer.width() != partner_bits
            {
                let partner = Self::unsigned(context.mlir(), partner_bits as usize);
                let as_int = partner.cast(value, context, block);
                return self.bytes_cast(as_int, context, block);
            }
            return self.bytes_cast(value, context, block);
        }
        // Reference types (array / struct / string / bytes / mapping) differ
        // only by data location; a reference→reference cast routes through
        // `sol.data_loc_cast`.
        if source.is_reference() && self.is_reference() {
            return Value::new(mlir_op!(
                context,
                block,
                DataLocCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        Value::new(mlir_op!(
            context,
            block,
            CastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// Emits a `sol.bytes_cast` casting `value` to this byte / fixed-bytes / integer target.
    fn bytes_cast<'block>(
        self,
        value: Value<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Value::new(mlir_op!(
            context,
            block,
            BytesCastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// Emits a `sol.address_cast` to this (address-side) type.
    fn address_cast<'block>(
        self,
        value: Value<'context, 'block>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Value::new(mlir_op!(
            context,
            block,
            AddressCastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// The bit width of the integer a `sol.bytes_cast` pairs with a fixed-bytes type (`8 * N`, or 8 for `!sol.byte`).
    fn partner_bits(r#type: Type<'context>) -> u32 {
        r#type
            .fixed_bytes_or_byte_width()
            .expect("a fixed-bytes / byte type has a width")
            * 8
    }
}

impl<'context> Type<'context> {
    /// Resolves a Slang semantic type to its MLIR (Sol dialect) type.
    ///
    /// `policy` picks each reference type's data location (declared location, or forced to memory
    /// for the external ABI representation); the `Struct` arm carries the parent's location into members.
    pub fn resolve(
        slang_type: &SlangType,
        policy: LocationPolicy,
        context: &Context<'context>,
    ) -> MlirType<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.is_signed() {
                    MlirType::from(IntegerType::signed(context.mlir(), bits))
                } else {
                    MlirType::from(IntegerType::unsigned(context.mlir(), bits))
                }
            }
            SlangType::Boolean(_) => MlirType::from(IntegerType::new(
                context.mlir(),
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => Type::address(context.mlir(), false).into_mlir(),
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => Type::address(context.mlir(), false).into_mlir(),
                LiteralKind::Integer { .. } => {
                    let mobile_type = literal_type
                        .mobile_type()
                        .expect("slang validated: integer literal fits in 256 bits");
                    Type::resolve(&mobile_type, policy, context)
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    MlirType::from(IntegerType::unsigned(context.mlir(), bits))
                }
                LiteralKind::String { .. } => {
                    Type::string(context.mlir(), solx_utils::DataLocation::Memory).into_mlir()
                }
                LiteralKind::HexString { bytes } => Type::fixed_bytes(
                    context.mlir(),
                    bytes.try_into().expect("hex string length fits in u32"),
                )
                .into_mlir(),
                LiteralKind::Rational { .. } => {
                    // A rational appears only as a compile-time intermediate that constant
                    // folding consumes; one surviving to runtime would fail downstream, not here.
                    Type::unsigned(context.mlir(), solx_utils::BIT_LENGTH_FIELD).into_mlir()
                }
            },
            SlangType::String(string_type) => {
                Type::string(context.mlir(), policy.data_location(string_type.location()))
                    .into_mlir()
            }
            SlangType::Bytes(bytes_type) => {
                Type::string(context.mlir(), policy.data_location(bytes_type.location()))
                    .into_mlir()
            }
            SlangType::ByteArray(byte_array_type) => {
                Type::fixed_bytes(context.mlir(), byte_array_type.width()).into_mlir()
            }
            SlangType::Array(array_type) => {
                let element_type = Type::resolve(&array_type.element_type(), policy, context);
                let location = policy.data_location(array_type.location());
                Type::array(context.mlir(), ArraySize::Dynamic, element_type, location).into_mlir()
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = Type::resolve(&fixed_array_type.element_type(), policy, context);
                let location = policy.data_location(fixed_array_type.location());
                Type::array(
                    context.mlir(),
                    ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Type::resolve(
                    &mapping_type.key_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    context,
                );
                let value_type = Type::resolve(
                    &mapping_type.value_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    context,
                );
                Type::mapping(context.mlir(), key_type, value_type).into_mlir()
            }
            SlangType::Struct(struct_type) => {
                let struct_location = policy.data_location(struct_type.location());
                let member_policy = policy.within_struct(struct_location);
                let struct_definition = match struct_type.definition() {
                    Definition::Struct(definition) => definition,
                    _ => unreachable!("Slang StructType always references a Struct definition"),
                };
                let mut member_types = Vec::new();
                for member in struct_definition.members().iter() {
                    let member_slang_type = member.get_type().expect("slang validated");
                    member_types.push(Type::resolve(&member_slang_type, member_policy, context));
                }
                Type::structure(context.mlir(), &member_types, struct_location).into_mlir()
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                Type::contract(
                    context.mlir(),
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
                    context.mlir(),
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
                Type::enumeration(context.mlir(), max.into()).into_mlir()
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt.target_type().expect("slang validated");
                Type::resolve(&target_type, policy, context)
            }
            SlangType::Function(function_type) => {
                // A function pointer lowers to `!sol.func_ref<fnTy>` (internal)
                // or `!sol.ext_func_ref<fnTy>` (external — address + selector).
                let (parameter_types, result_types) =
                    Type::function_pointer_signature(slang_type, context);
                if function_type.is_externally_visible() {
                    Type::ext_func_ref(context.mlir(), &parameter_types, &result_types).into_mlir()
                } else {
                    Type::func_ref(context.mlir(), &parameter_types, &result_types).into_mlir()
                }
            }
            SlangType::FixedPointNumber(fixed_point_type) => {
                let bits = fixed_point_type.bits();
                if fixed_point_type.is_signed() {
                    MlirType::from(IntegerType::signed(context.mlir(), bits))
                } else {
                    MlirType::from(IntegerType::unsigned(context.mlir(), bits))
                }
            }
            SlangType::Library(_) => Type::address(context.mlir(), false).into_mlir(),
            SlangType::Tuple(_) | SlangType::Void(_) => {
                unreachable!("tuple and void are resolved via Type::resolve_result_types")
            }
        }
    }

    /// The MLIR element type and data location of a dynamic-array / `bytes` base (the `.push` receiver).
    pub fn dynamic_array_element(
        base_type: &SlangType,
        context: &Context<'context>,
    ) -> (MlirType<'context>, solx_utils::DataLocation) {
        let (element_type, slang_location) = match base_type {
            SlangType::Array(array_type) => (
                Type::resolve(
                    &array_type.element_type(),
                    LocationPolicy::Declared(None),
                    context,
                ),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (
                Type::byte(context.mlir()).into_mlir(),
                bytes_type.location(),
            ),
            other => unreachable!(
                "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let location = match slang_location {
            SlangDataLocation::Inherited => {
                unreachable!("slang's binder should not surface Inherited at an array push base")
            }
            other => solx_utils::DataLocation::from_slang(other, None),
        };
        (element_type, location)
    }

    /// Resolves a return-position Slang type to MLIR result types: `void` is zero, a tuple expands per element.
    pub fn resolve_result_types(
        return_type: &SlangType,
        context: &Context<'context>,
    ) -> Vec<MlirType<'context>> {
        match return_type {
            SlangType::Void(_) => Vec::new(),
            SlangType::Tuple(tuple_type) => tuple_type
                .types()
                .iter()
                .map(|element_type| {
                    Type::resolve(element_type, LocationPolicy::Declared(None), context)
                })
                .collect(),
            other => vec![Type::resolve(
                other,
                LocationPolicy::Declared(None),
                context,
            )],
        }
    }

    /// Resolves a function-pointer callee type's `(parameter_types, result_types)` from Slang to MLIR.
    pub fn function_pointer_signature(
        callee_type: &SlangType,
        context: &Context<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>) {
        let SlangType::Function(function_type) = callee_type else {
            unreachable!("an indirect-call callee is always a function type");
        };
        let parameter_types = function_type
            .parameter_types()
            .iter()
            .map(|parameter_type| {
                Type::resolve(parameter_type, LocationPolicy::Declared(None), context)
            })
            .collect();
        let result_types = Type::resolve_result_types(&function_type.return_type(), context);
        (parameter_types, result_types)
    }

    /// Resolves a parameter's declared MLIR type from its Slang type.
    pub fn parameter(
        slang_type: Option<&SlangType>,
        context: &Context<'context>,
    ) -> MlirType<'context> {
        Type::resolve(
            slang_type.expect("slang validated"),
            LocationPolicy::Declared(None),
            context,
        )
    }
}

impl<'context> Type<'context> {
    /// Resolves a function's `(parameter_types, return_types)` from Slang to MLIR under `policy`
    /// (the declared signature, or the external ABI signature that forces reference types to memory).
    pub fn resolve_signature(
        function: &FunctionDefinition,
        policy: LocationPolicy,
        context: &Context<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>) {
        let resolve = |parameter: Parameter| {
            Type::resolve(
                &parameter.get_type().expect("slang validated"),
                policy,
                context,
            )
        };
        let parameter_types = function.parameters().iter().map(&resolve).collect();
        let return_types = match function.returns() {
            Some(returns) => returns.iter().map(&resolve).collect(),
            None => Vec::new(),
        };
        (parameter_types, return_types)
    }
}

impl<'context> IntoOds<MlirType<'context>> for Type<'context> {
    fn into_ods(self) -> MlirType<'context> {
        self.into_mlir()
    }
}
