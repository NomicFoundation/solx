//!
//! Solidity type conversion classification and dispatch.
//!

pub mod location_policy;

pub use self::location_policy::LocationPolicy;
pub mod resolve_signature;
pub mod resolve_type;
pub use self::resolve_signature::ResolveSignature;
pub use self::resolve_type::ResolveType;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::r#type::IntegerType;
use num_bigint::BigInt;
use num_traits::sign::Signed;
use slang_solidity_v2::ast::StateVariableDefinition;
use slang_solidity_v2::ast::Type as SlangType;
use solx_mlir::ods::sol::DefaultFuncConstantOperation;
use solx_mlir::ods::sol::MallocOperation;
use solx_mlir::ods::sol::StoreOperation;

/// Solidity type resolution and default-initialisation.
///
/// A transitional namespace: the cast/coercion this type once classified now
/// lives on [`crate::ast::Value`] (`coerce_to` / `cast`), routed by the target
/// [`crate::ast::Type`]. What remains here — Slang→MLIR type resolution and
/// zero / default-initialisation — moves onto `Type` and `Value` / `Pointer` in
/// the resolution and constants stages.
pub struct TypeConversion;

impl TypeConversion {
    /// `Option`-lifted [`ResolveType::resolve_type`]: maps a possibly-absent
    /// slang type — as returned by `node.get_type()` on a node the binder left
    /// untyped (an unresolved reference or semantic error) — through with a
    /// `None` inherited location, yielding `None` when the slang type is absent.
    // TODO: slang's binder does not fold binary expressions of literal operands —
    // its typing rules return the type of one operand (e.g. type of the left
    // operand for shifts), so `1 << 100` gets typed as ui8 (the type of `1`)
    // and constant subexpressions overflow at that width. solc folds via
    // `RationalNumberType::binaryOperatorResult`, sizing the result to fit the
    // folded value. Either teach slang to fold, or fold here before lowering.
    pub fn resolve_optional_slang_type<'context>(
        slang_type: Option<SlangType>,
        builder: &solx_mlir::Builder<'context>,
    ) -> Option<Type<'context>> {
        Some(slang_type?.resolve_type(LocationPolicy::Declared(None), builder))
    }

    /// Emits the zero value of a scalar value type that is not a plain
    /// integer/bool: `address(0)`, a zero `bytesN`, or an enum's `0` variant
    /// (a UDVT defers to its underlying type). The zero constant is materialised
    /// at the representation's own width and bridged with that type's dedicated
    /// cast — never by narrowing a wider constant, which the `sol.cast` fold
    /// mishandles. Plain integers/bools are zeroed directly by
    /// `Builder::emit_zero_initialized_alloca` and do not reach here.
    pub fn emit_scalar_zero<'context, 'block>(
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
                let zero = builder.emit_sol_constant(
                    0,
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS)
                        .into_mlir(),
                    block,
                );
                crate::ast::Value::from(zero)
                    .cast(crate::ast::Type::new(mlir_type), builder, block)
                    .into_mlir()
            }
            SlangType::ByteArray(byte_array_type) => {
                // `sol.bytes_cast`'s operand must match the fixed-bytes width
                // (`N * 8` bits), so emit the zero at that width directly.
                let bits = byte_array_type.width() * 8;
                let int_type = Type::from(IntegerType::unsigned(builder.context, bits));
                let zero = builder.emit_sol_constant(0, int_type, block);
                crate::ast::Value::from(zero)
                    .cast(crate::ast::Type::new(mlir_type), builder, block)
                    .into_mlir()
            }
            SlangType::Enum(_) => {
                let zero = builder.emit_sol_constant(
                    0,
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD)
                        .into_mlir(),
                    block,
                );
                crate::ast::Value::from(zero)
                    .cast(crate::ast::Type::new(mlir_type), builder, block)
                    .into_mlir()
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
                    let zero_address = builder.emit_sol_constant(
                        0,
                        crate::ast::Type::unsigned(
                            builder.context,
                            solx_utils::BIT_LENGTH_ETH_ADDRESS,
                        )
                        .into_mlir(),
                        block,
                    );
                    let address = crate::ast::Value::from(zero_address)
                        .cast(
                            crate::ast::Type::address(builder.context, false),
                            builder,
                            block,
                        )
                        .into_mlir();
                    builder.emit_sol_ext_func_constant(address, 0, mlir_type, block)
                } else {
                    sol_op!(builder, block, DefaultFuncConstantOperation.addr(mlir_type))
                }
            }
            SlangType::Contract(_) | SlangType::Interface(_) => {
                // A contract/interface reference's zero is `address(0)`
                // reinterpreted as the contract type (solc: `ui160` zero ->
                // `address` -> contract, two `sol.address_cast`s).
                let zero = builder.emit_sol_constant(
                    0,
                    crate::ast::Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS)
                        .into_mlir(),
                    block,
                );
                let address = crate::ast::Value::from(zero)
                    .cast(
                        crate::ast::Type::address(builder.context, false),
                        builder,
                        block,
                    )
                    .into_mlir();
                crate::ast::Value::from(address)
                    .cast(crate::ast::Type::new(mlir_type), builder, block)
                    .into_mlir()
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
    pub fn emit_default_initialized_slot<'context, 'block>(
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

    /// Resolves the declared Solidity type of a state variable to an MLIR type.
    pub fn resolve_state_variable_type<'context>(
        state_variable: &StateVariableDefinition,
        builder: &solx_mlir::Builder<'context>,
    ) -> anyhow::Result<Type<'context>> {
        let slang_type = state_variable
            .get_type()
            .expect("slang types every state variable");
        Ok(slang_type.resolve_type(LocationPolicy::Declared(None), builder))
    }
}
