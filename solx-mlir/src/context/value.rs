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
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::operation::OperationMutLike;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use slang_solidity_v2::ast::DataLocation;
use slang_solidity_v2::ast::Type as SlangType;
use solx_utils::BIT_LENGTH_X64;

use crate::Builder;
use crate::CmpPredicate;
use crate::IntoOds;
use crate::Type;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ConvCastOperation;
use crate::ods::sol::DefaultFuncConstantOperation;
use crate::ods::sol::EncodeOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::Keccak256Operation;
use crate::ods::sol::LibAddrOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::NewOperation;
use crate::ods::sol::PushOperation;

/// An MLIR value in the Sol dialect.
///
/// A newtype over the melior value — which already carries its own MLIR type, so
/// the entity stays thin — that is the home for the conversions a value undergoes:
/// coercion, casting, and the truthiness test live on the value itself, not on a
/// context or a type-conversion god-object. The conversion methods take the
/// [`Builder`] and the current block by parameter, exactly as a node's emission
/// does.
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

    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value —
    /// the common path for sizes, indices, and selectors whose magnitude is known
    /// to fit. Generic over the block because it is also emitted from inside the
    /// `Builder`'s own composite primitives, which carry a generic block.
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

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`] — the path
    /// for literals that overflow `i64`. An `address` constant is built at the
    /// `ui160` address width and cast through the router; a boolean is an `i1`
    /// attribute; every other integer width is carried by the FFI big-integer
    /// attribute (the one primitive a `BigInt` constant cannot express through the
    /// fixed-width `IntegerAttribute`).
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

    /// The zero of a scalar value type, built at its own representation width and
    /// bridged through the type's cast — never a narrowed wider constant (the
    /// `sol.cast` fold mishandles that). A UDVT arrives as its underlying type; a
    /// reference type is default-initialised through [`crate::Pointer`] instead.
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
            let bits = Self::constant(
                0,
                Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
                builder,
                block,
            );
            bits.cast(r#type, builder, block)
        } else if r#type.is_ext_function_ref() {
            // A zero address + zero selector packed into the ext func ref.
            let address = Self::zero(Type::address(builder.context, false), builder, block);
            Self::ext_func_constant(address, 0, r#type, builder, block)
        } else if r#type.is_function_ref() {
            // An internal pointer's zero reverts when called.
            Self::new(mlir_op!(
                builder,
                block,
                DefaultFuncConstantOperation.addr(r#type.into_mlir())
            ))
        } else if IntegerType::try_from(r#type.into_mlir()).is_ok() {
            Self::constant(0, r#type, builder, block)
        } else {
            unreachable!("Value::zero handles only scalar value types")
        }
    }

    /// The default value of a return position reached without an explicit
    /// `return <value>` — a zeroed buffer for a memory aggregate, an empty buffer
    /// for dynamic `string` / `bytes`, the representation's own zero otherwise.
    /// The kind is keyed on the Slang type; the scalar fallbacks reuse
    /// [`Self::zero`] / [`Self::constant`].
    pub fn type_default(
        slang_type: Option<&SlangType>,
        mlir_type: MlirType<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let is_memory = |location| matches!(location, DataLocation::Memory);
        match slang_type {
            Some(SlangType::FixedSizeArray(array)) if is_memory(array.location()) => {
                Self::new(mlir_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(mlir_type)
                        .zero_init(Attribute::unit(builder.context))
                ))
            }
            Some(SlangType::Struct(structure)) if is_memory(structure.location()) => {
                Self::new(mlir_op!(
                    builder,
                    block,
                    MallocOperation
                        .addr(mlir_type)
                        .zero_init(Attribute::unit(builder.context))
                ))
            }
            Some(SlangType::Array(array)) if is_memory(array.location()) => Self::new(mlir_op!(
                builder,
                block,
                MallocOperation
                    .addr(mlir_type)
                    .zero_init(Attribute::unit(builder.context))
            )),
            Some(SlangType::String(_) | SlangType::Bytes(_)) => {
                // A fresh zero-length buffer (plain `sol.malloc`, matching solc),
                // not a sized `new bytes(0)`.
                Self::new(mlir_op!(builder, block, MallocOperation.addr(mlir_type)))
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

    /// `sol.gasleft` — all remaining gas as a `ui256`. The default gas an
    /// external call forwards without an explicit `{gas: …}`, the gas of a bare
    /// low-level call, and the value of the `gasleft()` built-in: one home for
    /// the op rather than a Builder method and a built-in arm. Generic over the
    /// block because the call ops emit it from inside composite primitives.
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

    /// The field-width unsigned zero (`ui256` `0`) — the value a call forwards
    /// when it carries no explicit `{value: …}` option. One home for the constant
    /// rather than re-materialising the same `(0, ui256)` pair at each call site.
    pub fn field_zero(builder: &Builder<'context>, block: &BlockRef<'context, 'block>) -> Self {
        Self::constant(
            0,
            Type::unsigned(builder.context, solx_utils::BIT_LENGTH_FIELD),
            builder,
            block,
        )
    }

    /// `sol.ext_func_constant` packing a callee `address` and a 4-byte
    /// `selector` into an `!sol.ext_func_ref<…>` of `result_type` — the callee
    /// value of an external call, and the zero of an external function type.
    pub fn ext_func_constant<B>(
        address: Self,
        selector: u32,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            builder,
            block,
            ExtFuncConstantOperation
                .addr(address.inner)
                .selector(IntegerAttribute::new(
                    IntegerType::new(builder.context, Type::SELECTOR_BIT_WIDTH).into(),
                    selector as i64,
                ))
                .result(result_type.into_mlir())
        ))
    }

    /// The `!sol.ext_func_ref<…>` callee of an external interaction: `receiver`
    /// cast to `address`, packed with `selector` over the signature
    /// `(parameter_types) -> (return_types)`. Shared by CALL / STATICCALL, the
    /// `try`-call, and a `this.f` / `instance.f` external function-pointer value.
    pub fn external_callee(
        receiver: Self,
        selector: u32,
        parameter_types: &[MlirType<'context>],
        return_types: &[MlirType<'context>],
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let address = receiver.cast(Type::address(builder.context, false), builder, block);
        let ext_func_ref_type = Type::ext_func_ref(builder.context, parameter_types, return_types);
        Self::ext_func_constant(address, selector, ext_func_ref_type, builder, block)
    }

    /// `sol.func_constant` — an internal function pointer (`!sol.func_ref<…>`)
    /// to the symbol `name`, the value a bare internal-function reference lowers
    /// to. The null-pointer sibling is [`Self::zero`]'s function-ref arm.
    pub fn function_constant(
        name: &str,
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            builder,
            block,
            FuncConstantOperation
                .addr(result_type.into_mlir())
                .sym(FlatSymbolRefAttribute::new(builder.context, name))
        ))
    }

    /// `sol.lib_addr` — a library's linked deploy address: a placeholder the LLVM
    /// linker resolves by the library's [`solx_utils::ContractName`] full path
    /// (`<file>:<Library>`), the one key it shares with
    /// [`solx_utils::Libraries::as_linker_symbols`]. The value of `address(L)` and
    /// the callee address of an external library call.
    pub fn library_address(
        name: &solx_utils::ContractName,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            builder,
            block,
            LibAddrOperation
                ._name(StringAttribute::new(builder.context, &name.full_path))
                .val(Type::address(builder.context, false).into_mlir())
        ))
    }

    /// `sol.new` — contract creation embedding `obj_name`'s deploy bytecode,
    /// yielding the new instance. `val` is the forwarded wei; a `salt` selects
    /// CREATE2 over CREATE. The operands are appended in ODS order (val, salt,
    /// ctorArgs) and `operand_segment_sizes` is set by hand because melior's ODS
    /// builder does not synthesize it for this `AttrSizedOperandSegments` op, so
    /// the verifier rejects the op without it. The optional salt must be appended
    /// before the variadic ctor args — appending it after would transpose the salt
    /// and the first constructor argument.
    pub fn create_contract(
        obj_name: &str,
        val: Self,
        salt: Option<Self>,
        ctor_args: &[MlirValue<'context, 'block>],
        result_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut new_builder = NewOperation::builder(builder.context, builder.unknown_location)
            .obj_name(StringAttribute::new(builder.context, obj_name))
            .val(val.inner);
        if let Some(salt) = salt {
            new_builder = new_builder.salt(salt.inner);
        }
        let new_builder = new_builder
            .ctor_args(ctor_args)
            .out(result_type.into_mlir());
        let mut operation: Operation = new_builder.build().into();
        let ctor_args_count =
            i32::try_from(ctor_args.len()).expect("constructor argument count fits in i32");
        let salt_segment = i32::from(salt.is_some());
        let segment_sizes =
            DenseI32ArrayAttribute::new(builder.context, &[1, salt_segment, ctor_args_count]);
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        Self::new(
            block
                .append_operation(operation)
                .result(0)
                .expect("sol.new always produces one result")
                .into(),
        )
    }

    /// `sol.keccak256` over a byte buffer, yielding the 32-byte hash. The buffer
    /// is coerced to memory first — a storage / calldata `bytes` is a reference,
    /// which solc copies to memory before hashing — a no-op when already memory.
    /// Shared by the `keccak256` built-in and `abi.encodeWithSignature`'s
    /// runtime-signature hash.
    pub fn keccak256(
        buffer: Self,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let input = buffer
            .cast(
                Type::string(builder.context, solx_utils::DataLocation::Memory),
                builder,
                block,
            )
            .into_mlir();
        Self::new(mlir_op!(
            builder,
            block,
            Keccak256Operation
                .addr(input)
                .result(Type::fixed_bytes(builder.context, 32))
        ))
    }

    /// `sol.encode` — the ABI-encoded `bytes memory` payload of `ins`. A
    /// `selector`, when present, is prepended as the leading 4 bytes and must
    /// already be `!sol.fixed_bytes<4>`. `packed` selects the packed encoding (no
    /// per-element padding, `abi.encodePacked`). Shared by every `abi.encode*`
    /// arm — the standard / packed encode, the selector / signature / call forms.
    ///
    /// Sets `operand_segment_sizes` by hand because melior's ODS-generated builder
    /// does not synthesize the attribute for this `AttrSizedOperandSegments` op;
    /// the dialect verifier rejects the op without it.
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

    /// A function/error selector or event topic as a `bytesN` value: the integer
    /// `value` at `width_bytes` width cast to `fixedbytes<width_bytes>` (4 bytes
    /// for a selector, 32 for an event topic).
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

    /// Appends a default element to this dynamic-array / `bytes` value (typed by
    /// its `base_type`) and returns the new element's place together with its
    /// element MLIR type (`sol.push`). A reference-typed element yields its
    /// reference directly (the caller copies into it via `sol.copy`); a value-typed
    /// one yields a `!sol.ptr` to the slot at the array's location (stored into via
    /// `sol.store`) — a reference element pushed as a pointer would force a
    /// memory→storage data-location cast the backend cannot lower.
    pub fn push_slot(
        self,
        base_type: &SlangType,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> (Self, MlirType<'context>) {
        let (element_type, location) = Type::dynamic_array_element(base_type, builder);
        let push_result_type = if Type::new(element_type).is_reference() {
            element_type
        } else {
            Type::pointer(builder.context, element_type, location).into_mlir()
        };
        let new_slot = Self::new(mlir_op!(
            builder,
            block,
            PushOperation.inp(self.into_mlir()).addr(push_result_type)
        ));
        (new_slot, element_type)
    }

    /// Calls this function-pointer value with `argument_values`, returning the
    /// decoded result values. Dispatch is on the value's actual reference kind,
    /// not slang's `is_externally_visible`: an internal pointer (`func_ref`, even
    /// for a `public` function used as a bare value) dispatches through `sol.icall`;
    /// an external one (`ext_func_ref`) through `sol.ext_icall`, forwarding
    /// `call_value` (or zero) as `msg.value` and dropping the status (a non-`try`
    /// call reverts internally on failure). A `view`/`pure` callee lowers to a
    /// STATICCALL via `is_static` (a function-pointer value carries no mutability,
    /// so it passes `false`).
    pub fn call_indirect(
        self,
        argument_values: &[MlirValue<'context, 'block>],
        result_types: &[MlirType<'context>],
        call_value: Option<MlirValue<'context, 'block>>,
        is_static: bool,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Vec<MlirValue<'context, 'block>> {
        if self.r#type().is_ext_function_ref() {
            let value = call_value.unwrap_or_else(|| Self::field_zero(builder, block).into_mlir());
            let mut out_types = Vec::with_capacity(result_types.len() + 1);
            out_types
                .push(Type::signless(builder.context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
            out_types.extend_from_slice(result_types);
            let mut operation_builder =
                ExtICallOperation::builder(builder.context, builder.unknown_location)
                    .outs(&out_types)
                    .callee(self.into_mlir())
                    .callee_operands(argument_values)
                    .gas(Self::gas_left(builder, block).into_mlir())
                    .value(value);
            if is_static {
                operation_builder = operation_builder.static_call(Attribute::unit(builder.context));
            }
            let operation = block.append_operation(operation_builder.build().into());
            (0..result_types.len())
                .map(|index| {
                    operation
                        .result(index + 1)
                        .expect("sol.ext_icall produces a status plus its declared results")
                        .into()
                })
                .collect()
        } else {
            let operation = block.append_operation(mlir_op_build!(
                builder,
                ICallOperation
                    .outs(result_types)
                    .callee(self.into_mlir())
                    .callee_operands(argument_values)
            ));
            (0..result_types.len())
                .map(|index| {
                    operation
                        .result(index)
                        .expect("sol.icall produces its declared result count")
                        .into()
                })
                .collect()
        }
    }

    /// Casts to `target_type`, handing the value to the target type's cast router
    /// ([`Type::cast`]) — the kind-dispatch that selects the dialect cast op
    /// (`sol.cast` / `sol.bytes_cast` / `sol.address_cast` / …), and a no-op when
    /// the value already has `target_type`. A cast to `i1` is a plain
    /// representation cast; truthiness (`x != 0`) is [`Self::is_nonzero`], not a cast.
    pub fn cast(
        self,
        target_type: Type<'context>,
        builder: &Builder<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        target_type.cast(self, builder, block)
    }

    /// Reinterprets the value's representation as `target_type` via
    /// `sol.conv_cast` — a representation-preserving cast the conversion pipeline
    /// rewrites to the remapped value. It crosses the inline-assembly boundary: a
    /// Solidity local's `!sol.ptr<T, Stack>` is reinterpreted as the `!llvm.ptr`
    /// that Yul `llvm.load`/`llvm.store` operate on. Returns the value unchanged
    /// when it already has `target_type`.
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
        let predicate_attribute = IntegerAttribute::new(
            IntegerType::new(builder.context, BIT_LENGTH_X64 as u32).into(),
            predicate as i64,
        );
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

    /// Tests the value against zero, producing an `i1`. Short-circuits when the
    /// value is already `i1` (e.g. from a `sol.cmp`), avoiding a redundant
    /// `sol.cmp ne, %i1, %zero`.
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
