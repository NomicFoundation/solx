//!
//! An MLIR type in the Sol dialect: its construction and property queries.
//!

pub mod array_size;

use std::ffi::c_char;

use melior::ir::Attribute;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;
use num::BigInt;
use num::bigint::Sign;

use crate::IntoOds;
use crate::ffi;

use self::array_size::ArraySize;

/// A thin wrapper over a `melior` type handle in the Sol dialect.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Type<'context> {
    /// The wrapped melior type.
    pub inner: MlirType<'context>,
}

impl<'context> Type<'context> {
    /// Bit width of a Solidity function selector: 4 bytes.
    pub const SELECTOR_BIT_WIDTH: u32 = solx_utils::BIT_LENGTH_X32 as u32;

    /// Wraps a melior type.
    pub fn new(inner: MlirType<'context>) -> Self {
        Self { inner }
    }

    /// An unsigned integer type of `bits` width (`ui<bits>`).
    pub fn unsigned(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::unsigned(context, bits as u32)))
    }

    /// A signed integer type of `bits` width (`si<bits>`).
    pub fn signed(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::signed(context, bits as u32)))
    }

    /// A signless integer type of `bits` width (`i<bits>`).
    pub fn signless(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::new(context, bits as u32)))
    }

    /// The boolean type: a signless `i1`.
    pub fn boolean(context: &'context melior::Context) -> Self {
        Self::signless(context, solx_utils::BIT_LENGTH_BOOLEAN)
    }

    /// A `sol::AddressType` with the given payability.
    pub fn address(context: &'context melior::Context, payable: bool) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateAddressType(context.to_raw(), payable))
        })
    }

    /// A `sol::PointerType` with the given element type and data location.
    pub fn pointer(
        context: &'context melior::Context,
        element_type: Self,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreatePointerType(
                context.to_raw(),
                element_type.inner.to_raw(),
                location as u32,
            ))
        })
    }

    /// A `sol::ContractType` for the named contract with the given payability.
    pub fn contract(context: &'context melior::Context, name: &str, payable: bool) -> Self {
        let name_bytes = name.as_bytes();
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateContractType(
                context.to_raw(),
                name_bytes.as_ptr() as *const c_char,
                name_bytes.len(),
                payable,
            ))
        })
    }

    /// A `sol::StringType` at the given data location (`bytes` and `string`
    /// share `!sol.string`).
    pub fn string(context: &'context melior::Context, location: solx_utils::DataLocation) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateStringType(context.to_raw(), location as u32))
        })
    }

    /// A `sol::FixedBytesType` of the given byte width.
    pub fn fixed_bytes(context: &'context melior::Context, width: u32) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateFixedBytesType(context.to_raw(), width))
        })
    }

    /// A `sol::ArrayType` of `element_type` at `location`.
    pub fn array(
        context: &'context melior::Context,
        size: ArraySize,
        element_type: Self,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateArrayType(
                context.to_raw(),
                i64::from(size),
                element_type.inner.to_raw(),
                location as u32,
            ))
        })
    }

    /// A `sol::MappingType` with the given key and value types.
    pub fn mapping(context: &'context melior::Context, key_type: Self, value_type: Self) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateMappingType(
                context.to_raw(),
                key_type.inner.to_raw(),
                value_type.inner.to_raw(),
            ))
        })
    }

    /// A `sol::StructType` from member types and a data location.
    pub fn structure(
        context: &'context melior::Context,
        member_types: &[Self],
        location: solx_utils::DataLocation,
    ) -> Self {
        let raw_types: Vec<mlir_sys::MlirType> = member_types
            .iter()
            .map(|member_type| member_type.inner.to_raw())
            .collect();
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateStructType(
                context.to_raw(),
                raw_types.as_ptr(),
                raw_types.len(),
                location as u32,
            ))
        })
    }

    /// A `sol::EnumType` whose maximum valid value is `max`, one less than the number of enum members.
    pub fn enumeration(context: &'context melior::Context, max: u32) -> Self {
        Self::new(unsafe { MlirType::from_raw(ffi::solxCreateEnumType(context.to_raw(), max)) })
    }

    /// Whether this is an integer type.
    pub fn is_integer(self) -> bool {
        IntegerType::try_from(self.inner).is_ok()
    }

    /// The bit width of this integer type. Panics on a non-integer type, so a caller must first
    /// establish the type is an integer through [`Self::is_integer`].
    pub fn integer_bit_width(self) -> u32 {
        IntegerType::try_from(self.inner)
            .expect("integer_bit_width called on a non-integer type")
            .width()
    }

    /// The integer attribute of `value` at this type, built via the big-integer FFI constructor for
    /// values wider than `i64`.
    pub fn big_integer_attribute(self, value: &BigInt) -> Attribute<'context> {
        let (sign, words) = value.to_u64_digits();
        unsafe {
            Attribute::from_raw(ffi::solxCreateIntegerAttr(
                self.inner.to_raw(),
                sign == Sign::Minus,
                words.len(),
                words.as_ptr(),
            ))
        }
    }

    /// The element type of this non-mapping reference type, derived C-side by
    /// `mlirSolGetEltType`. For struct types, `index` selects the member.
    pub fn element_type(self, index: u64) -> Self {
        Self::new(unsafe { MlirType::from_raw(ffi::mlirSolGetEltType(self.inner.to_raw(), index)) })
    }

    /// The place type a `sol.gep` step yields, given this `!sol.ptr` base type and an
    /// `element_type`, derived C-side by `sol::GepOp::getResultType`.
    pub fn gep_result_type(self, element_type: Self) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::mlirSolGepGetResultType(
                self.inner.to_raw(),
                element_type.inner.to_raw(),
            ))
        })
    }

    /// The inner melior type, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirType<'context> {
        self.inner
    }
}

impl<'context> std::fmt::Display for Type<'context> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl<'context> IntoOds<MlirType<'context>> for Type<'context> {
    fn into_ods(self) -> MlirType<'context> {
        self.into_mlir()
    }
}
