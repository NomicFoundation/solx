//!
//! Solidity type conversion classification and dispatch.
//!

pub mod location_policy;

pub use self::location_policy::LocationPolicy;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_traits::sign::Signed;
use slang_solidity_v2::ast::Definition;
use slang_solidity_v2::ast::FunctionDefinition;
use slang_solidity_v2::ast::LiteralKind;
use slang_solidity_v2::ast::Parameter;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::DefaultFuncConstantOperation;
use solx_mlir::ods::sol::MallocOperation;
use solx_mlir::ods::sol::StoreOperation;

use crate::ast::contract::ContractEmitter;

/// Classification of Solidity type conversions.
///
/// Used for both explicit conversions (`uint256(x)`, `address(x)`, `bool(x)`)
/// and implicit operand widening in arithmetic, assignment, and comparison.
pub enum TypeConversion<'context> {
    /// `bool(x)` — comparison against zero, not bit-truncation.
    Bool,
    /// `address(x)` / `payable(x)` — `sol.cast` to ui160 then `sol.address_cast`.
    Address,
    /// Integer type cast — `sol.cast` to target.
    Cast(Type<'context>),
}

impl<'context> TypeConversion<'context> {
    /// Maps a Slang semantic type to an MLIR type.
    ///
    /// `policy` picks each reference type's data location: [`LocationPolicy::Declared`]
    /// uses the type's own Slang location (with an inherited fallback for
    /// struct-field-relative `Inherited`), [`LocationPolicy::ForceMemory`] forces
    /// the external (ABI) representation where `calldata` cannot cross the call
    /// boundary. Top-level callers pass `LocationPolicy::Declared(None)`; the
    /// `Struct` arm carries the parent struct's location into member resolution.
    pub fn resolve_slang_type(
        slang_type: &SlangType,
        policy: LocationPolicy,
        builder: &solx_mlir::Builder<'context>,
    ) -> Type<'context> {
        match slang_type {
            SlangType::Integer(integer_type) => {
                let bits = integer_type.bits();
                if integer_type.is_signed() {
                    Type::from(IntegerType::signed(builder.context, bits))
                } else {
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
            }
            SlangType::Boolean(_) => Type::from(IntegerType::new(
                builder.context,
                solx_utils::BIT_LENGTH_BOOLEAN as u32,
            )),
            SlangType::Address(_) => builder.types.sol_address,
            SlangType::Literal(literal_type) => match literal_type.kind() {
                LiteralKind::Address { .. } => builder.types.sol_address,
                LiteralKind::Integer { value } => {
                    let bits = Self::integer_bits_required(&value) as usize;
                    let bits = bits
                        .next_multiple_of(solx_utils::BIT_LENGTH_BYTE)
                        .max(solx_utils::BIT_LENGTH_BYTE);
                    let bits = u32::try_from(bits).expect("bit size fits in 32 bits");
                    if value.is_negative() {
                        Type::from(IntegerType::signed(builder.context, bits))
                    } else {
                        Type::from(IntegerType::unsigned(builder.context, bits))
                    }
                }
                LiteralKind::HexInteger { bytes, .. } => {
                    let bits = bytes * solx_utils::BIT_LENGTH_BYTE as u32;
                    Type::from(IntegerType::unsigned(builder.context, bits))
                }
                LiteralKind::String { .. } => {
                    builder.types.string(solx_utils::DataLocation::Memory)
                }
                LiteralKind::HexString { bytes } => builder
                    .types
                    .fixed_bytes(bytes.try_into().expect("hex string length fits in u32")),
                LiteralKind::Rational { .. } => {
                    // Sentinel: a rational appears only as a compile-time
                    // intermediate that constant folding consumes (see the folding
                    // gate in `ExpressionContext::emit`); a rational that survived
                    // to runtime would fail downstream, not at type resolution.
                    builder.types.ui256
                }
            },
            SlangType::String(string_type) => builder
                .types
                .string(policy.data_location(string_type.location())),
            SlangType::Bytes(bytes_type) => builder
                .types
                .string(policy.data_location(bytes_type.location())),
            SlangType::ByteArray(byte_array_type) => {
                builder.types.fixed_bytes(byte_array_type.width())
            }
            SlangType::Array(array_type) => {
                let element_type =
                    Self::resolve_slang_type(&array_type.element_type(), policy, builder);
                let location = policy.data_location(array_type.location());
                builder
                    .types
                    .array(solx_mlir::ArraySize::Dynamic, element_type, location)
            }
            SlangType::FixedSizeArray(fixed_array_type) => {
                let element_type =
                    Self::resolve_slang_type(&fixed_array_type.element_type(), policy, builder);
                let location = policy.data_location(fixed_array_type.location());
                builder.types.array(
                    solx_mlir::ArraySize::Fixed(fixed_array_type.size() as u64),
                    element_type,
                    location,
                )
            }
            SlangType::Mapping(mapping_type) => {
                let key_type = Self::resolve_slang_type(
                    &mapping_type.key_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    builder,
                );
                let value_type = Self::resolve_slang_type(
                    &mapping_type.value_type(),
                    LocationPolicy::Declared(Some(solx_utils::DataLocation::Storage)),
                    builder,
                );
                builder.types.mapping(key_type, value_type)
            }
            SlangType::Struct(struct_type) => {
                let struct_location = policy.data_location(struct_type.location());
                let member_policy = policy.within_struct(struct_location);
                let struct_definition = match struct_type.definition() {
                    Definition::Struct(definition) => definition,
                    _ => unreachable!("Slang StructType always references a Struct definition"),
                };
                // TODO(v2): move struct-member type resolution into Slang itself
                // so consumers don't have to walk `members()` and propagate the
                // struct's data location by hand.
                let mut member_types = Vec::new();
                for member in struct_definition.members().iter() {
                    let member_slang_type = member
                        .get_type()
                        .expect("struct member type resolved by semantic analysis");
                    member_types.push(Self::resolve_slang_type(
                        &member_slang_type,
                        member_policy,
                        builder,
                    ));
                }
                builder.types.structure(&member_types, struct_location)
            }
            SlangType::Contract(contract_type) => {
                let contract_definition = match contract_type.definition() {
                    Definition::Contract(definition) => definition,
                    _ => unreachable!("Slang ContractType always references a Contract definition"),
                };
                builder.types.contract(
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
                // Interfaces are never `payable` themselves; payability lives
                // on the address-cast at the call site.
                builder
                    .types
                    .contract(interface_definition.name().name().as_str(), false)
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
                builder.types.enumeration(max.into())
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::resolve_slang_type(&target_type, policy, builder)
            }
            SlangType::Function(function_type) => {
                // A function pointer lowers to `!sol.func_ref<fnTy>` (internal)
                // or `!sol.ext_func_ref<fnTy>` (external — address + selector).
                // A void return contributes zero result types; a tuple return
                // expands to one result per element.
                let parameter_types: Vec<_> = function_type
                    .parameter_types()
                    .iter()
                    .map(|parameter_type| {
                        Self::resolve_slang_type(
                            parameter_type,
                            LocationPolicy::Declared(None),
                            builder,
                        )
                    })
                    .collect();
                let result_types: Vec<_> = match function_type.return_type() {
                    SlangType::Void(_) => Vec::new(),
                    SlangType::Tuple(tuple_type) => tuple_type
                        .types()
                        .iter()
                        .map(|element_type| {
                            Self::resolve_slang_type(
                                element_type,
                                LocationPolicy::Declared(None),
                                builder,
                            )
                        })
                        .collect(),
                    other => {
                        vec![Self::resolve_slang_type(
                            &other,
                            LocationPolicy::Declared(None),
                            builder,
                        )]
                    }
                };
                if function_type.is_externally_visible() {
                    builder.types.ext_func_ref(&parameter_types, &result_types)
                } else {
                    builder.types.func_ref(&parameter_types, &result_types)
                }
            }
            _ => unimplemented!("unsupported Slang type"),
        }
    }

    /// `Option`-lifted [`Self::resolve_slang_type`]: maps a possibly-absent
    /// slang type — as returned by `node.get_type()` on a node the binder left
    /// untyped (an unresolved reference or semantic error) — through with a
    /// `None` inherited location, yielding `None` when the slang type is absent.
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before lowering.
    pub fn resolve_optional_slang_type(
        slang_type: Option<SlangType>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Type<'context>> {
        Some(Self::resolve_slang_type(
            &slang_type?,
            LocationPolicy::Declared(None),
            builder,
        ))
    }

    /// Emits the zero value of a scalar value type that is not a plain
    /// integer/bool: `address(0)`, a zero `bytesN`, or an enum's `0` variant
    /// (a UDVT defers to its underlying type). The zero constant is materialised
    /// at the representation's own width and bridged with that type's dedicated
    /// cast — never by narrowing a wider constant, which the `sol.cast` fold
    /// mishandles. Plain integers/bools are zeroed directly by
    /// `Builder::emit_zero_initialized_alloca` and do not reach here.
    pub fn emit_scalar_zero<'block>(
        slang_type: &SlangType,
        mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        match slang_type {
            SlangType::Integer(_) | SlangType::Boolean(_) => {
                builder.emit_sol_constant(0, mlir_type, block)
            }
            SlangType::Address(_) => {
                // `sol.address_cast`'s operand is the 160-bit address width;
                // emit the zero at that width directly (no constant narrowing).
                let zero = builder.emit_sol_constant(0, builder.types.ui160, block);
                builder.emit_sol_address_cast(zero, mlir_type, block)
            }
            SlangType::ByteArray(byte_array_type) => {
                // `sol.bytes_cast`'s operand must match the fixed-bytes width
                // (`N * 8` bits), so emit the zero at that width directly.
                let bits = byte_array_type.width() * 8;
                let int_type = Type::from(IntegerType::unsigned(builder.context, bits));
                let zero = builder.emit_sol_constant(0, int_type, block);
                builder.emit_sol_bytes_cast(zero, mlir_type, block)
            }
            SlangType::Enum(_) => {
                let zero = builder.emit_sol_constant(0, builder.types.ui256, block);
                builder.emit_sol_enum_cast(zero, mlir_type, block)
            }
            SlangType::UserDefinedValue(udvt) => {
                let target_type = udvt
                    .target_type()
                    .expect("UDVT target type resolved by semantic analysis");
                Self::emit_scalar_zero(&target_type, mlir_type, builder, block)
            }
            SlangType::Function(function_type) => {
                // The zero value of an external function pointer is a zero
                // address + zero selector packed into an `!sol.ext_func_ref`; of
                // an internal one, the dialect's `default_func_constant` (a
                // pointer that reverts when called).
                if function_type.is_externally_visible() {
                    let zero_address = builder.emit_sol_constant(0, builder.types.ui160, block);
                    let address = builder.emit_sol_address_cast(
                        zero_address,
                        builder.types.sol_address,
                        block,
                    );
                    builder.emit_sol_ext_func_constant(address, 0, mlir_type, block)
                } else {
                    sol_op!(builder, block, DefaultFuncConstantOperation.addr(mlir_type))
                }
            }
            SlangType::Contract(_) | SlangType::Interface(_) => {
                // A contract/interface reference's zero is `address(0)`
                // reinterpreted as the contract type (solc: `ui160` zero ->
                // `address` -> contract, two `sol.address_cast`s).
                let zero = builder.emit_sol_constant(0, builder.types.ui160, block);
                let address = builder.emit_sol_address_cast(zero, builder.types.sol_address, block);
                builder.emit_sol_address_cast(address, mlir_type, block)
            }
            _ => unreachable!(
                "emit_scalar_zero handles only address/bytesN/enum/integer/bool/UDVT/function/contract value types"
            ),
        }
    }

    /// Allocates a stack slot for a value of `slang_type` (lowered to
    /// `mlir_type`) and default-initialises it to the type's zero, mirroring
    /// solc's `print-init` emission:
    /// - a **memory aggregate** (fixed array, struct, or dynamic array) points
    ///   at a fresh zero-filled allocation (`sol.malloc zero_init`);
    /// - an empty **`string` / `bytes`** is a plain `sol.malloc` of a
    ///   zero-length buffer — never a *sized* allocation, which advances the
    ///   free pointer and misplaces a buffer inline assembly writes past its
    ///   length;
    /// - a **non-integer scalar value type** (address, `bytesN`, enum, a UDVT
    ///   over one, a function pointer, a contract/interface ref) gets its
    ///   representation's own zero ([`Self::emit_scalar_zero`]);
    /// - an **integer/bool** gets a zeroed slot;
    /// - **anything else** is a reference (a `storage`/`calldata` aggregate, a
    ///   mapping, or a `storage` named return) the body binds before reading, so
    ///   a bare slot suffices.
    ///
    /// The single default-initialisation primitive shared by local variable
    /// declarations and named return slots.
    pub fn emit_default_initialized_slot<'block>(
        slang_type: Option<&SlangType>,
        mlir_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block> {
        let pointer = builder.emit_sol_alloca(mlir_type, block);
        // A memory aggregate is malloc-backed; a `storage` reference (e.g.
        // `returns (S storage)`) is a slot pointer assigned in the body, so the
        // `Memory` guard keeps it a bare slot.
        let aggregate_location = match slang_type {
            Some(SlangType::FixedSizeArray(array)) => Some(array.location()),
            Some(SlangType::Struct(struct_type)) => Some(struct_type.location()),
            Some(SlangType::Array(array_type)) => Some(array_type.location()),
            _ => None,
        };
        if matches!(
            aggregate_location,
            Some(slang_solidity_v2::ast::DataLocation::Memory)
        ) {
            let zero = sol_op!(
                builder,
                block,
                MallocOperation
                    .addr(mlir_type)
                    .zero_init(Attribute::unit(builder.context))
            );
            sol_op_void!(builder, block, StoreOperation.val(zero).addr(pointer));
        } else if matches!(slang_type, Some(SlangType::String(_) | SlangType::Bytes(_))) {
            let zero = sol_op!(builder, block, MallocOperation.addr(mlir_type));
            sol_op_void!(builder, block, StoreOperation.val(zero).addr(pointer));
        } else if let Some(
            scalar_value_type @ (SlangType::Address(_)
            | SlangType::ByteArray(_)
            | SlangType::Enum(_)
            | SlangType::UserDefinedValue(_)
            | SlangType::Function(_)
            | SlangType::Contract(_)
            | SlangType::Interface(_)),
        ) = slang_type
        {
            let zero = Self::emit_scalar_zero(scalar_value_type, mlir_type, builder, block);
            sol_op_void!(builder, block, StoreOperation.val(zero).addr(pointer));
        } else if IntegerType::try_from(mlir_type).is_ok() {
            let zero = builder.emit_sol_constant(0, mlir_type, block);
            sol_op_void!(builder, block, StoreOperation.val(zero).addr(pointer));
        }
        pointer
    }

    // TODO: Remove when nomicFoundation/slang#1793 is merged and we can instead
    // depend on `LiteralType::mobile_type()` for literal type conversion.
    fn integer_bits_required(value: &BigInt) -> u32 {
        if value.is_negative() {
            let magnitude_minus_one = -value - 1u32;
            u32::try_from(magnitude_minus_one.bits())
                .expect("literal magnitude bit count fits in u32")
                + 1
        } else {
            u32::try_from(value.bits())
                .expect("literal bit count fits in u32")
                .max(1)
        }
    }

    /// Coerces `value` to `target_type`, emitting the conversion (nothing when
    /// the types already match). The single entry callers use for implicit
    /// widening and explicit conversions.
    pub fn coerce<'block>(
        value: Value<'context, 'block>,
        target_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Self::from_target_type(target_type, builder).emit(value, builder, block)
    }

    /// Classifies a target type into the appropriate conversion variant.
    pub fn from_target_type(
        target_type: Type<'context>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Self {
        if target_type == builder.types.i1 {
            Self::Bool
        } else if target_type == builder.types.sol_address {
            Self::Address
        } else {
            Self::Cast(target_type)
        }
    }

    /// Returns the MLIR target type this conversion produces.
    pub fn to_target_type(&self, builder: &solx_mlir::Builder<'context>) -> Type<'context> {
        match self {
            Self::Bool => builder.types.i1,
            Self::Address => builder.types.sol_address,
            Self::Cast(target_type) => *target_type,
        }
    }

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let slang_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        Ok(Self::resolve_slang_type(
            &slang_type,
            LocationPolicy::Declared(None),
            builder,
        ))
    }

    /// Resolves a function's parameter and return types from Slang AST to MLIR.
    ///
    /// `policy` is the data-location policy for the resolved types:
    /// [`LocationPolicy::Declared`] for the DECLARED signature (used inside the
    /// callee's own body), [`LocationPolicy::ForceMemory`] for the EXTERNAL (ABI)
    /// signature — an external call ABI-encodes its arguments and decodes its
    /// results into memory (`calldata` cannot cross the call boundary), so solc
    /// shows a `bytes calldata` parameter as `!sol.string<Memory>` in the call's
    /// `callee_type`.
    pub fn resolve_function_types(
        function: &FunctionDefinition,
        policy: LocationPolicy,
        builder: &solx_mlir::Builder<'context>,
    ) -> (Vec<Type<'context>>, Vec<Type<'context>>) {
        let resolve = |parameter: Parameter| {
            Self::resolve_slang_type(
                &parameter
                    .get_type()
                    .expect("parameter type resolved by semantic analysis"),
                policy,
                builder,
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
    pub fn emit<'block>(
        self,
        value: melior::ir::Value<'context, 'block>,
        builder: &solx_mlir::Builder<'context>,
        block: &melior::ir::BlockRef<'context, 'block>,
    ) -> melior::ir::Value<'context, 'block>
    where
        'context: 'block,
    {
        if value.r#type() == self.to_target_type(builder) {
            return value;
        }
        match self {
            Self::Bool => {
                let zero = builder.emit_sol_constant(0, value.r#type(), block);
                crate::ast::Value::from(value)
                    .compare(
                        crate::ast::Value::from(zero),
                        solx_mlir::CmpPredicate::Ne,
                        builder,
                        block,
                    )
                    .into_mlir()
            }
            Self::Address => {
                let address_type = builder.types.sol_address;
                let truncated = if melior::ir::r#type::IntegerType::try_from(value.r#type()).is_ok()
                {
                    let ui160 = builder.types.ui160;
                    builder.emit_sol_cast(value, ui160, block)
                } else {
                    value
                };
                builder.emit_sol_address_cast(truncated, address_type, block)
            }
            Self::Cast(target_type) => builder.emit_sol_cast(value, target_type, block),
        }
    }
}
