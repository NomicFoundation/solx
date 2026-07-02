//!
//! An MLIR type in the Sol dialect: its construction, kind predicates, and property queries.
//!

pub mod array_size;
pub mod location_policy;

use std::ffi::c_char;

use melior::ir::Attribute;
use melior::ir::Type as MlirType;
use melior::ir::TypeLike;
use melior::ir::r#type::IntegerType;
use num::BigInt;
use num::bigint::Sign;
use slang_solidity_v2::ast::DataLocation;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Parameters;
use slang_solidity_v2::ast::Type as SlangType;

use crate::Context;
use crate::IntoOds;
use crate::ffi;

use self::array_size::ArraySize;
use self::location_policy::LocationPolicy;

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

    /// Resolves a Slang semantic type to its MLIR Sol-dialect type.
    ///
    /// `policy` picks each reference type's data location: the declared location, or memory forced for
    /// the external ABI representation; the `Struct` arm carries the parent's location into members.
    pub fn resolve(
        slang_type: &SlangType,
        policy: LocationPolicy,
        context: &Context<'context>,
    ) -> MlirType<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.is_signed() {
                    MlirType::from(IntegerType::signed(context.mlir_context, bits))
                } else {
                    MlirType::from(IntegerType::unsigned(context.mlir_context, bits))
                }
            }
            SlangType::Boolean(_) => MlirType::from(IntegerType::new(
                context.mlir_context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => Self::address(context.mlir_context, false).into_mlir(),
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => {
                    Self::address(context.mlir_context, false).into_mlir()
                }
                LiteralKind::Integer { .. } => {
                    let mobile_type = literal_type
                        .mobile_type()
                        .expect("slang validated: integer literal fits in 256 bits");
                    Self::resolve(&mobile_type, policy, context)
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    MlirType::from(IntegerType::unsigned(context.mlir_context, bits))
                }
                LiteralKind::String { .. } => {
                    Self::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir()
                }
                LiteralKind::HexString { bytes } => Self::fixed_bytes(
                    context.mlir_context,
                    bytes.try_into().expect("hex string length fits in u32"),
                )
                .into_mlir(),
                LiteralKind::Rational { .. } => {
                    Self::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
                }
            },
            SlangType::String(string_type) => Self::string(
                context.mlir_context,
                policy.data_location(string_type.location()),
            )
            .into_mlir(),
            SlangType::Bytes(bytes_type) => Self::string(
                context.mlir_context,
                policy.data_location(bytes_type.location()),
            )
            .into_mlir(),
            SlangType::ByteArray(byte_array_type) => {
                Self::fixed_bytes(context.mlir_context, byte_array_type.width()).into_mlir()
            }
            SlangType::Array(array_type) => {
                let element_type = Self::resolve(&array_type.element_type(), policy, context);
                let location = policy.data_location(array_type.location());
                Self::array(
                    context.mlir_context,
                    ArraySize::Dynamic,
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type = Self::resolve(&fixed_array_type.element_type(), policy, context);
                let location = policy.data_location(fixed_array_type.location());
                Self::array(
                    context.mlir_context,
                    ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
                .into_mlir()
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Self::resolve(
                    &mapping_type.key_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    context,
                );
                let value_type = Self::resolve(
                    &mapping_type.value_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    context,
                );
                Self::mapping(context.mlir_context, key_type, value_type).into_mlir()
            }
            SlangType::Struct(struct_type) => {
                let struct_location = policy.data_location(struct_type.location());
                let member_policy = policy.within_struct(struct_location);
                let Definition::Struct(struct_definition) = struct_type.definition() else {
                    unreachable!("Slang StructType always references a Struct definition")
                };
                let mut member_types = Vec::new();
                for member in struct_definition.members().iter() {
                    let member_slang_type = member.get_type().expect("slang validated");
                    member_types.push(Self::resolve(&member_slang_type, member_policy, context));
                }
                Self::structure(context.mlir_context, &member_types, struct_location).into_mlir()
            }
            SlangType::Contract(contract_type) => {
                let Definition::Contract(contract_definition) = contract_type.definition() else {
                    unreachable!("Slang ContractType always references a Contract definition")
                };
                Self::contract(
                    context.mlir_context,
                    contract_definition.name().name().as_str(),
                    contract_definition.is_payable(),
                )
                .into_mlir()
            }
            SlangType::Interface(interface_type) => {
                let Definition::Interface(interface_definition) = interface_type.definition()
                else {
                    unreachable!("Slang InterfaceType always references an Interface definition")
                };
                Self::contract(
                    context.mlir_context,
                    interface_definition.name().name().as_str(),
                    false,
                )
                .into_mlir()
            }
            SlangType::Enum(enum_type) => {
                let Definition::Enum(enum_definition) = enum_type.definition() else {
                    unreachable!("Slang EnumType always references an Enum definition")
                };
                let member_count = enum_definition.members().iter().count();
                let max = u8::try_from(member_count - 1).expect("enum member count fits in u8");
                Self::enumeration(context.mlir_context, max.into()).into_mlir()
            }
            SlangType::UserDefinedValue(user_defined_value_type) => {
                let target_type = user_defined_value_type
                    .target_type()
                    .expect("slang validated");
                Self::resolve(&target_type, policy, context)
            }
            SlangType::Function(function_type) => {
                let (parameter_types, result_types) =
                    Self::function_pointer_signature(slang_type, context);
                if function_type.is_externally_visible() {
                    Self::ext_func_ref(context.mlir_context, &parameter_types, &result_types)
                        .into_mlir()
                } else {
                    Self::func_ref(context.mlir_context, &parameter_types, &result_types)
                        .into_mlir()
                }
            }
            SlangType::FixedPointNumber(fixed_point_type) => {
                let bits = fixed_point_type.bits();
                if fixed_point_type.is_signed() {
                    MlirType::from(IntegerType::signed(context.mlir_context, bits))
                } else {
                    MlirType::from(IntegerType::unsigned(context.mlir_context, bits))
                }
            }
            SlangType::Library(_) => Self::address(context.mlir_context, false).into_mlir(),
            SlangType::Tuple(_) | SlangType::Void(_) => {
                unreachable!("tuple and void are resolved via Self::resolve_result_types")
            }
        }
    }

    /// Resolves a possibly-absent Slang type: the `Option`-lift over [`Self::resolve`].
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

    /// Resolves a state variable's declared type, which Slang always supplies, in its declared location.
    pub fn resolve_state_variable(
        slang_type: &SlangType,
        context: &Context<'context>,
    ) -> MlirType<'context> {
        Self::resolve(slang_type, LocationPolicy::Declared(None), context)
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
                    Self::resolve(element_type, LocationPolicy::Declared(None), context)
                })
                .collect(),
            other => vec![Self::resolve(
                other,
                LocationPolicy::Declared(None),
                context,
            )],
        }
    }

    /// Resolves a function's `(parameter_types, return_types)` from Slang to MLIR under `policy`: the
    /// declared signature, or the external ABI signature that forces reference types to memory.
    pub fn resolve_signature(
        function: &FunctionDefinition,
        policy: LocationPolicy,
        context: &Context<'context>,
    ) -> (Vec<MlirType<'context>>, Vec<MlirType<'context>>) {
        let parameter_types = Self::resolve_parameters(&function.parameters(), policy, context);
        let return_types = match function.returns() {
            Some(returns) => Self::resolve_parameters(&returns, policy, context),
            None => Vec::new(),
        };
        (parameter_types, return_types)
    }

    /// Resolves a parameter list's declared MLIR types from Slang, in declaration order.
    pub fn resolve_parameters(
        parameters: &Parameters,
        policy: LocationPolicy,
        context: &Context<'context>,
    ) -> Vec<MlirType<'context>> {
        parameters
            .iter()
            .map(|parameter| {
                Self::resolve(
                    &parameter.get_type().expect("slang validated"),
                    policy,
                    context,
                )
            })
            .collect()
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
                Self::resolve(parameter_type, LocationPolicy::Declared(None), context)
            })
            .collect();
        let result_types = Self::resolve_result_types(&function_type.return_type(), context);
        (parameter_types, result_types)
    }

    /// The MLIR element type and data location of a dynamic-array / `bytes` base, the `.push` receiver.
    pub fn dynamic_array_element(
        base_type: &SlangType,
        context: &Context<'context>,
    ) -> (MlirType<'context>, solx_utils::DataLocation) {
        let (element_type, slang_location) = match base_type {
            SlangType::Array(array_type) => (
                Self::resolve(
                    &array_type.element_type(),
                    LocationPolicy::Declared(None),
                    context,
                ),
                array_type.location(),
            ),
            SlangType::Bytes(bytes_type) => (
                Self::byte(context.mlir_context).into_mlir(),
                bytes_type.location(),
            ),
            other => unreachable!(
                "Solidity's .push is a member of dynamic arrays and bytes only; got {:?}",
                std::mem::discriminant(other)
            ),
        };
        let location = match slang_location {
            DataLocation::Inherited => {
                unreachable!("slang's binder should not surface Inherited at an array push base")
            }
            other => solx_utils::DataLocation::from_slang(other, None),
        };
        (element_type, location)
    }

    /// An unsigned integer type of `bits` width (`ui<bits>`).
    pub fn unsigned(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::unsigned(context, bits as u32)))
    }

    /// A signless integer type of `bits` width (`i<bits>`): the boolean `i1` and the Yul word `i256`.
    pub fn signless(context: &'context melior::Context, bits: usize) -> Self {
        Self::new(MlirType::from(IntegerType::new(context, bits as u32)))
    }

    /// The opaque LLVM pointer type (`!llvm.ptr`): a Yul-local slot.
    pub fn llvm_ptr(context: &'context melior::Context) -> Self {
        Self::new(melior::dialect::llvm::r#type::pointer(context, 0))
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

    /// The single `sol::ByteType` (the `bytes`/`string` element type).
    pub fn byte(context: &'context melior::Context) -> Self {
        Self::new(unsafe { MlirType::from_raw(ffi::solxCreateByteType(context.to_raw())) })
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

    /// A `sol::ExtFuncRefType`: an external function reference, an address and selector, over `parameter_types -> result_types`.
    pub fn ext_func_ref(
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
            MlirType::from_raw(ffi::solxCreateExtFuncRefType(
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
        unsafe { ffi::solxIsEnumType(self.inner.to_raw()) }
    }

    /// Whether this is the Sol address type (`!sol.address`).
    pub fn is_address(self) -> bool {
        unsafe { ffi::solxIsAddressType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol contract type (`!sol.contract<...>`).
    pub fn is_contract(self) -> bool {
        unsafe { ffi::solxIsContractType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol fixed-bytes type (`!sol.fixedbytes<N>`).
    pub fn is_fixed_bytes(self) -> bool {
        unsafe { ffi::solxIsFixedBytesType(self.inner.to_raw()) }
    }

    /// Whether this is the single-byte `!sol.byte` (distinct from `!sol.fixedbytes<1>`).
    pub fn is_byte(self) -> bool {
        unsafe { ffi::solxIsByteType(self.inner.to_raw()) }
    }

    /// Whether this is the dynamic-bytes type `!sol.string` (shared by `string` and `bytes`).
    pub fn is_string(self) -> bool {
        unsafe { ffi::solxIsStringType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol array type (`!sol.array<...>`).
    pub fn is_array(self) -> bool {
        unsafe { ffi::solxIsArrayType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol struct type (`!sol.struct<...>`).
    pub fn is_struct(self) -> bool {
        unsafe { ffi::solxIsStructType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol mapping type (`!sol.mapping<...>`).
    pub fn is_mapping(self) -> bool {
        unsafe { ffi::solxIsMappingType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol reference type: array, struct, string/`bytes`, or mapping.
    pub fn is_reference(self) -> bool {
        self.is_string() || self.is_array() || self.is_struct() || self.is_mapping()
    }

    /// Whether this is a Sol function reference of either kind, internal or external.
    pub fn is_function_ref(self) -> bool {
        let raw = self.inner.to_raw();
        unsafe { ffi::solxIsFuncRefType(raw) || ffi::solxIsExtFuncRefType(raw) }
    }

    /// Whether this is a Sol external function reference (`!sol.ext_func_ref<...>`).
    pub fn is_ext_function_ref(self) -> bool {
        unsafe { ffi::solxIsExtFuncRefType(self.inner.to_raw()) }
    }

    /// Whether this is a Sol pointer (`!sol.ptr<T, Loc>`): a typed place.
    pub fn is_pointer(self) -> bool {
        unsafe { ffi::solxIsPointerType(self.inner.to_raw()) }
    }

    /// The pointee type `T` of a `!sol.ptr<T, Loc>`; the caller must ensure this is a pointer.
    pub fn pointee(self) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::solxPointerTypePointeeType(self.inner.to_raw()))
        })
    }

    /// The data location of a pointer's `Loc` or a string/array/struct's own
    /// location.
    pub fn data_location(self) -> solx_utils::DataLocation {
        let raw = self.inner.to_raw();
        let ordinal = if self.is_pointer() {
            unsafe { ffi::solxPointerTypeDataLocation(raw) }
        } else {
            unsafe { ffi::solxReferenceTypeDataLocation(raw) }
        };
        solx_utils::DataLocation::try_from(ordinal).unwrap_or_else(|ordinal| {
            unreachable!("unexpected !sol.ptr data-location ordinal {ordinal}")
        })
    }

    /// The element / field type reached by stepping into this aggregate: the index is ignored for non-structs.
    pub fn element_type(self, field_index: usize) -> Self {
        Self::new(unsafe {
            MlirType::from_raw(ffi::mlirSolGetEltType(
                self.inner.to_raw(),
                field_index as u64,
            ))
        })
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
            Some(unsafe { ffi::solxFixedBytesTypeSize(self.inner.to_raw()) })
        } else if self.is_byte() {
            Some(1)
        } else {
            None
        }
    }

    /// Whether this is a signed integer type; false for any non-integer type.
    pub fn is_signed(self) -> bool {
        IntegerType::try_from(self.inner).is_ok_and(|integer| integer.is_signed())
    }

    /// The bit width of an integer type, or 256 for any non-integer type.
    pub fn integer_bit_width(self) -> u32 {
        IntegerType::try_from(self.inner).map_or(solx_utils::BIT_LENGTH_FIELD as u32, |integer| {
            integer.width()
        })
    }

    /// The bit width of the unsigned integer a `sol.bytes_cast` pairs with this fixed-bytes type.
    pub fn fixed_bytes_integer_bits(self) -> u32 {
        self.fixed_bytes_or_byte_width()
            .expect("a fixed-bytes / byte type has a width")
            * solx_utils::BIT_LENGTH_BYTE as u32
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
