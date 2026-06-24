//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Operation;
use melior::ir::Type as MlirType;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::DenseI32ArrayAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationMutLike;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use slang_solidity_v2::ast::DataLocation;
use slang_solidity_v2::ast::Type as SlangType;

use crate::Builder;
use crate::CmpPredicate;
use crate::IntoOds;
use crate::Type;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ConvCastOperation;
use crate::ods::sol::EncodeOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::Keccak256Operation;
use crate::ods::sol::LengthOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::StringLitOperation;

/// An MLIR value in the Sol dialect; the home for the conversions a value undergoes.
#[derive(Clone, Copy)]
pub struct Value<'context, 'block> {
    inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Value<'context, 'block> {
    /// Wraps a melior value.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// Views a `!sol.ptr`-typed value as a [`Pointer`] place — the inverse of
    /// [`Pointer::into_value`]. The caller must ensure this value is a pointer.
    pub fn into_pointer(self) -> crate::Pointer<'context, 'block> {
        crate::Pointer::new(self.inner)
    }

    /// The value's type.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value.
    pub fn constant<B>(
        value: i64,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result_type = result_type.into_mlir();
        Self::new(mlir_op!(
            builder,
            block,
            ConstantOperation
                .value(Attribute::from(IntegerAttribute::new(result_type, value)))
                .result(result_type)
        ))
    }

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`] (literals that overflow `i64`).
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if result_type == Type::address(builder.context, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                builder,
                block,
            );
            return integer.cast(result_type, builder, block);
        }
        let attribute: Attribute<'context> = if result_type.integer_bit_width()
            == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(result_type.into_mlir(), i64::from(*value != BigInt::ZERO)).into()
        } else {
            let (sign, words) = value.to_u64_digits();
            unsafe {
                Attribute::from_raw(crate::ffi::solxCreateIntegerAttr(
                    result_type.into_mlir().to_raw(),
                    sign == num::bigint::Sign::Minus,
                    words.len(),
                    words.as_ptr(),
                ))
            }
        };
        Self::new(mlir_op!(
            builder,
            block,
            ConstantOperation
                .value(attribute)
                .result(result_type.into_mlir())
        ))
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean(
        value: bool,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN),
            builder,
            block,
        )
    }

    /// A `string memory` value holding `text` verbatim, via `sol.string_lit`.
    /// The bytes are taken as-is — a string literal need not be valid UTF-8.
    pub fn string_literal(
        text: &str,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            builder,
            block,
            StringLitOperation
                .value(StringAttribute::new(builder.context, text))
                .addr(Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir())
        ))
    }

    /// The zero of a scalar value type, built at its own representation width and cast through the type.
    pub fn zero(
        r#type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if r#type.is_address() {
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_contract() {
            // address(0) reinterpreted as the contract (solc: ui160 -> address -> contract).
            let address = Self::zero(Type::address(builder.context, false), builder, block);
            address.cast(r#type, builder, block)
        } else if let Some(width) = r#type.fixed_bytes_or_byte_width() {
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, (width * 8) as usize),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_enum() {
            unimplemented!("zero-init of enum type is not yet supported")
        } else if r#type.is_ext_function_ref() {
            unimplemented!("zero-init of external function-pointer type is not yet supported")
        } else if r#type.is_function_ref() {
            unimplemented!("zero-init of function-pointer type is not yet supported")
        } else if IntegerType::try_from(r#type.into_mlir()).is_ok() {
            Self::constant(0, r#type, builder, block)
        } else {
            unreachable!("Value::zero handles only scalar value types")
        }
    }

    /// `sol.malloc` — a fresh memory buffer typed as `mlir_type`. `Some` `size`
    /// allocates a dynamically-sized buffer; `zero_init` zero-fills it.
    pub fn malloc(
        mlir_type: MlirType<'context>,
        size: Option<MlirValue<'context, 'block>>,
        zero_init: bool,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut op_builder =
            MallocOperation::builder(builder.context, builder.unknown_location).addr(mlir_type);
        if let Some(size) = size {
            op_builder = op_builder.size(size);
        }
        if zero_init {
            op_builder = op_builder.zero_init(Attribute::unit(builder.context));
        }
        Self::new(
            block
                .append_operation(op_builder.build().into())
                .result(0)
                .expect("sol.malloc produces one result")
                .into(),
        )
    }

    /// The default value of a return position reached without an explicit `return <value>`, keyed on the Slang type.
    pub fn type_default(
        slang_type: Option<&SlangType>,
        mlir_type: MlirType<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let is_memory = |location| matches!(location, DataLocation::Memory);
        match slang_type {
            Some(SlangType::FixedSizeArray(array)) if is_memory(array.location()) => {
                Self::malloc(mlir_type, None, true, builder, block)
            }
            Some(SlangType::Struct(structure)) if is_memory(structure.location()) => {
                Self::malloc(mlir_type, None, true, builder, block)
            }
            Some(SlangType::Array(array)) if is_memory(array.location()) => {
                Self::malloc(mlir_type, None, true, builder, block)
            }
            Some(SlangType::String(_) | SlangType::Bytes(_)) => {
                // A fresh zero-length buffer (plain `sol.malloc`), not a sized `new bytes(0)`.
                Self::malloc(mlir_type, None, false, builder, block)
            }
            Some(
                SlangType::Address(_)
                | SlangType::ByteArray(_)
                | SlangType::Enum(_)
                | SlangType::UserDefinedValue(_)
                | SlangType::Function(_)
                | SlangType::Contract(_)
                | SlangType::Interface(_),
            ) => Self::zero(Type::new(mlir_type), builder, block),
            _ => Self::constant(0, Type::new(mlir_type), builder, block),
        }
    }

    /// `sol.gasleft` — all remaining gas as a `ui256` (the `gasleft()` built-in and the default forwarded call gas).
    pub fn gas_left<B>(builder: &Builder<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            builder,
            block,
            GasLeftOperation
                .val(Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// A `uint256` constant — Solidity's default integer width.
    pub fn uint256(
        value: i64,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::constant(
            value,
            Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            block,
        )
    }

    /// `sol.keccak256` over a byte buffer, yielding the 32-byte hash.
    pub fn keccak256(
        buffer: Self,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            builder,
            block,
            Keccak256Operation
                .addr(buffer.into_mlir())
                .result(Type::fixed_bytes(builder.context, 32))
        ))
    }

    /// `sol.encode` — the ABI-encoded `bytes memory` payload of `ins`. A `selector`,
    /// when present, is prepended as the leading 4 bytes; `packed` selects `abi.encodePacked`.
    pub fn abi_encode(
        ins: &[MlirValue<'context, 'block>],
        selector: Option<MlirValue<'context, 'block>>,
        packed: bool,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut op_builder = EncodeOperation::builder(builder.context, builder.unknown_location)
            .ins(ins)
            .res(Type::string(builder.context, solx_utils::DataLocation::Memory).into_mlir());
        if let Some(selector_value) = selector {
            op_builder = op_builder.selector(selector_value);
        }
        if packed {
            op_builder = op_builder.packed(Attribute::unit(builder.context));
        }
        let mut operation: Operation = op_builder.build().into();
        // TODO: drop this manual segment-sizes plumbing once the melior op-builder
        // macro emits `operand_segment_sizes` automatically for ops with variadic
        // or optional operand groups.
        let ins_count = i32::try_from(ins.len()).expect("encode argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(
            builder.context,
            &[ins_count, i32::from(selector.is_some())],
        );
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        Self::new(
            block
                .append_operation(operation)
                .result(0)
                .expect("sol.encode always produces one result")
                .into(),
        )
    }

    /// A selector or event topic as a `bytesN` value: `value` at `width_bytes` width cast to `fixedbytes<width_bytes>`.
    pub fn selector_constant(
        value: &BigInt,
        width_bytes: u32,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let integer = Self::constant_from_bigint(
            value,
            Type::unsigned(
                builder.context,
                width_bytes as usize * solx_utils::BIT_LENGTH_BYTE,
            ),
            builder,
            block,
        );
        integer.cast(
            Type::fixed_bytes(builder.context, width_bytes),
            builder,
            block,
        )
    }

    /// Appends a default element to this dynamic array / `bytes` (`sol.push`), returning the element's
    /// place and MLIR type. A reference element yields its reference directly; a value element a `!sol.ptr`.
    pub fn push_slot(
        self,
        base_type: &SlangType,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> (Self, MlirType<'context>) {
        let (element_type, location) = Type::dynamic_array_element(base_type, builder);
        let push_result_type = Type::new(element_type)
            .address_type(location, builder.context)
            .into_mlir();
        let new_slot = Self::new(mlir_op!(
            builder,
            block,
            PushOperation.inp(self.into_mlir()).addr(push_result_type)
        ));
        (new_slot, element_type)
    }

    /// Casts to `target_type` via the target type's cast router ([`Type::cast`]); a no-op when already that type.
    pub fn cast(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        target_type.cast(self, builder, block)
    }

    /// Reinterprets the value's representation as `target_type` via `sol.conv_cast` (e.g. across the
    /// inline-assembly boundary: a `!sol.ptr<T, Stack>` as the `!llvm.ptr` Yul operates on).
    pub fn reinterpret(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            builder,
            block,
            ConvCastOperation
                .inp(self.inner)
                .out(target_type.into_mlir())
        ))
    }

    /// Compares this value against `other` under `predicate` via `sol.cmp`,
    /// producing an `i1`.
    pub fn compare(
        self,
        other: Self,
        predicate: CmpPredicate,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let predicate_attribute = predicate.attribute(builder.context);
        let value: MlirValue<'context, 'block> = mlir_op!(
            builder,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .result(
                    Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir()
                )
        );
        Self::new(value)
    }

    /// The length of this dynamic value (array / `bytes` / `string`) as a `ui256`,
    /// via `sol.length`.
    pub fn length(self, builder: &Builder<'context>, block: &BlockRef<'context, 'block>) -> Self {
        Self::new(mlir_op!(
            builder,
            block,
            LengthOperation
                .inp(self.inner)
                .len(Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD).into_mlir())
        ))
    }

    /// Tests the value against zero, producing an `i1`. Short-circuits when already `i1`.
    pub fn is_nonzero(
        self,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN) {
            return self;
        }
        let zero = Self::constant(0, self.r#type(), builder, block);
        self.compare(zero, CmpPredicate::Ne, builder, block)
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}

impl<'context, 'block> IntoOds<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn into_ods(self) -> MlirValue<'context, 'block> {
        self.into_mlir()
    }
}
