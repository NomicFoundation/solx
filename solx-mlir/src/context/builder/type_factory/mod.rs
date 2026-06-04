//!
//! MLIR type factory for Sol dialect emission.
//!
//! Provides pre-cached common types and factory methods for constructing
//! parameterized Sol dialect types (pointers, addresses, integers).
//!

pub mod array_size;

use melior::ir::Type;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;

use self::array_size::ArraySize;

/// MLIR type factory: pre-cached common types and parameterized constructors.
///
/// All types are constructed through typed APIs — no string parsing.
pub struct TypeFactory<'context> {
    /// The MLIR context for constructing new types.
    context: &'context melior::Context,

    /// 1-bit boolean type (`i1`).
    pub i1: Type<'context>,
    /// Unsigned 64-bit integer type (`ui64`, struct/array field-index width).
    pub ui64: Type<'context>,
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
    // ---- Sol-dialect type predicates ----
    //
    // Typed `isa<>` introspection via the C-FFI in `sol_attr_stubs.cpp` — no
    // string parsing, so the `AsmPrinter` form can change without silently
    // miscompiling. Centralized here so every caller shares one definition; the
    // category predicates (reference, function-ref, address-like) compose the
    // per-type FFI predicates.

    /// Whether `ty` is a Sol enum type (`!sol.enum<N>`).
    pub fn is_sol_enum(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsEnumType(ty.to_raw()) }
    }

    /// Whether `ty` is the Sol address type (`!sol.address`).
    pub fn is_sol_address(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsAddressType(ty.to_raw()) }
    }

    /// Whether `ty` is a Sol contract type (`!sol.contract<…>`).
    pub fn is_sol_contract(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsContractType(ty.to_raw()) }
    }

    /// Whether `ty` is an address-like type — `!sol.address` or
    /// `!sol.contract<…>` — for which conversions use `sol.address_cast`.
    pub fn is_sol_address_like(ty: Type<'_>) -> bool {
        Self::is_sol_address(ty) || Self::is_sol_contract(ty)
    }

    /// Whether `ty` is a Sol fixed-bytes type (`!sol.fixedbytes<N>`).
    pub fn is_sol_fixed_bytes(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsFixedBytesType(ty.to_raw()) }
    }

    /// The byte width `N` of a `!sol.fixedbytes<N>` type, or `None` for any
    /// other type.
    pub fn fixed_bytes_width(ty: Type<'_>) -> Option<u32> {
        if Self::is_sol_fixed_bytes(ty) {
            Some(unsafe { crate::ffi::solxGetFixedBytesWidth(ty.to_raw()) })
        } else {
            None
        }
    }

    /// Whether `ty` is the single-byte `!sol.byte` — the element type of
    /// `bytes`/`string`, distinct from `!sol.fixedbytes<1>`.
    pub fn is_sol_byte(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsByteType(ty.to_raw()) }
    }

    /// The MLIR element type of an aggregate: the type of struct field `index`,
    /// or — for an array / `bytes` / `string`, whose elements share one type —
    /// the element type (`index` is ignored). The result preserves the
    /// aggregate's data location (a `Storage` aggregate yields `Storage`-located
    /// elements, a `Memory` one yields `Memory`-located elements).
    pub fn element_type<'a>(aggregate_type: Type<'a>, index: u64) -> Type<'a> {
        // SAFETY: `mlirSolGetEltType` returns a valid MlirType from
        // `sol::getEltType` for an in-range index.
        unsafe { Type::from_raw(crate::ffi::mlirSolGetEltType(aggregate_type.to_raw(), index)) }
    }

    /// Whether `ty` is a Sol reference type: array, struct, string/`bytes`, or
    /// mapping. (`bytes` and `string` share `!sol.string`.)
    pub fn is_sol_reference(ty: Type<'_>) -> bool {
        let raw = ty.to_raw();
        unsafe {
            crate::ffi::solxIsStringType(raw)
                || crate::ffi::solxIsArrayType(raw)
                || crate::ffi::solxIsStructType(raw)
                || crate::ffi::solxIsMappingType(raw)
        }
    }

    /// Whether `ty` is a Sol function-pointer type — internal
    /// `!sol.func_ref<…>` or external `!sol.ext_func_ref<…>`.
    pub fn is_sol_function_ref(ty: Type<'_>) -> bool {
        let raw = ty.to_raw();
        unsafe {
            crate::ffi::solxIsFuncRefType(raw) || crate::ffi::solxIsExtFuncRefType(raw)
        }
    }

    /// Whether `ty` is an external function reference (`!sol.ext_func_ref<…>`),
    /// as opposed to an internal `!sol.func_ref<…>`.
    pub fn is_sol_ext_function_ref(ty: Type<'_>) -> bool {
        unsafe { crate::ffi::solxIsExtFuncRefType(ty.to_raw()) }
    }

    /// Bit width of a Solidity function selector (4 bytes).
    pub const SELECTOR_BIT_WIDTH: u32 = solx_utils::BIT_LENGTH_X32 as u32;

    /// Creates a new type factory with pre-cached common types.
    pub fn new(context: &'context melior::Context) -> Self {
        let i1 = Type::from(IntegerType::new(
            context,
            solx_utils::BIT_LENGTH_BOOLEAN as u32,
        ));
        let ui64 = Type::from(IntegerType::unsigned(
            context,
            solx_utils::BIT_LENGTH_X64 as u32,
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
            ui64,
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

    /// Creates a `sol::ArrayType`.
    pub fn array(
        &self,
        size: ArraySize,
        element_type: Type<'context>,
        location: solx_utils::DataLocation,
    ) -> Type<'context> {
        // SAFETY: `solxCreateArrayType` returns a valid MlirType from the
        // C++ Sol dialect. The context and element type pointers are valid.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateArrayType(
                self.context.to_raw(),
                size.as_dialect_i64(),
                element_type.to_raw(),
                location as u32,
            ))
        }
    }

    /// Creates a `sol::MappingType` with the given key and value types.
    pub fn mapping(&self, key_type: Type<'context>, value_type: Type<'context>) -> Type<'context> {
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
        let raw_types: Vec<mlir_sys::MlirType> = member_types.iter().map(|t| t.to_raw()).collect();
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

    /// Creates a `sol::EnumType` whose maximum valid value is `max`
    /// (one less than the number of enum members).
    pub fn enumeration(&self, max: u32) -> Type<'context> {
        // SAFETY: `solxCreateEnumType` returns a valid MlirType from the
        // C++ Sol dialect. The context pointer is valid.
        unsafe { Type::from_raw(crate::ffi::solxCreateEnumType(self.context.to_raw(), max)) }
    }

    /// Creates a `sol::EnumType` sized for an enum with `member_count` members.
    /// Solidity caps enums at 256 members, so the maximum valid enumerator
    /// index (`member_count - 1`) always fits in a `u8`.
    pub fn enumeration_for_member_count(&self, member_count: usize) -> Type<'context> {
        let max = u8::try_from(member_count.saturating_sub(1))
            .expect("enum member count fits in u8");
        self.enumeration(max.into())
    }

    /// Creates a `sol::FuncRefType` for an internal function pointer with the
    /// given parameter and result types.
    pub fn func_ref(
        &self,
        parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
    ) -> Type<'context> {
        let parameters: Vec<_> = parameter_types.iter().map(|t| t.to_raw()).collect();
        let results: Vec<_> = result_types.iter().map(|t| t.to_raw()).collect();
        // SAFETY: `solxCreateFuncRefType` returns a valid MlirType from the
        // C++ Sol dialect. The pointers reference local vectors valid for
        // the duration of the call.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateFuncRefType(
                self.context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        }
    }

    /// Creates a `sol::ExtFuncRefType` for an external function reference
    /// (address + selector) with the given parameter and result types.
    pub fn ext_func_ref(
        &self,
        parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
    ) -> Type<'context> {
        let parameters: Vec<_> = parameter_types.iter().map(|t| t.to_raw()).collect();
        let results: Vec<_> = result_types.iter().map(|t| t.to_raw()).collect();
        // SAFETY: `solxCreateExtFuncRefType` returns a valid MlirType from the
        // C++ Sol dialect. The pointers reference local vectors valid for the
        // duration of the call.
        unsafe {
            Type::from_raw(crate::ffi::solxCreateExtFuncRefType(
                self.context.to_raw(),
                parameters.as_ptr(),
                parameters.len(),
                results.as_ptr(),
                results.len(),
            ))
        }
    }
}
