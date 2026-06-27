//!
//! An MLIR type in the Sol dialect: its construction, predicates, and the casts it routes.
//!

pub mod array_size;
pub mod location_policy;
pub mod resolve_signature;
pub mod resolve_type;

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;
use slang_solidity_v2::ast::Type as SlangType;

use crate::Builder;
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
        builder: &Builder<'context>,
    ) -> Option<MlirType<'context>> {
        Some(Self::resolve(
            &slang_type?,
            LocationPolicy::Declared(None),
            builder,
        ))
    }

    /// Resolves a state variable's declared type (Slang always types one) in its
    /// declared location.
    pub fn resolve_state_variable(
        slang_type: &SlangType,
        builder: &Builder<'context>,
    ) -> MlirType<'context> {
        Self::resolve(slang_type, LocationPolicy::Declared(None), builder)
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
        builder: &Builder<'context>,
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
                builder,
                block,
                EnumCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        // Contract ↔ contract (inheritance up/downcast, interface).
        if source.is_contract() && self.is_contract() {
            return Value::new(mlir_op!(
                builder,
                block,
                ContractCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        // address ↔ {integer, contract, fixedbytes<20>}. `sol.address_cast`
        // requires the integer side to be exactly `ui160`, so a wider/narrower
        // integer bridges through `ui160` (then a plain `sol.cast` resizes it).
        if source.is_address() || self.is_address() {
            let ui160 = Self::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS);
            if source.is_address() {
                if self.is_contract() || self.is_fixed_bytes() || self == ui160 {
                    return self.address_cast(value, builder, block);
                }
                let as_160 = ui160.address_cast(value, builder, block);
                return self.cast(as_160, builder, block);
            }
            if source.is_contract() || source.is_fixed_bytes() || source == ui160 {
                return self.address_cast(value, builder, block);
            }
            let as_160 = ui160.cast(value, builder, block);
            return self.address_cast(as_160, builder, block);
        }
        // Dynamic `bytes`/`string` → `bytesN`: take the leading N bytes via the
        // dedicated op (`sol.bytes_cast` rejects a `!sol.string` operand).
        if source.is_reference() && self.is_fixed_bytes() {
            return Value::new(mlir_op!(
                builder,
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
                let partner = Self::unsigned(builder.context, partner_bits as usize);
                let as_int = partner.bytes_cast(value, builder, block);
                return self.cast(as_int, builder, block);
            }
            return self.bytes_cast(value, builder, block);
        }
        if self.is_fixed_bytes() || self.is_byte() {
            let partner_bits = Self::partner_bits(self);
            if let Ok(integer) = IntegerType::try_from(source.into_mlir())
                && integer.width() != partner_bits
            {
                let partner = Self::unsigned(builder.context, partner_bits as usize);
                let as_int = partner.cast(value, builder, block);
                return self.bytes_cast(as_int, builder, block);
            }
            return self.bytes_cast(value, builder, block);
        }
        // Reference types (array / struct / string / bytes / mapping) differ
        // only by data location; a reference→reference cast routes through
        // `sol.data_loc_cast`.
        if source.is_reference() && self.is_reference() {
            return Value::new(mlir_op!(
                builder,
                block,
                DataLocCastOperation.inp(value.into_mlir()).out(self.inner)
            ));
        }
        Value::new(mlir_op!(
            builder,
            block,
            CastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// Emits a `sol.bytes_cast` casting `value` to this byte / fixed-bytes / integer target.
    fn bytes_cast<'block>(
        self,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Value::new(mlir_op!(
            builder,
            block,
            BytesCastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// Emits a `sol.address_cast` to this (address-side) type.
    fn address_cast<'block>(
        self,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Value::new(mlir_op!(
            builder,
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

impl<'context> IntoOds<MlirType<'context>> for Type<'context> {
    fn into_ods(self) -> MlirType<'context> {
        self.into_mlir()
    }
}
