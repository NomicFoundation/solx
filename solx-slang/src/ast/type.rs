//!
//! An MLIR type produced during emission, and the casts it routes.
//!

use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::r#type::IntegerType;
use solx_mlir::Builder;
use solx_mlir::TypeFactory;
use solx_mlir::ods::sol::BytesCastOperation;
use solx_mlir::ods::sol::CastOperation;
use solx_mlir::ods::sol::ContractCastOperation;
use solx_mlir::ods::sol::DataLocCastOperation;
use solx_mlir::ods::sol::DynBytesToFixedBytesOperation;
use solx_mlir::ods::sol::EnumCastOperation;

use crate::ast::Value;

/// An MLIR type produced during emission.
///
/// A newtype over the melior type that is the home for the cast a value
/// undergoes to *reach* this type. [`Self::cast`] is the one router: keyed on
/// the source and target type kinds, it selects the dialect cast op each pair
/// needs (`sol.cast`, `sol.bytes_cast`, `sol.enum_cast`, `sol.contract_cast`,
/// `sol.address_cast`, `sol.data_loc_cast`). A value hands itself to its target
/// type — [`Value::cast`] / [`Value::coerce_to`] delegate here — so the kind
/// classification lives in exactly one place.
#[derive(Clone, Copy)]
pub struct Type<'context> {
    inner: MlirType<'context>,
}

impl<'context> Type<'context> {
    /// Wraps a melior type.
    pub fn new(inner: MlirType<'context>) -> Self {
        Self { inner }
    }

    /// The inner melior type, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirType<'context> {
        self.inner
    }

    /// Casts `value` to this (target) type, returning it unchanged when it
    /// already has this type.
    ///
    /// `sol.cast` is integer-only — its verifier rejects enum, address,
    /// contract, and fixed-bytes operands/results, each of which has a dedicated
    /// cast op. This is the single place that classifies the source/target kinds
    /// and routes to the right op, so every caller (value transfers, comparisons,
    /// ABI/event encoders, explicit conversions) gets the correct cast without
    /// repeating the dispatch.
    pub fn cast<'block>(
        self,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        let target = self.inner;
        let source = value.r#type();
        if source == target {
            return value;
        }
        // Enum ↔ integer (`sol.enum_cast` accepts the integer-backed enum;
        // narrowing to an enum range-checks and may revert).
        if TypeFactory::is_sol_enum(source) || TypeFactory::is_sol_enum(target) {
            return Value::new(sol_op!(
                builder,
                block,
                EnumCastOperation.inp(value.into_mlir()).out(target)
            ));
        }
        // Contract ↔ contract (inheritance up/downcast, interface).
        if TypeFactory::is_sol_contract(source) && TypeFactory::is_sol_contract(target) {
            return Value::new(sol_op!(
                builder,
                block,
                ContractCastOperation.inp(value.into_mlir()).out(target)
            ));
        }
        // address ↔ {integer, contract, fixedbytes<20>}. `sol.address_cast`
        // requires the integer side to be exactly `ui160`, so a wider/narrower
        // integer bridges through `ui160` (then a plain `sol.cast` resizes it).
        // `sol.address_cast` itself stays a `Builder` primitive (`emit_constant`
        // materialises `address` constants through it); the constants stage will
        // lift that and dissolve the leaf in here with the others.
        if TypeFactory::is_sol_address(source) || TypeFactory::is_sol_address(target) {
            let ui160 = builder.types.ui160;
            if TypeFactory::is_sol_address(source) {
                if TypeFactory::is_sol_contract(target)
                    || TypeFactory::is_sol_fixed_bytes(target)
                    || target == ui160
                {
                    return Value::new(builder.emit_sol_address_cast(
                        value.into_mlir(),
                        target,
                        block,
                    ));
                }
                let as_160 = builder.emit_sol_address_cast(value.into_mlir(), ui160, block);
                return self.cast(Value::new(as_160), builder, block);
            }
            if TypeFactory::is_sol_contract(source)
                || TypeFactory::is_sol_fixed_bytes(source)
                || source == ui160
            {
                return Value::new(builder.emit_sol_address_cast(value.into_mlir(), target, block));
            }
            let as_160 = Self::new(ui160).cast(value, builder, block);
            return Value::new(builder.emit_sol_address_cast(as_160.into_mlir(), target, block));
        }
        // Dynamic `bytes`/`string` → `bytesN`: take the leading N bytes via the
        // dedicated op (`sol.bytes_cast` rejects a `!sol.string` operand).
        if TypeFactory::is_sol_reference(source) && TypeFactory::is_sol_fixed_bytes(target) {
            return Value::new(sol_op!(
                builder,
                block,
                DynBytesToFixedBytesOperation
                    .inp(value.into_mlir())
                    .out(target)
            ));
        }
        // byte / bytesN ↔ {byte, bytesN, integer}. `sol.bytes_cast` connects
        // `fixedbytes<N>` ↔ `ui(N*8)` (and `byte` ↔ `ui8`) and resizes
        // fixedbytes↔fixedbytes / fixedbytes↔byte directly (right-aligned byte
        // padding, NOT integer sign/zero extension). Only an integer counterpart
        // whose width differs from the fixed-bytes partner width must first be
        // resized through that partner integer (e.g. `fixedbytes<1>` → `ui256`
        // via `ui8`); same-width and fixedbytes/byte counterparts stay direct.
        if TypeFactory::is_sol_fixed_bytes(source) || TypeFactory::is_sol_byte(source) {
            let partner_bits = Self::partner_bits(source);
            if let Ok(integer) = IntegerType::try_from(target)
                && integer.width() != partner_bits
            {
                let partner = MlirType::from(IntegerType::unsigned(builder.context, partner_bits));
                let as_int = Self::new(partner).bytes_cast(value, builder, block);
                return self.cast(as_int, builder, block);
            }
            return self.bytes_cast(value, builder, block);
        }
        if TypeFactory::is_sol_fixed_bytes(target) || TypeFactory::is_sol_byte(target) {
            let partner_bits = Self::partner_bits(target);
            if let Ok(integer) = IntegerType::try_from(source)
                && integer.width() != partner_bits
            {
                let partner = MlirType::from(IntegerType::unsigned(builder.context, partner_bits));
                let as_int = Self::new(partner).cast(value, builder, block);
                return self.bytes_cast(as_int, builder, block);
            }
            return self.bytes_cast(value, builder, block);
        }
        // Reference types (array / struct / string / bytes / mapping) differ
        // only by data location; a reference→reference cast routes through
        // `sol.data_loc_cast`.
        if TypeFactory::is_sol_reference(source) && TypeFactory::is_sol_reference(target) {
            return Value::new(sol_op!(
                builder,
                block,
                DataLocCastOperation.inp(value.into_mlir()).out(target)
            ));
        }
        Value::new(sol_op!(
            builder,
            block,
            CastOperation.inp(value.into_mlir()).out(target)
        ))
    }

    /// Emits a `sol.bytes_cast` casting `value` to this byte / fixed-bytes /
    /// integer target — the single construction site the [`Self::cast`] router
    /// reaches for every byte-flavoured pair (directly and through its partner
    /// integer bridge).
    fn bytes_cast<'block>(
        self,
        value: Value<'context, 'block>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Value<'context, 'block>
    where
        'context: 'block,
    {
        Value::new(sol_op!(
            builder,
            block,
            BytesCastOperation.inp(value.into_mlir()).out(self.inner)
        ))
    }

    /// The bit width of the integer a `sol.bytes_cast` pairs with a fixed-bytes
    /// type: `8 * N` for `!sol.fixedbytes<N>`, and 8 for the single `!sol.byte`.
    fn partner_bits(ty: MlirType<'context>) -> u32 {
        TypeFactory::fixed_bytes_or_byte_width(ty).expect("a fixed-bytes / byte type has a width")
            * 8
    }
}

impl<'context> From<MlirType<'context>> for Type<'context> {
    fn from(inner: MlirType<'context>) -> Self {
        Self::new(inner)
    }
}
