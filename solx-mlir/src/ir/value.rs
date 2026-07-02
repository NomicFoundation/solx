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
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationMutLike;
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
use crate::ods::sol::ArrayLitOperation;
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
use crate::ods::sol::EnumCastOperation;
use crate::ods::sol::ExtCallOperation;
use crate::ods::sol::ExtFuncAddrOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtFuncSelectorOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::LibAddrOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::NewOperation;
use crate::ods::sol::PopOperation;
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

    /// Materialises a `sol.constant` from an arbitrary-width [`BigInt`], selecting the dialect by target
    /// type: an address is built at `ui160` and cast; a boolean folds to `0`/`1`; every other integer
    /// takes the big-integer attribute.
    pub fn constant_from_bigint<B>(
        value: &BigInt,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if result_type == Type::address(context.mlir_context, false) {
            let integer = Self::constant_from_bigint(
                value,
                Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                context,
                block,
            );
            return integer.address_cast(result_type, context, block);
        }
        let attribute: Attribute<'context> = if result_type.integer_bit_width()
            == solx_utils::BIT_LENGTH_BOOLEAN as u32
        {
            IntegerAttribute::new(
                result_type.into_mlir(),
                i64::from(*value != BigInt::ZERO),
            )
            .into()
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
    pub fn boolean<B>(value: bool, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::constant_from_bigint(
            &BigInt::from(u8::from(value)),
            Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN),
            context,
            block,
        )
    }

    /// A `string memory` value holding `text` verbatim, via `sol.string_lit`.
    /// The bytes are taken as-is; a string literal need not be valid UTF-8.
    pub fn string_literal<B>(text: &str, context: &Context<'context>, block: &B) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Self::new(mlir_op!(
            context,
            block,
            StringLitOperation
                .value(StringAttribute::new(context.mlir_context, text))
                .addr(Type::string(context.mlir_context, solx_utils::DataLocation::Memory).into_mlir())
        ))
    }

    /// `sol.malloc`: a fresh memory buffer typed as `result_type`. `Some` `size` allocates a
    /// dynamically-sized buffer; `zero_init` zero-fills it.
    pub fn malloc<B>(
        result_type: Type<'context>,
        size: Option<MlirValue<'context, 'block>>,
        zero_init: bool,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut builder = MallocOperation::builder(context.mlir_context, context.location())
            .addr(result_type.into_mlir());
        if let Some(size) = size {
            builder = builder.size(size);
        }
        if zero_init {
            builder = builder.zero_init(Attribute::unit(context.mlir_context));
        }
        Self::new(
            block
                .append_operation(builder.build().into())
                .result(0)
                .expect("sol.malloc produces one result")
                .into(),
        )
    }

    /// `sol.new`: contract creation embedding `object_name`'s deploy bytecode. `value` is the
    /// forwarded wei; a `salt` selects CREATE2.
    ///
    /// A `try`/`catch` guard passes `try_call` to mark the creation, so the conversion yields a
    /// success status instead of reverting on failure; otherwise the creation reverts and the caller
    /// discards the status.
    ///
    /// `operand_segment_sizes` is set by hand because melior's ODS builder does not synthesize the
    /// attribute for the `AttrSizedOperandSegments` op; the salt must precede the variadic constructor
    /// arguments or the two transpose.
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
        let mut builder = NewOperation::builder(context.mlir_context, context.location())
            .obj_name(StringAttribute::new(context.mlir_context, object_name))
            .val(value.inner);
        if let Some(salt) = salt {
            builder = builder.salt(salt.inner);
        }
        let mut builder = builder
            .ctor_args(constructor_arguments)
            .out(result_type.into_mlir());
        if try_call {
            builder = builder.try_call(Attribute::unit(context.mlir_context));
        }
        let mut operation: Operation = builder.build().into();
        let constructor_argument_count = i32::try_from(constructor_arguments.len())
            .expect("constructor argument count fits in i32");
        let segment_sizes = DenseI32ArrayAttribute::new(
            context.mlir_context,
            &[1, i32::from(salt.is_some()), constructor_argument_count],
        );
        operation.set_inherent_attribute("operand_segment_sizes", segment_sizes.into());
        Self::new(
            block
                .append_operation(operation)
                .result(0)
                .expect("sol.new always produces one result")
                .into(),
        )
    }

    /// `sol.ext_call` to a statically-resolved external callee `callee_name`, dispatched by
    /// `selector` at `receiver` cast to `address`. Returns the success status and the decoded
    /// results.
    ///
    /// `call_value` selects the forwarded wei, defaulting to zero; `call_gas` selects the forwarded
    /// gas, defaulting to `sol.gasleft`. A `view`/`pure` callee passes `is_static` for a `STATICCALL`.
    /// A `try`/`catch` guard passes `try_call` to surface the status; otherwise the call reverts on
    /// failure and the caller discards the status.
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
        let uint256 = Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD);
        let address = receiver
            .address_cast(Type::address(context.mlir_context, false), context, block)
            .into_mlir();
        let value = call_value
            .unwrap_or_else(|| Self::constant(0, uint256, context, block).into_mlir());
        let gas = call_gas.unwrap_or_else(|| {
            block
                .append_operation(
                    GasLeftOperation::builder(context.mlir_context, context.location())
                        .val(uint256.into_mlir())
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.gasleft produces one result")
                .into()
        });
        let selector_value =
            Self::constant(i64::from(selector), uint256, context, block).into_mlir();
        let callee_type = FunctionType::new(context.mlir_context, parameter_types, result_types);
        let mut builder = ExtCallOperation::builder(context.mlir_context, context.location())
            .callee(StringAttribute::new(context.mlir_context, callee_name))
            .ins(argument_values)
            .addr(address)
            .gas(gas)
            .val(value)
            .selector(selector_value)
            .callee_type(TypeAttribute::new(callee_type.into()))
            .status(Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
            .outs(result_types);
        if is_static {
            builder = builder.static_call(Attribute::unit(context.mlir_context));
        }
        if try_call {
            builder = builder.try_call(Attribute::unit(context.mlir_context));
        }
        let operation = block.append_operation(builder.build().into());
        let status = operation
            .result(0)
            .expect("sol.ext_call produces a status result")
            .into();
        let results = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces its declared result count")
                    .into()
            })
            .collect();
        (status, results)
    }

    /// `sol.lib_addr`: a library's linked deploy address, a placeholder the linker resolves by the
    /// library's full path.
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

    /// `sol.ext_call` to a library function `callee_name` at its deployed `address`, dispatched by
    /// `selector`. A library call is always a `DELEGATECALL`, so it carries the `delegate_call` and
    /// `library_call` markers, reverts on failure, and discards the status.
    pub fn library_call(
        address: Self,
        callee_name: &str,
        selector: u32,
        parameter_types: &[MlirType<'context>],
        argument_values: &[MlirValue<'context, 'block>],
        result_types: &[MlirType<'context>],
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Vec<MlirValue<'context, 'block>> {
        let uint256 = Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD);
        let gas = block
            .append_operation(
                GasLeftOperation::builder(context.mlir_context, context.location())
                    .val(uint256.into_mlir())
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gasleft produces one result")
            .into();
        let value = Self::constant(0, uint256, context, block).into_mlir();
        let selector_value =
            Self::constant(i64::from(selector), uint256, context, block).into_mlir();
        let callee_type = FunctionType::new(context.mlir_context, parameter_types, result_types);
        let operation = block.append_operation(
            ExtCallOperation::builder(context.mlir_context, context.location())
                .callee(StringAttribute::new(context.mlir_context, callee_name))
                .ins(argument_values)
                .addr(address.into_mlir())
                .gas(gas)
                .val(value)
                .selector(selector_value)
                .delegate_call(Attribute::unit(context.mlir_context))
                .library_call(Attribute::unit(context.mlir_context))
                .callee_type(TypeAttribute::new(callee_type.into()))
                .status(Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
                .outs(result_types)
                .build()
                .into(),
        );
        (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces its declared result count")
                    .into()
            })
            .collect()
    }

    /// `sol.default_storage`: the default value of a storage or transient aggregate.
    pub fn default_storage<B>(
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
            DefaultStorageOperation.result(result_type.into_mlir())
        ))
    }

    /// `sol.default_calldata`: the default value of a calldata aggregate.
    pub fn default_calldata<B>(
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
            DefaultCallDataOperation.result(result_type.into_mlir())
        ))
    }

    /// The zero value of `mlir_type`, chosen by its Solidity `slang_type`.
    ///
    /// An aggregate resolves to its data location's empty value: a memory array / struct / `bytes` /
    /// `string` to a `sol.malloc` buffer (zero-filled for arrays and structs), a storage or transient
    /// aggregate to `sol.default_storage`, a calldata aggregate to `sol.default_calldata`. An address
    /// or contract zeroes at `ui160` and casts through `sol.address_cast`, a `bytesN` at `ui<N*8>`
    /// through `sol.bytes_cast`, an enumeration at `ui256` through `sol.enum_cast`, and every remaining
    /// scalar takes the `sol.constant` zero of its own type.
    pub fn type_default(
        slang_type: &SlangType,
        mlir_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        match slang_type {
            SlangType::Array(array) => {
                Self::aggregate_default(array.location(), mlir_type, true, context, block)
            }
            SlangType::FixedSizeArray(array) => {
                Self::aggregate_default(array.location(), mlir_type, true, context, block)
            }
            SlangType::Struct(structure) => {
                Self::aggregate_default(structure.location(), mlir_type, true, context, block)
            }
            SlangType::String(string) => {
                Self::aggregate_default(string.location(), mlir_type, false, context, block)
            }
            SlangType::Bytes(bytes) => {
                Self::aggregate_default(bytes.location(), mlir_type, false, context, block)
            }
            SlangType::Address(_) | SlangType::Contract(_) | SlangType::Interface(_) => {
                let address_type = Type::address(context.mlir_context, false);
                Self::constant(
                    0,
                    Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                    context,
                    block,
                )
                .address_cast(address_type, context, block)
                .address_cast(mlir_type, context, block)
            }
            SlangType::ByteArray(byte_array) => {
                let bits = byte_array.width() as usize * solx_utils::BIT_LENGTH_BYTE;
                Self::constant(0, Type::unsigned(context.mlir_context, bits), context, block)
                    .bytes_cast(mlir_type, context, block)
            }
            SlangType::Function(function_type) if function_type.is_externally_visible() => {
                let address = Self::constant(
                    0,
                    Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_ETH_ADDRESS),
                    context,
                    block,
                )
                .address_cast(Type::address(context.mlir_context, false), context, block);
                Self::external_function_constant(address, 0, mlir_type, context, block)
            }
            SlangType::Function(_) => Self::function_pointer_zero(mlir_type, context, block),
            SlangType::Enum(_) => {
                Self::constant(
                    0,
                    Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD),
                    context,
                    block,
                )
                .enum_cast(mlir_type, context, block)
            }
            _ => Self::constant(0, mlir_type, context, block),
        }
    }

    /// The empty aggregate value of `result_type` at its Solidity `location`: a memory buffer via
    /// `sol.malloc` (`zero_init` zero-fills a fixed-shape aggregate but not a dynamic `bytes` /
    /// `string`), a storage aggregate via `sol.default_storage`, a calldata aggregate via
    /// `sol.default_calldata`.
    fn aggregate_default(
        location: DataLocation,
        result_type: Type<'context>,
        zero_init: bool,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        match location {
            DataLocation::Memory => Self::malloc(result_type, None, zero_init, context, block),
            DataLocation::Storage => Self::default_storage(result_type, context, block),
            DataLocation::Calldata => Self::default_calldata(result_type, context, block),
            DataLocation::Inherited => {
                unreachable!("a reference aggregate default carries a resolved data location")
            }
        }
    }

    /// `sol.array_lit`: an array of `array_type` constructed from `elements`.
    pub fn array_literal<B>(
        elements: &[MlirValue<'context, 'block>],
        array_type: Type<'context>,
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
            ArrayLitOperation.ins(elements).addr(array_type.into_mlir())
        ))
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

    /// `sol.default_func_constant`: the zero internal function pointer of `result_type`, which reverts
    /// when called; the `delete` reset of a function-pointer lvalue.
    pub fn function_pointer_zero(
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            DefaultFuncConstantOperation.addr(result_type.into_mlir())
        ))
    }

    /// `sol.ext_func_constant`: an external function pointer (`!sol.ext_func_ref<...>`) packing the
    /// callee `address` and its four-byte `selector`.
    pub fn external_function_constant(
        address: Self,
        selector: u32,
        result_type: Type<'context>,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncConstantOperation
                .addr(address.into_mlir())
                .selector(IntegerAttribute::new(
                    IntegerType::new(context.mlir_context, Type::SELECTOR_BIT_WIDTH).into(),
                    i64::from(selector),
                ))
                .result(result_type.into_mlir())
        ))
    }

    /// The `!sol.ext_func_ref<...>` value of an external interaction: `receiver` cast to `address`,
    /// packed with `selector` over `parameter_types -> result_types`.
    pub fn external_callee(
        receiver: Self,
        selector: u32,
        parameter_types: &[MlirType<'context>],
        result_types: &[MlirType<'context>],
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        let address =
            receiver.address_cast(Type::address(context.mlir_context, false), context, block);
        let result_type = Type::ext_func_ref(context.mlir_context, parameter_types, result_types);
        Self::external_function_constant(address, selector, result_type, context, block)
    }

    /// Calls this function-pointer value, returning the decoded results. An internal `!sol.func_ref`
    /// dispatches through `sol.icall`; an external `!sol.ext_func_ref` through `sol.ext_icall`,
    /// forwarding `call_value` as `msg.value`, `call_gas` as the gas, and dropping the status.
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
        if self.r#type().is_external_function_ref() {
            let (_status, results) = self.external_call_indirect(
                argument_values,
                result_types,
                call_value,
                call_gas,
                is_static,
                false,
                context,
                block,
            );
            return results;
        }
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

    /// Calls this external function-pointer value through `sol.ext_icall`, returning the success
    /// status and the decoded results. `call_value` forwards `msg.value` (defaulting to zero) and
    /// `call_gas` the gas (defaulting to `sol.gasleft`). A `view`/`pure` pointer passes `is_static`
    /// for a `STATICCALL`; a `try`/`catch` guard passes `try_call` to surface the status.
    pub fn external_call_indirect(
        self,
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
        let uint256 = Type::unsigned(context.mlir_context, solx_utils::BIT_LENGTH_FIELD);
        let value = call_value
            .unwrap_or_else(|| Self::constant(0, uint256, context, block).into_mlir());
        let gas = call_gas.unwrap_or_else(|| {
            block
                .append_operation(
                    GasLeftOperation::builder(context.mlir_context, context.location())
                        .val(uint256.into_mlir())
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.gasleft produces one result")
                .into()
        });
        let mut out_types = Vec::with_capacity(result_types.len() + 1);
        out_types
            .push(Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir());
        out_types.extend_from_slice(result_types);
        let mut builder = ExtICallOperation::builder(context.mlir_context, context.location())
            .outs(&out_types)
            .callee(self.into_mlir())
            .callee_operands(argument_values)
            .gas(gas)
            .value(value);
        if is_static {
            builder = builder.static_call(Attribute::unit(context.mlir_context));
        }
        if try_call {
            builder = builder.try_call(Attribute::unit(context.mlir_context));
        }
        let operation = block.append_operation(builder.build().into());
        let status = operation
            .result(0)
            .expect("sol.ext_icall produces a status result")
            .into();
        let results = (0..result_types.len())
            .map(|index| {
                operation
                    .result(index + 1)
                    .expect("sol.ext_icall produces its declared result count")
                    .into()
            })
            .collect();
        (status, results)
    }

    /// The four-byte selector of this external function-pointer value, via `sol.ext_func_selector`.
    pub fn external_function_selector(
        self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncSelectorOperation
                .func(self.into_mlir())
                .result(Type::fixed_bytes(context.mlir_context, 4).into_mlir())
        ))
    }

    /// The callee `address` of this external function-pointer value, via `sol.ext_func_addr`.
    pub fn external_function_address(
        self,
        context: &Context<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> Self {
        Self::new(mlir_op!(
            context,
            block,
            ExtFuncAddrOperation
                .func(self.into_mlir())
                .result(Type::address(context.mlir_context, false).into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.cast`, a no-op when already that type.
    pub fn cast<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
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
    /// inline-assembly boundary: a `!sol.ptr<T, Stack>` as the `!llvm.ptr` Yul operates on. A no-op
    /// when already that type.
    pub fn reinterpret<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
            block,
            ConvCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.bytes_cast`, between byte / fixed-bytes / integer types; a
    /// no-op when already that type.
    pub fn bytes_cast<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
            block,
            BytesCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.address_cast`, between address and integer types.
    pub fn address_cast<B>(
        self,
        target_type: Type<'context>,
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
            AddressCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.contract_cast`, between two contract or interface types.
    pub fn contract_cast<B>(
        self,
        target_type: Type<'context>,
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
            ContractCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.enum_cast`, between an enumeration and its backing integer.
    pub fn enum_cast<B>(
        self,
        target_type: Type<'context>,
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
            EnumCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts to `target_type` via `sol.data_loc_cast`, relocating a reference value between data
    /// locations; a no-op when already that type.
    pub fn data_loc_cast<B>(
        self,
        target_type: Type<'context>,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if self.r#type() == target_type {
            return self;
        }
        Self::new(mlir_op!(
            context,
            block,
            DataLocCastOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Casts a dynamic-bytes value to `target_type` via `sol.dyn_bytes_to_fixedbytes`, the fixed-width
    /// prefix of a `bytes` value.
    pub fn dyn_bytes_to_fixedbytes<B>(
        self,
        target_type: Type<'context>,
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
            DynBytesToFixedBytesOperation
                .inp(self.into_mlir())
                .out(target_type.into_mlir())
        ))
    }

    /// Appends a default element to this dynamic array / `bytes` value (`sol.push`), returning the
    /// reference to the newly appended slot of `slot_type`.
    pub fn push<B>(
        self,
        slot_type: Type<'context>,
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
            PushOperation.inp(self.inner).addr(slot_type.into_mlir())
        ))
    }

    /// Removes the last element of this dynamic array / `bytes` value (`sol.pop`).
    pub fn pop<B>(self, context: &Context<'context>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        mlir_op_void!(context, block, PopOperation.inp(self.inner));
    }

    /// Compares this value against `other` under `predicate` via `sol.cmp`, producing an `i1`.
    pub fn compare<B>(
        self,
        other: Self,
        predicate: CmpPredicate,
        context: &Context<'context>,
        block: &B,
    ) -> Self
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let predicate_attribute = predicate.attribute(context.mlir_context);
        Self::new(mlir_op!(
            context,
            block,
            CmpOperation
                .predicate(Attribute::from(predicate_attribute))
                .lhs(self.inner)
                .rhs(other.inner)
                .result(Type::signless(context.mlir_context, solx_utils::BIT_LENGTH_BOOLEAN).into_mlir())
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
