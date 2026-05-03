//!
//! MLIR type factory for Sol dialect emission.
//!
//! Provides pre-cached common types and factory methods for constructing
//! parameterized Sol dialect types (pointers, addresses, integers).
//!

use melior::ir::Type;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;

/// MLIR type factory: pre-cached common types and parameterized constructors.
///
/// All types are constructed through typed APIs — no string parsing.
pub struct TypeFactory<'context> {
    /// The MLIR context for constructing new types.
    context: &'context melior::Context,

    /// 1-bit boolean type (`i1`).
    pub i1: Type<'context>,
    /// Unsigned 160-bit integer type (`ui160`, address width).
    pub ui160: Type<'context>,
    /// Unsigned 256-bit integer type (`ui256`).
    pub ui256: Type<'context>,
    /// Sol address type (`!sol.address`).
    pub sol_address: Type<'context>,
    /// Sol storage pointer type (`!sol.ptr<ui256, Storage>`).
    pub sol_ptr_storage: Type<'context>,
    /// Sol memory string type (`!sol.string<Memory>`).
    pub sol_string_memory: Type<'context>,
}

impl<'context> TypeFactory<'context> {
    /// Bit width of a Solidity function selector (4 bytes).
    pub const SELECTOR_BIT_WIDTH: u32 = solx_utils::BIT_LENGTH_X32 as u32;

    /// Creates a new type factory with pre-cached common types.
    pub fn new(context: &'context melior::Context) -> Self {
        let i1 = Type::from(IntegerType::new(
            context,
            solx_utils::BIT_LENGTH_BOOLEAN as u32,
        ));
        let ui160 = Type::from(IntegerType::unsigned(
            context,
            solx_utils::BIT_LENGTH_ETH_ADDRESS as u32,
        ));
        let ui256 = Type::from(IntegerType::unsigned(
            context,
            solx_utils::BIT_LENGTH_FIELD as u32,
        ));
        // SAFETY: `solxCreateAddressType` returns a valid MlirType from
        // the C++ Sol dialect. The context pointer is valid.
        let sol_address =
            unsafe { Type::from_raw(crate::ffi::solxCreateAddressType(context.to_raw(), false)) };
        // SAFETY: `solxCreatePointerType` returns a valid MlirType from
        // the C++ Sol dialect. The context and element type pointers are valid.
        let sol_ptr_storage = unsafe {
            Type::from_raw(crate::ffi::solxCreatePointerType(
                context.to_raw(),
                ui256.to_raw(),
                solx_utils::DataLocation::Storage as u32,
            ))
        };
        // SAFETY: `solxCreateStringType` returns a valid MlirType from
        // the C++ Sol dialect. The context pointer is valid.
        let sol_string_memory = unsafe {
            Type::from_raw(crate::ffi::solxCreateStringType(
                context.to_raw(),
                solx_utils::DataLocation::Memory as u32,
            ))
        };
        Self {
            context,
            i1,
            ui160,
            ui256,
            sol_address,
            sol_ptr_storage,
            sol_string_memory,
        }
    }

    /// Returns the bit width of an MLIR integer type, or 256 for non-integer types.
    pub fn integer_bit_width(r#type: Type<'_>) -> u32 {
        IntegerType::try_from(r#type).map_or(solx_utils::BIT_LENGTH_FIELD as u32, |integer_type| {
            integer_type.width()
        })
    }

    /// Creates a `sol::AddressType` with the given payability.
    pub fn address(&self, payable: bool) -> Type<'context> {
        // SAFETY: `solxCreateAddressType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer is valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateAddressType(
                self.context.to_raw(),
                payable,
            ))
        }
    }

    /// Creates a `sol::PointerType` with the given element type and data location.
    pub fn pointer(
        &self,
        element_type: Type<'context>,
        location: solx_utils::DataLocation,
    ) -> Type<'context> {
        // SAFETY: `solxCreatePointerType` returns a valid MlirType from the
        // C++ Sol dialect. The context and element type pointers are valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreatePointerType(
                self.context.to_raw(),
                element_type.to_raw(),
                location as u32,
            ))
        }
    }

    /// Creates a `sol::ContractType` for the named contract with the given payability.
    pub fn contract(&self, name: &str, payable: bool) -> Type<'context> {
        let name_bytes = name.as_bytes();
        // SAFETY: `solxCreateContractType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer and the name byte range are
        // valid for the duration of the call.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateContractType(
                self.context.to_raw(),
                name_bytes.as_ptr() as *const std::ffi::c_char,
                name_bytes.len(),
                payable,
            ))
        }
    }

    /// Creates a `sol::StringType` at the given data location.
    pub fn string(&self, location: solx_utils::DataLocation) -> Type<'context> {
        // SAFETY: `solxCreateStringType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer is valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateStringType(
                self.context.to_raw(),
                location as u32,
            ))
        }
    }

    /// Creates a `sol::FixedBytesType` of the given byte width.
    pub fn fixed_bytes(&self, width: u32) -> Type<'context> {
        // SAFETY: `solxCreateFixedBytesType` returns a valid MlirType from
        // the C++ Sol dialect. The context pointer is valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateFixedBytesType(
                self.context.to_raw(),
                width,
            ))
        }
    }

    /// Creates a `sol::ArrayType`. `size = -1` denotes a dynamic array.
    pub fn array(
        &self,
        size: i64,
        element_type: Type<'context>,
        location: solx_utils::DataLocation,
    ) -> Type<'context> {
        // SAFETY: `solxCreateArrayType` returns a valid MlirType from the
        // C++ Sol dialect. The context and element type pointers are valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateArrayType(
                self.context.to_raw(),
                size,
                element_type.to_raw(),
                location as u32,
            ))
        }
    }

    /// Creates a `sol::MappingType` with the given key and value types.
    pub fn mapping(
        &self,
        key_type: Type<'context>,
        value_type: Type<'context>,
    ) -> Type<'context> {
        // SAFETY: `solxCreateMappingType` returns a valid MlirType from the
        // C++ Sol dialect. The context, key, and value type pointers are valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateMappingType(
                self.context.to_raw(),
                key_type.to_raw(),
                value_type.to_raw(),
            ))
        }
    }

    /// Creates a `sol::StructType` from member types and a data location.
    pub fn structure(
        &self,
        member_types: &[Type<'context>],
        location: solx_utils::DataLocation,
    ) -> Type<'context> {
        let raw_types: Vec<mlir_sys::MlirType> =
            member_types.iter().map(|t| t.to_raw()).collect();
        // SAFETY: `solxCreateStructType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer is valid; the member type
        // slice is borrowed for the duration of the call.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateStructType(
                self.context.to_raw(),
                raw_types.as_ptr(),
                raw_types.len(),
                location as u32,
            ))
        }
    }
}
