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

    /// A signless integer type of `bits` width (`i<bits>`): the boolean `i1`.
    pub fn signless(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::new(context, bits as u32)))
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
        element_type: MlirType<'context>,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreatePointerType(
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

    /// The single-byte `!sol.byte`, the element type of `bytes` / `string`.
    ///
    /// The dialect exposes no byte-type constructor and Rust cannot build `!sol.byte` directly, so it
    /// is taken as the element type of a `!sol.string`.
    pub fn byte(context: &'context melior::Context) -> Self {
        let string_type = Self::string(context, solx_utils::DataLocation::Memory).into_mlir();
        Self::new(unsafe { MlirType::from_raw(ffi::mlirSolGetEltType(string_type.to_raw(), 0)) })
    }

    /// A `sol::ArrayType` of `element_type` at `location`.
    pub fn array(
        context: &'context melior::Context,
        size: ArraySize,
        element_type: MlirType<'context>,
        location: solx_utils::DataLocation,
    ) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateArrayType(
                context.to_raw(),
                i64::from(size),
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
            MlirType::from_raw(ffi::solxCreateMappingType(
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
        let raw_types: Vec<mlir_sys::MlirType> = member_types
            .iter()
            .map(|member_type| member_type.to_raw())
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

    /// A `sol::FuncRefType`: an internal function pointer over `parameter_types -> result_types`.
    pub fn func_ref(
        context: &'context melior::Context,
        parameter_types: &[MlirType<'context>],
        result_types: &[MlirType<'context>],
    ) -> Self {
        let parameters: Vec<_> = parameter_types
            .iter()
            .map(|parameter_type| parameter_type.to_raw())
            .collect();
        let results: Vec<_> = result_types
            .iter()
            .map(|result_type| result_type.to_raw())
            .collect();
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxCreateFuncRefType(
                context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        })
    }

    /// Whether this is a Sol internal function reference (`!sol.func_ref<...>`).
    pub fn is_function_ref(self) -> bool {
        unsafe { ffi::solxIsFuncRefType(self.inner.to_raw()) }
    }

    /// Whether this is the single-byte `!sol.byte`, the element type of `bytes` / `string`.
    ///
    /// The dialect exposes no byte-type query, so the type is reconstructed via [`Self::byte`] and
    /// matched by equality.
    pub fn is_byte(self, context: &'context melior::Context) -> bool {
        self == Self::byte(context)
    }

    /// Whether this is the dynamic-bytes type `!sol.string` (shared by `string` and `bytes`), at any
    /// data location.
    ///
    /// The dialect exposes no string-type query, so the type is reconstructed at each data location
    /// and matched by equality.
    pub fn is_string(self, context: &'context melior::Context) -> bool {
        [
            solx_utils::DataLocation::Storage,
            solx_utils::DataLocation::CallData,
            solx_utils::DataLocation::Memory,
            solx_utils::DataLocation::Stack,
            solx_utils::DataLocation::Immutable,
            solx_utils::DataLocation::Transient,
        ]
        .into_iter()
        .any(|location| self.inner == Self::string(context, location).into_mlir())
    }

    /// The byte width of a fixed-width byte type: `N` for `!sol.fixedbytes<N>`, `1` for the single
    /// `!sol.byte`, and `None` for any other type.
    ///
    /// The dialect exposes no width query, so each `bytes1 ..= bytes32` type is reconstructed and
    /// matched by equality.
    pub fn fixed_bytes_or_byte_width(self, context: &'context melior::Context) -> Option<u32> {
        if self.is_byte(context) {
            return Some(1);
        }
        (1..=solx_utils::BYTE_LENGTH_FIELD as u32)
            .find(|&width| self.inner == Self::fixed_bytes(context, width).into_mlir())
    }

    /// The bit width of an integer type, or 256 for any non-integer type.
    pub fn integer_bit_width(self) -> u32 {
        IntegerType::try_from(self.inner).map_or(solx_utils::BIT_LENGTH_FIELD as u32, |integer| {
            integer.width()
        })
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

impl<'context> IntoOds<MlirType<'context>> for Type<'context> {
    fn into_ods(self) -> MlirType<'context> {
        self.into_mlir()
    }
}
