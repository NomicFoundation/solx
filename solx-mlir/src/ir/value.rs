//!
//! An MLIR value in the Sol dialect, and the conversions it undergoes.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Type as MlirType;
use melior::ir::Value as MlirValue;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;
use num::BigInt;
use slang_solidity_v2::ast::DataLocation;
use slang_solidity_v2::ast::Type as SlangType;

use crate::CmpPredicate;
use crate::Context;
use crate::IntoOds;
use crate::Pointer;
use crate::Type;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ContractCastOperation;
use crate::ods::sol::ConvCastOperation;
use crate::ods::sol::DataLocCastOperation;
use crate::ods::sol::DefaultCallDataOperation;
use crate::ods::sol::DefaultFuncConstantOperation;
use crate::ods::sol::DefaultStorageOperation;
use crate::ods::sol::DynBytesToFixedBytesOperation;
use crate::ods::sol::EncodeOperation;
use crate::ods::sol::EnumCastOperation;
use crate::ods::sol::ExtCallOperation;
use crate::ods::sol::ExtFuncAddrOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtFuncSelectorOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::Keccak256Operation;
use crate::ods::sol::LengthOperation;
use crate::ods::sol::LibAddrOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::NewOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::StringLitOperation;

/// An MLIR value in the Sol dialect; the home for the conversions a value undergoes.
#[derive(Clone, Copy)]
pub struct Value<'context, 'block> {
    /// The wrapped melior value.
    pub inner: MlirValue<'context, 'block>,
}

impl<'context, 'block> Value<'context, 'block> {
    /// Wraps a melior value.
    pub fn new(inner: MlirValue<'context, 'block>) -> Self {
        Self { inner }
    }

    /// Materialises a `sol.constant` of `result_type` from an `i64`-sized value.
    pub fn constant<B>(
        value: i64,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let result_type = result_type.into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            ConstantOperation
                .value(Attribute::from(IntegerAttribute::new(result_type, value)))
                .result(result_type)
        ))
    }

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`], for literals that overflow `i64`.
    pub fn constant_from_bigint(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if result_type == Type::address(context.mlir_context, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                context,
                block,
            );
            return integer.cast(result_type, context, block);
        }
        let attribute: Attribute<'context> = if result_type.integer_bit_width()
            == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(result_type.into_mlir(), i64::from(*value != BigInt::ZERO)).into()
        } else {
            result_type.big_integer_attribute(value)
        };
        Self::new(mlir_op!(
            context,
            block,
            ConstantOperation
                .value(attribute)
                .result(result_type.into_mlir())
        ))
    }

    /// Materialises an `i1` boolean constant.
    pub fn boolean(
        value: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN),
            context,
            block,
        )
    }

    /// A `uint256` constant: Solidity's default integer width.
    pub fn uint256(
        value: i64,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::constant(
            value,
            Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
            context,
            block,
        )
    }

    /// The zero of a scalar value type, built at its own representation width and cast through the type.
    pub fn zero(
        r#type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if r#type.is_address() {
            let bits = Self::constant(
                0,
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                context,
                block,
            );
            bits.cast(r#type, context, block)
        } else if r#type.is_contract() {
            let address = Self::zero(Type::address(context.mlir_context, false), context, block);
            address.cast(r#type, context, block)
        } else if let Some(width) = r#type.fixed_bytes_or_byte_width() {
            let bits = Self::constant(
                0,
                Type::unsigned(
                    context.mlir_context,
                    width as usize * solx_utils::BIT_LENGTH_BYTE,
                ),
                context,
                block,
            );
            bits.cast(r#type, context, block)
        } else if r#type.is_enum() {
            let bits = Self::constant(
                0,
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                context,
                block,
            );
            bits.cast(r#type, context, block)
        } else if r#type.is_ext_function_ref() {
            let address = Self::zero(Type::address(context.mlir_context, false), context, block);
            Self::ext_func_constant(address, 0, r#type, context, block)
        } else if r#type.is_function_ref() {
            Self::new(mlir_op!(
                context,
                block,
                DefaultFuncConstantOperation.addr(r#type.into_mlir())
            ))
        } else if IntegerType::try_from(r#type.into_mlir()).is_ok() {
            Self::constant(0, r#type, context, block)
        } else {
            unreachable!("Self::zero handles only scalar value types")
        }
    }

    /// The default value of a return position reached without an explicit `return <value>`, keyed on the Slang type.
    pub fn type_default(
        slang_type: Option<&SlangType>,
        mlir_type: MlirType<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let is_memory = |location| matches!(location, DataLocation::Memory);
        match slang_type {
            Some(SlangType::FixedSizeArray(array)) if is_memory(array.location()) => {
                Self::malloc(mlir_type, None, true, context, block)
            }
            Some(SlangType::Struct(structure)) if is_memory(structure.location()) => {
                Self::malloc(mlir_type, None, true, context, block)
            }
            Some(SlangType::Array(array)) if is_memory(array.location()) => {
                Self::malloc(mlir_type, None, true, context, block)
            }
            Some(SlangType::String(_) | SlangType::Bytes(_)) => {
                Self::malloc(mlir_type, None, false, context, block)
            }
            Some(
                SlangType::Address(_)
                | SlangType::ByteArray(_)
                | SlangType::Enum(_)
                | SlangType::UserDefinedValue(_)
                | SlangType::Function(_)
                | SlangType::Contract(_)
                | SlangType::Interface(_),
            ) => Self::zero(Type::new(mlir_type), context, block),
            _ => Self::constant(0, Type::new(mlir_type), context, block),
        }
    }

    /// A `string memory` value holding `text` verbatim, via `sol.string_lit`.
    /// The bytes are taken as-is; a string literal need not be valid UTF-8.
    pub fn string_literal(
        text: &str,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            StringLitOperation
                .value(StringAttribute::new(context.mlir_context, text))
                .addr(
                    Type::string(context.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir()
                )
        ))
    }

    /// `sol.malloc`: a fresh memory buffer typed as `mlir_type`. `Some` `size`
    /// allocates a dynamically-sized buffer; `zero_init` zero-fills it.
    pub fn malloc(
        mlir_type: MlirType<'context>,
        size: Option<MlirValue<'context, 'block>>,
        zero_init: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut operation_builder =
            MallocOperation::builder(context.mlir_context, context.location()).addr(mlir_type);
        if let Some(size) = size {
            operation_builder = operation_builder.size(size);
        }
        if zero_init {
            operation_builder = operation_builder.zero_init(Attribute::unit(context.mlir_context));
        }
        Self::new(
            block
                .append_operation(operation_builder.build().into())
                .result(0)
                .expect("sol.malloc produces one result")
                .into(),
        )
    }

    /// `sol.default_storage`: the default value of a storage or transient aggregate.
    pub fn default_storage(
        r#type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            DefaultStorageOperation.result(r#type.into_mlir())
        ))
    }

    /// `sol.default_calldata`: the default value of a calldata aggregate.
    pub fn default_calldata(
        r#type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            DefaultCallDataOperation.result(r#type.into_mlir())
        ))
    }

    /// A selector or event topic as a `bytesN` value: `value` at `width_bytes` width cast to `fixedbytes<width_bytes>`.
    pub fn selector_constant(
        value: &BigInt,
        width_bytes: u32,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let integer = Self::constant_from_bigint(
            value,
            Type::unsigned(
                context.mlir_context,
                width_bytes as usize * solx_utils::BIT_LENGTH_BYTE,
            ),
            context,
            block,
        );
        integer.cast(
            Type::fixed_bytes(context.mlir_context, width_bytes),
            context,
            block,
        )
    }

    /// `sol.func_constant`: an internal function pointer (`!sol.func_ref<...>`) to the symbol `name`.
    pub fn function_constant(
        name: &str,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            FuncConstantOperation
                .addr(result_type.into_mlir())
                .sym(FlatSymbolRefAttribute::new(context.mlir_context, name))
        ))
    }

    /// `sol.ext_func_constant` packing a callee `address` and 4-byte `selector` into an `!sol.ext_func_ref<...>`.
    pub fn ext_func_constant<B>(
        address: Self,
        selector: u32,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncConstantOperation
                .addr(address.inner)
                .selector(IntegerAttribute::new(
                    IntegerType::new(context.mlir_context, Type::SELECTOR_BIT_WIDTH).into(),
                    selector as i64,
                ))
                .result(result_type.into_mlir())
        ))
    }

    /// The `!sol.ext_func_ref<...>` callee of an external interaction: `receiver` cast to `address`, packed with `selector`.
    pub fn external_callee(
        receiver: Self,
        selector: u32,
        parameter_types: &[MlirType<'context>],
        return_types: &[MlirType<'context>],
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let address = receiver.cast(Type::address(context.mlir_context, false), context, block);
        let ext_func_ref_type =
            Type::ext_func_ref(context.mlir_context, parameter_types, return_types);
        Self::ext_func_constant(address, selector, ext_func_ref_type, context, block)
    }

    /// `sol.lib_addr`: a library's linked deploy address, a placeholder the linker resolves by the library's full path.
    pub fn library_address(
        name: &solx_utils::ContractName,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            LibAddrOperation
                ._name(StringAttribute::new(context.mlir_context, &name.full_path))
                .val(Type::address(context.mlir_context, false).into_mlir())
        ))
    }

    /// `sol.gasleft`: all remaining gas as a `ui256`.
    pub fn gas_left<B>(context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            GasLeftOperation.val(
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            )
        ))
    }

    /// `sol.new`: contract creation embedding `object_name`'s deploy bytecode. `value`
    /// is the forwarded wei; a `salt` selects CREATE2. The salt must be appended
    /// before the variadic constructor arguments or the two transpose.
    pub fn create_contract(
        object_name: &str,
        value: Self,
        salt: Option<Self>,
        constructor_arguments: &[MlirValue<'context, 'block>],
        result_type: Type<'context>,
        try_call: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut operation_builder = NewOperation::builder(context.mlir_context, context.location())
            .obj_name(StringAttribute::new(context.mlir_context, object_name))
            .val(value.inner);
        if let Some(salt) = salt {
            operation_builder = operation_builder.salt(salt.inner);
        }
        let mut operation_builder = operation_builder
            .ctor_args(constructor_arguments)
            .out(result_type.into_mlir());
        if try_call {
            operation_builder = operation_builder.try_call(Attribute::unit(context.mlir_context));
        }
        Self::new(
            block
                .append_operation(operation_builder.build().into())
                .result(0)
                .expect("sol.new always produces one result")
                .into(),
        )
    }

    /// `sol.ext_call` to a statically-resolved external callee `callee_name`, dispatched by `selector`
    /// at `receiver` cast to `address`. Returns the success status and the decoded results. With
    /// `try_call` the status is surfaced for a `try`/`catch`; otherwise the call reverts on failure and
    /// the caller discards the status. A `view`/`pure` callee passes `is_static` for a `STATICCALL`.
    pub fn external_call(
        receiver: Self,
        callee_name: &str,
        selector: u32,
        parameter_types: &[MlirType<'context>],
        argument_values: &[MlirValue<'context, 'block>],
        result_types: &[MlirType<'context>],
        call_value: Option<MlirValue<'context, 'block>>,
        call_gas: Option<MlirValue<'context, 'block>>,
        is_static: bool,
        try_call: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> (
        MlirValue<'context, 'block>,
        Vec<MlirValue<'context, 'block>>,
    ) {
        let address = receiver
            .cast(Type::address(context.mlir_context, false), context, block)
            .into_mlir();
        let value = call_value.unwrap_or_else(|| Self::uint256(0, context, block).into_mlir());
        let gas = call_gas.unwrap_or_else(|| Self::gas_left(context, block).into_mlir());
        let selector_value = Self::uint256(i64::from(selector), context, block).into_mlir();
        let callee_type = FunctionType::new(context.mlir_context, parameter_types, result_types);
        let mut operation_builder =
            ExtCallOperation::builder(context.mlir_context, context.location())
                .callee(StringAttribute::new(context.mlir_context, callee_name))
                .ins(argument_values)
                .addr(address)
                .gas(gas)
                .val(value)
                .selector(selector_value)
                .callee_type(TypeAttribute::new(callee_type.into()))
                .status(
                    Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN)
                        .into_mlir(),
                )
                .outs(result_types);
        if is_static {
            operation_builder =
                operation_builder.static_call(Attribute::unit(context.mlir_context));
        }
        if try_call {
            operation_builder = operation_builder.try_call(Attribute::unit(context.mlir_context));
        }
        let operation = block.append_operation(operation_builder.build().into());
        let status = operation
            .result(0)
            .expect("sol.ext_call produces a status result")
            .into();
        let results = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces a status plus its declared results")
                    .into()
            })
            .collect();
        (status, results)
    }

    /// Calls this function-pointer value, returning the decoded results. Dispatch is on the value's
    /// reference kind: an internal `func_ref` through `sol.icall`, an external `ext_func_ref` through
    /// `sol.ext_icall`, forwarding `call_value` as `msg.value` and dropping the status.
    pub fn call_indirect(
        self,
        argument_values: &[MlirValue<'context, 'block>],
        result_types: &[MlirType<'context>],
        call_value: Option<MlirValue<'context, 'block>>,
        call_gas: Option<MlirValue<'context, 'block>>,
        is_static: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Vec<MlirValue<'context, 'block>> {
        if self.r#type().is_ext_function_ref() {
            let value = call_value.unwrap_or_else(|| Self::uint256(0, context, block).into_mlir());
            let gas = call_gas.unwrap_or_else(|| Self::gas_left(context, block).into_mlir());
            let mut out_types = Vec::with_capacity(result_types.len() + 1);
            out_types.push(
                Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir(),
            );
            out_types.extend_from_slice(result_types);
            let mut operation_builder =
                ExtICallOperation::builder(context.mlir_context, context.location())
                    .outs(&out_types)
                    .callee(self.into_mlir())
                    .callee_operands(argument_values)
                    .gas(gas)
                    .value(value);
            if is_static {
                operation_builder =
                    operation_builder.static_call(Attribute::unit(context.mlir_context));
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
                context,
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

    /// `sol.encode`: the ABI-encoded `bytes memory` payload of `ins`. A `selector`,
    /// when present, is prepended as the leading 4 bytes; `packed` selects `abi.encodePacked`.
    pub fn abi_encode(
        ins: &[MlirValue<'context, 'block>],
        selector: Option<MlirValue<'context, 'block>>,
        packed: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let mut operation_builder =
            EncodeOperation::builder(context.mlir_context, context.location())
                .ins(ins)
                .res(
                    Type::string(context.mlir_context, solx_utils::DataLocation::Memory)
                        .into_mlir(),
                );
        if let Some(selector_value) = selector {
            operation_builder = operation_builder.selector(selector_value);
        }
        if packed {
            operation_builder = operation_builder.packed(Attribute::unit(context.mlir_context));
        }
        Self::new(
            block
                .append_operation(operation_builder.build().into())
                .result(0)
                .expect("sol.encode always produces one result")
                .into(),
        )
    }

    /// `sol.keccak256` over a byte buffer, yielding the 32-byte hash. The buffer is coerced to memory first.
    pub fn keccak256(
        buffer: Self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let input = buffer
            .cast(
                Type::string(context.mlir_context, solx_utils::DataLocation::Memory),
                context,
                block,
            )
            .into_mlir();
        Self::new(mlir_op!(
            context,
            block,
            Keccak256Operation.addr(input).result(Type::fixed_bytes(
                context.mlir_context,
                solx_utils::BYTE_LENGTH_FIELD as u32,
            ))
        ))
    }

    /// Casts to `target_type`, a no-op when already that type; each non-integer kind routes to its own cast op.
    pub fn cast(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let source = self.r#type();
        if source == target_type {
            return self;
        }
        if source.is_enum() || target_type.is_enum() {
            return Self::new(mlir_op!(
                context,
                block,
                EnumCastOperation
                    .inp(self.into_mlir())
                    .out(target_type.into_mlir())
            ));
        }
        if source.is_contract() && target_type.is_contract() {
            return Self::new(mlir_op!(
                context,
                block,
                ContractCastOperation
                    .inp(self.into_mlir())
                    .out(target_type.into_mlir())
            ));
        }
        if source.is_address() || target_type.is_address() {
            let ui160 = Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS);
            if source.is_address() {
                if target_type.is_contract() || target_type.is_fixed_bytes() || target_type == ui160
                {
                    return self.address_cast(target_type, context, block);
                }
                let as_160 = self.address_cast(ui160, context, block);
                return as_160.cast(target_type, context, block);
            }
            if source.is_contract() || source.is_fixed_bytes() || source == ui160 {
                return self.address_cast(target_type, context, block);
            }
            let as_160 = self.cast(ui160, context, block);
            return as_160.address_cast(target_type, context, block);
        }
        if source.is_reference() && target_type.is_fixed_bytes() {
            return Self::new(mlir_op!(
                context,
                block,
                DynBytesToFixedBytesOperation
                    .inp(self.into_mlir())
                    .out(target_type.into_mlir())
            ));
        }
        if source.is_fixed_bytes() || source.is_byte() {
            let bridge_bits = source.fixed_bytes_integer_bits();
            if let Ok(integer) = IntegerType::try_from(target_type.into_mlir())
                && integer.width() != bridge_bits
            {
                let bridge = Type::unsigned(context.mlir_context, bridge_bits as usize);
                let as_int = self.bytes_cast(bridge, context, block);
                return as_int.cast(target_type, context, block);
            }
            return self.bytes_cast(target_type, context, block);
        }
        if target_type.is_fixed_bytes() || target_type.is_byte() {
            let bridge_bits = target_type.fixed_bytes_integer_bits();
            if let Ok(integer) = IntegerType::try_from(source.into_mlir())
                && integer.width() != bridge_bits
            {
                let bridge = Type::unsigned(context.mlir_context, bridge_bits as usize);
                let as_int = self.cast(bridge, context, block);
                return as_int.bytes_cast(target_type, context, block);
            }
            return self.bytes_cast(target_type, context, block);
        }
        if source.is_reference() && target_type.is_reference() {
            return Self::new(mlir_op!(
                context,
                block,
                DataLocCastOperation
                    .inp(self.into_mlir())
                    .out(target_type.into_mlir())
            ));
        }
        Self::new(mlir_op!(
            context,
            block,
            CastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Reinterprets the value's representation as `target_type` via `sol.conv_cast`, e.g. across the
    /// inline-assembly boundary: a `!sol.ptr<T, Stack>` as the `!llvm.ptr` Yul operates on.
    pub fn reinterpret(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
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
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let predicate_attribute = predicate.attribute(context.mlir_context);
        let value: MlirValue<'context, 'block> = mlir_op!(
            context,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .result(
                    Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN)
                        .into_mlir()
                )
        );
        Self::new(value)
    }

    /// Tests the value against zero, producing an `i1`. Short-circuits when already `i1`.
    pub fn is_nonzero(
        self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        if self.r#type() == Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN) {
            return self;
        }
        let zero = Self::constant(0, self.r#type(), context, block);
        self.compare(zero, CmpPredicate::Ne, context, block)
    }

    /// The length of this dynamic array, `bytes`, or `string` value as a `ui256`, via `sol.length`.
    pub fn length(self, context: &Context<'context>, block: &BlockRef<'context, 'block>) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            LengthOperation.inp(self.inner).len(
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD).into_mlir()
            )
        ))
    }

    /// Appends a default element to this dynamic array / `bytes` (`sol.push`), returning the element's
    /// place and MLIR type. A reference element yields its reference directly; a value element a `!sol.ptr`.
    pub fn push_slot(
        self,
        base_type: &SlangType,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> (Self, MlirType<'context>) {
        let (element_type, location) = Type::dynamic_array_element(base_type, context);
        let push_result_type = Type::new(element_type)
            .address_type(location, context.mlir_context)
            .into_mlir();
        let new_slot = Self::new(mlir_op!(
            context,
            block,
            PushOperation.inp(self.into_mlir()).addr(push_result_type)
        ));
        (new_slot, element_type)
    }

    /// The 4-byte selector of this external function-pointer value, via `sol.ext_func_selector`.
    pub fn ext_func_selector(
        self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncSelectorOperation.func(self.inner).result(
                Type::fixed_bytes(context.mlir_context, solx_utils::BYTE_LENGTH_X32 as u32)
                    .into_mlir()
            )
        ))
    }

    /// The `address` component of this external function-pointer value, via `sol.ext_func_addr`.
    pub fn ext_func_address(
        self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncAddrOperation
                .func(self.inner)
                .result(Type::address(context.mlir_context, false).into_mlir())
        ))
    }

    /// The value's type.
    pub fn r#type(self) -> Type<'context> {
        Type::new(self.inner.r#type())
    }

    /// The inner melior value, for the op-construction boundary.
    pub fn into_mlir(self) -> MlirValue<'context, 'block> {
        self.inner
    }

    /// Emits a `sol.bytes_cast` to the `target_type` byte / fixed-bytes / integer target.
    fn bytes_cast(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            BytesCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Emits a `sol.address_cast` to the address-side `target_type`.
    fn address_cast(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            AddressCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }
}

impl<'context, 'block> From<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn from(inner: MlirValue<'context, 'block>) -> Self {
        Self::new(inner)
    }
}

impl<'context, 'block> From<Pointer<'context, 'block>> for Value<'context, 'block> {
    /// A `!sol.ptr` place is itself a first-class SSA value; both wrap the same handle.
    fn from(pointer: Pointer<'context, 'block>) -> Self {
        Self::new(pointer.into_mlir())
    }
}

impl<'context, 'block> IntoOds<MlirValue<'context, 'block>> for Value<'context, 'block> {
    fn into_ods(self) -> MlirValue<'context, 'block> {
        self.into_mlir()
    }
}
