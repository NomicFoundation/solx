//!
//! MLIR builder for Sol dialect emission.
//!
//! Contains the [`Builder`] type with cached MLIR types and emission methods
//! for Sol dialect operations: contracts, functions, constants, control flow,
//! memory, comparisons, calls, state variables, and EVM context intrinsics.
//!

pub mod type_factory;

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use ruint::aliases::U256;

use crate::CmpPredicate;
use crate::StateMutability;
use crate::context::builder::type_factory::TypeFactory;
use crate::ods::sol::AddModOperation;
use crate::ods::sol::AddOperation;
use crate::ods::sol::AddrOfOperation;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::AndOperation;
use crate::ods::sol::ArrayLitOperation;
use crate::ods::sol::AssertOperation;
use crate::ods::sol::BalanceOperation;
use crate::ods::sol::BaseFeeOperation;
use crate::ods::sol::BlobBaseFeeOperation;
use crate::ods::sol::BlockNumberOperation;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CAddOperation;
use crate::ods::sol::CDivOperation;
use crate::ods::sol::CExpOperation;
use crate::ods::sol::CMulOperation;
use crate::ods::sol::CSubOperation;
use crate::ods::sol::CallOperation;
use crate::ods::sol::CallValueOperation;
use crate::ods::sol::CallerOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::ChainIdOperation;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::CodeHashOperation;
use crate::ods::sol::CodeOperation;
use crate::ods::sol::CoinbaseOperation;
use crate::ods::sol::ConditionOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ContinueOperation;
use crate::ods::sol::ContractCastOperation;
use crate::ods::sol::ContractOperation;
use crate::ods::sol::ConvCastOperation;
use crate::ods::sol::CopyOperation;
use crate::ods::sol::DataLocCastOperation;
use crate::ods::sol::DefaultFuncConstantOperation;
use crate::ods::sol::DeleteOperation;
use crate::ods::sol::DifficultyOperation;
use crate::ods::sol::DivOperation;
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::DynBytesToFixedBytesOperation;
use crate::ods::sol::EcrecoverOperation;
use crate::ods::sol::EnumCastOperation;
use crate::ods::sol::ExpOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::GasLimitOperation;
use crate::ods::sol::GasPriceOperation;
use crate::ods::sol::GepOperation;
use crate::ods::sol::GetCallDataOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::IfOperation;
use crate::ods::sol::Keccak256Operation;
use crate::ods::sol::LengthOperation;
use crate::ods::sol::LoadOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::MapOperation;
use crate::ods::sol::ModOperation;
use crate::ods::sol::MulModOperation;
use crate::ods::sol::MulOperation;
use crate::ods::sol::NotOperation;
use crate::ods::sol::OrOperation;
use crate::ods::sol::OriginOperation;
use crate::ods::sol::PopOperation;
use crate::ods::sol::PrevRandaoOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::PushStringOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::ReturnOperation;
use crate::ods::sol::RevertOperation;
use crate::ods::sol::Ripemd160Operation;
use crate::ods::sol::Sha256Operation;
use crate::ods::sol::ShlOperation;
use crate::ods::sol::ShrOperation;
use crate::ods::sol::SigOperation;
use crate::ods::sol::StateVarOperation;
use crate::ods::sol::StoreOperation;
use crate::ods::sol::StringLitOperation;
use crate::ods::sol::SubOperation;
use crate::ods::sol::TimestampOperation;
use crate::ods::sol::WhileOperation;
use crate::ods::sol::XorOperation;
use crate::ods::sol::YieldOperation;

/// Cached MLIR types and emission methods for building MLIR operations.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
    /// Type factory: pre-cached common types and parameterized constructors.
    pub types: TypeFactory<'context>,
    /// Monotonic counter assigning a unique non-zero `id` to each `sol.func`.
    /// The Sol→Yul lowering of `sol.func_constant` / `sol.icall` (internal
    /// function pointers) requires every pointer-target function to carry an
    /// `id`; assigning to all functions is harmless for the rest.
    function_id_counter: std::cell::Cell<i64>,
}

impl<'context> Builder<'context> {
    /// Creates a new builder with pre-cached types.
    pub fn new(context: &'context melior::Context) -> Self {
        Self {
            context,
            unknown_location: Location::unknown(context),
            types: TypeFactory::new(context),
            function_id_counter: std::cell::Cell::new(1),
        }
    }

    // ==== Structure ====

    /// Emits a `sol.contract` operation with a body region.
    ///
    /// Returns the body block inside the contract region for appending
    /// function definitions.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_contract<'block>(
        &self,
        name: &str,
        kind: crate::ContractKind,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        // SAFETY: `solxCreateContractKindAttr` returns a valid MlirAttribute.
        let kind_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateContractKindAttr(
                self.context.to_raw(),
                kind as u32,
            ))
        };

        block
            .append_operation(
                ContractOperation::builder(self.context, self.unknown_location)
                    .sym_name(StringAttribute::new(self.context, name))
                    .kind(kind_attribute)
                    .body_region(body_region)
                    .build()
                    .into(),
            )
            .region(0)
            .expect("contract has one region")
            .first_block()
            .expect("contract body has one block")
    }

    /// Emits a `sol.func` operation with the given name, parameter types,
    /// result types, selector, state mutability, and optional function kind.
    ///
    /// Returns the entry block of the function body for appending operations.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_func<'block>(
        &self,
        name: &str,
        parameter_types: &[Type<'context>],
        result_types: &[Type<'context>],
        selector: Option<u32>,
        state_mutability: StateMutability,
        kind: Option<crate::FunctionKind>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let function_type = FunctionType::new(self.context, parameter_types, result_types);
        let body_region = Region::new();
        let entry_block = Block::new(
            &parameter_types
                .iter()
                .map(|parameter_type| (*parameter_type, self.unknown_location))
                .collect::<Vec<_>>(),
        );
        body_region.append_block(entry_block);

        // SAFETY: `solxCreateStateMutabilityAttr` returns a valid MlirAttribute.
        let mutability_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateStateMutabilityAttr(
                self.context.to_raw(),
                state_mutability as u32,
            ))
        };

        let mut builder = FuncOperation::builder(self.context, self.unknown_location)
            .sym_name(StringAttribute::new(self.context, name))
            .function_type(TypeAttribute::new(function_type.into()))
            .state_mutability(mutability_attribute)
            .body(body_region);

        if let Some(function_kind) = kind {
            // SAFETY: `solxCreateFunctionKindAttr` returns a valid MlirAttribute.
            let kind_attribute = unsafe {
                Attribute::from_raw(crate::ffi::solxCreateFunctionKindAttr(
                    self.context.to_raw(),
                    function_kind as u32,
                ))
            };
            builder = builder.kind(kind_attribute);
        }

        if let Some(selector_value) = selector {
            builder = builder.selector(IntegerAttribute::new(
                IntegerType::new(self.context, TypeFactory::SELECTOR_BIT_WIDTH).into(),
                selector_value as i64,
            ));
        }

        // Preserve the pre-lowering (Sol-typed) signature: the Sol→Yul lowering
        // reads `orig_fn_type` after the live `function_type` has been converted
        // to lowered types. Set on every function — the constructor/external
        // dispatch and the fallback (which has no selector) all rely on it.
        builder = builder.orig_fn_type(TypeAttribute::new(function_type.into()));

        // Assign a unique non-zero id so the function can be the target of an
        // internal function pointer (`sol.func_constant` / `sol.icall`).
        let function_id = self.function_id_counter.get();
        self.function_id_counter.set(function_id + 1);
        builder = builder.id(IntegerAttribute::new(
            IntegerType::new(self.context, 64).into(),
            function_id,
        ));

        let operation = block.append_operation(builder.build().into());
        operation
            .region(0)
            .expect("func has one region")
            .first_block()
            .expect("func body has entry block")
    }

    // ==== Constants ====

    /// Emits a `sol.constant` of the given type.
    ///
    /// Use this variant when the constant type is known at emission time.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_constant<'block, B>(
        &self,
        value: i64,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ConstantOperation::builder(self.context, self.unknown_location)
                    .value(IntegerAttribute::new(result_type, value).into())
                    .result(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.constant always produces one result")
            .into()
    }

    /// Emits a typed integer constant, selecting the dialect by target type.
    ///
    /// `i1` is the signless boolean type owned by the arith dialect; every
    /// other integer type is signed or unsigned and belongs to the sol
    /// dialect. This is the single entry point for MLIR integer constants
    /// that carry a `BigInt`-sized value.
    pub fn emit_constant<'block, B>(
        &self,
        value: &BigInt,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if result_type == self.types.sol_address {
            let integer = self.emit_constant(value, self.types.ui160, block);
            return self.emit_sol_address_cast(integer, result_type, block);
        }
        if TypeFactory::integer_bit_width(result_type) == solx_utils::BIT_LENGTH_BOOLEAN as u32 {
            let boolean_attribute =
                IntegerAttribute::new(result_type, i64::from(*value != BigInt::ZERO)).into();
            return self
                .emit_constant_operation(boolean_attribute, result_type, block)
                .expect("well-typed boolean constant never fails emission");
        }
        let (sign, words) = value.to_u64_digits();
        let attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateIntegerAttr(
                result_type.to_raw(),
                sign == num::bigint::Sign::Minus,
                words.len(),
                words.as_ptr(),
            ))
        };
        self.emit_constant_operation(attribute, result_type, block)
            .expect("well-typed BigInt constant never fails emission")
    }

    /// Emits an `i1` boolean constant.
    pub fn emit_bool<'block, B>(&self, value: bool, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        self.emit_constant(&BigInt::from(u8::from(value)), self.types.i1, block)
    }

    // ==== String literals ====

    /// Emits a `sol.string_lit` constant with a `!sol.string<Memory>` result.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_string_lit<'block, B>(&self, value: &str, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        self.emit_sol_string_lit_bytes(value.as_bytes(), block)
    }

    /// Emits a `sol.string_lit` constant from raw bytes (result `!sol.string<Memory>`).
    ///
    /// `hex"…"`, escaped, and `\x..` string literals decode to arbitrary byte
    /// sequences that need not be valid UTF-8 (e.g. `hex"12_34_5678_9A"` ends
    /// in `0x9A`). MLIR's `StringAttr` stores its payload by length, so the
    /// bytes are carried through verbatim; routing through a `&str` and `char`
    /// conversion instead re-encodes every byte ≥ 0x80 as multi-byte UTF-8 and
    /// corrupts the value.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_string_lit_bytes<'block, B>(
        &self,
        bytes: &[u8],
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // SAFETY: the `&str` is only consumed by `StringAttribute::new`, which
        // hands it to `StringRef::new` — that reads `.as_bytes().as_ptr()` and
        // `.len()` and never assumes UTF-8 validity. No other code observes it
        // as a Rust string, so non-UTF-8 bytes are sound here.
        let value = unsafe { std::str::from_utf8_unchecked(bytes) };
        block
            .append_operation(
                StringLitOperation::builder(self.context, self.unknown_location)
                    .value(StringAttribute::new(self.context, value))
                    .addr(self.types.sol_string_memory)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.string_lit always produces one result")
            .into()
    }

    /// Emits a `sol.lib_addr` yielding the linked deploy address of the
    /// library identified by `name` — the fully-qualified `file:Library`
    /// linker symbol (matching solc), which the linker resolves at link time.
    ///
    /// Built generically because the op's `name` `StrAttr` collides with the
    /// melior builder's reserved `name`, so it has no generated setter.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_lib_addr<'block, B>(&self, name: &str, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                melior::ir::operation::OperationBuilder::new("sol.lib_addr", self.unknown_location)
                    .add_attributes(&[(
                        melior::ir::Identifier::new(self.context, "name"),
                        StringAttribute::new(self.context, name).into(),
                    )])
                    .add_results(&[self.types.sol_address])
                    .build()
                    .expect("valid sol.lib_addr"),
            )
            .result(0)
            .expect("sol.lib_addr produces one result")
            .into()
    }

    /// Emits a `sol.revert` carrying an optional payload.
    ///
    /// `signature` doubles as the payload string: for custom errors
    /// (`revert MyError(x, y)`) it is the canonical signature
    /// (`"MyError(uint256,address)"`) and the evaluated arguments are passed
    /// in `args` with `is_custom_error = true`. For string-message reverts
    /// (`revert("message")`) it is the literal message, with no `args` and
    /// `is_custom_error = false`. For plain `revert()` it is empty, with no
    /// `args` and `is_custom_error = false`.
    ///
    /// `sol.revert` does not carry the `IsTerminator` trait, so callers must
    /// ensure the enclosing block reaches a structural terminator through the
    /// normal codegen path (a following statement, a region yield, or the
    /// function-epilogue default return).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_revert<'block, B>(
        &self,
        signature: &str,
        args: &[Value<'context, 'block>],
        is_custom_error: bool,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut builder = RevertOperation::builder(self.context, self.unknown_location)
            .signature(StringAttribute::new(self.context, signature))
            .args(args);
        if is_custom_error {
            builder = builder.call(Attribute::unit(self.context));
        }
        block.append_operation(builder.build().into());
    }

    /// Emits a `sol.assert` panic if the condition is false.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_assert<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            AssertOperation::builder(self.context, self.unknown_location)
                .cond(condition)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.require` conditional revert with an optional message.
    ///
    /// Reverts if `condition` is false. When `msg` is `Some`, the revert
    /// includes the string as a revert reason. Not a terminator — execution
    /// continues after this op when the condition is true.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_require<'block, B>(
        &self,
        condition: Value<'context, 'block>,
        msg: Option<&str>,
        args: &[Value<'context, 'block>],
        is_call: bool,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut builder = RequireOperation::builder(self.context, self.unknown_location)
            .cond(condition)
            .args(args);
        if let Some(msg) = msg {
            builder = builder.msg(StringAttribute::new(self.context, msg));
        }
        if is_call {
            builder = builder.call(Attribute::unit(self.context));
        }
        block.append_operation(builder.build().into());
    }

    /// Emits a `sol.return` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_return<'block, B>(&self, operands: &[Value<'context, 'block>], block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            ReturnOperation::builder(self.context, self.unknown_location)
                .operands(operands)
                .build()
                .into(),
        );
    }

    // ==== Control flow ====

    /// Emits a `sol.if` with then and else regions.
    ///
    /// Returns `(then_block, else_block)`. The caller emits into each region
    /// and terminates them with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_if<'block>(
        &self,
        condition: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>)
    where
        'context: 'block,
    {
        let then_region = Region::new();
        let then_block = Block::new(&[]);
        then_region.append_block(then_block);

        let else_region = Region::new();
        let else_block = Block::new(&[]);
        else_region.append_block(else_block);

        let operation = block.append_operation(
            IfOperation::builder(self.context, self.unknown_location)
                .cond(condition)
                .then_region(then_region)
                .else_region(else_region)
                .build()
                .into(),
        );

        let then_ref = operation
            .region(0)
            .expect("sol.if has then region")
            .first_block()
            .expect("then region has a block");
        let else_ref = operation
            .region(1)
            .expect("sol.if has else region")
            .first_block()
            .expect("else region has a block");
        (then_ref, else_ref)
    }

    /// Emits a `sol.while` with condition and body regions.
    ///
    /// Returns `(cond_block, body_block)`. The condition region must be
    /// terminated with `sol.condition`. The body region with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_while<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let operation = block.append_operation(
            WhileOperation::builder(self.context, self.unknown_location)
                .cond(cond_region)
                .body(body_region)
                .build()
                .into(),
        );

        let cond_ref = operation
            .region(0)
            .expect("sol.while has cond region")
            .first_block()
            .expect("cond region has a block");
        let body_ref = operation
            .region(1)
            .expect("sol.while has body region")
            .first_block()
            .expect("body region has a block");
        (cond_ref, body_ref)
    }

    /// Emits a `sol.do` (do-while) with body and condition regions.
    ///
    /// Returns `(body_block, cond_block)`. The body executes first.
    /// Body terminates with `sol.yield`, condition with `sol.condition`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_do_while<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, BlockRef<'context, 'block>) {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let operation = block.append_operation(
            DoWhileOperation::builder(self.context, self.unknown_location)
                .body(body_region)
                .cond(cond_region)
                .build()
                .into(),
        );

        let body_ref = operation
            .region(0)
            .expect("sol.do has body region")
            .first_block()
            .expect("body region has a block");
        let cond_ref = operation
            .region(1)
            .expect("sol.do has cond region")
            .first_block()
            .expect("cond region has a block");
        (body_ref, cond_ref)
    }

    /// Emits a `sol.for` with condition, body, and step regions.
    ///
    /// Returns `(cond_block, body_block, step_block)`. Condition terminates
    /// with `sol.condition`, body and step with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_for<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
    ) {
        let cond_region = Region::new();
        let cond_block = Block::new(&[]);
        cond_region.append_block(cond_block);

        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        let step_region = Region::new();
        let step_block = Block::new(&[]);
        step_region.append_block(step_block);

        let operation = block.append_operation(
            ForOperation::builder(self.context, self.unknown_location)
                .cond(cond_region)
                .body(body_region)
                .step(step_region)
                .build()
                .into(),
        );

        let cond_ref = operation
            .region(0)
            .expect("sol.for has cond region")
            .first_block()
            .expect("cond region has a block");
        let body_ref = operation
            .region(1)
            .expect("sol.for has body region")
            .first_block()
            .expect("body region has a block");
        let step_ref = operation
            .region(2)
            .expect("sol.for has step region")
            .first_block()
            .expect("step region has a block");
        (cond_ref, body_ref, step_ref)
    }

    /// Emits a `sol.yield` region terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_yield<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            YieldOperation::builder(self.context, self.unknown_location)
                .ins(&[])
                .build()
                .into(),
        );
    }

    /// Emits a `sol.condition` loop condition terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_condition<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            ConditionOperation::builder(self.context, self.unknown_location)
                .condition(condition)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.break` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_break<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            BreakOperation::builder(self.context, self.unknown_location)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.continue` terminator.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_continue<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            ContinueOperation::builder(self.context, self.unknown_location)
                .build()
                .into(),
        );
    }

    // ==== Memory ====

    /// Emits a `sol.alloca` for a local variable, returning the pointer.
    ///
    /// Returns a `!sol.ptr<{element_type}, Stack>` pointer. Use this when
    /// the declared Solidity type is known (e.g. `uint64` → `ui64`).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR type or operation cannot be constructed, indicating
    /// a bug in the builder.
    pub fn emit_sol_alloca<'block, B>(
        &self,
        element_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let ptr_type = self
            .types
            .pointer(element_type, solx_utils::DataLocation::Stack);
        block
            .append_operation(
                AllocaOperation::builder(self.context, self.unknown_location)
                    .alloc_type(TypeAttribute::new(ptr_type))
                    .addr(ptr_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.alloca always produces one result")
            .into()
    }

    /// Emits a `sol.malloc` for an aggregate type, returning the address.
    ///
    /// Use for memory-located structs, arrays, bytes, and strings constructed
    /// via literals (e.g. `S(a, b)` struct construction).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_malloc<'block, B>(
        &self,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MallocOperation::builder(self.context, self.unknown_location)
                    .addr(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.malloc always produces one result")
            .into()
    }

    /// Emits a `sol.malloc` for a dynamically-sized aggregate (`new T[](n)`,
    /// `new bytes(n)`), passing the element count / byte length as the optional
    /// `size` operand so the allocation and length slot are set up correctly.
    pub fn emit_sol_malloc_sized<'block, B>(
        &self,
        result_type: Type<'context>,
        size: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MallocOperation::builder(self.context, self.unknown_location)
                    .addr(result_type)
                    .size(size)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.malloc always produces one result")
            .into()
    }

    /// Like [`Self::emit_sol_malloc`] but zero-initialises the allocation. Use
    /// for an aggregate whose contents are NOT immediately overwritten — an
    /// uninitialised fixed-size memory array / struct local or return value —
    /// where Solidity's default value requires the bytes to read as zero. The
    /// `zero_init` flag drives a memset in the backend allocator (a plain
    /// `sol.malloc` reuses dirty memory, e.g. left over from a `keccak`).
    pub fn emit_sol_malloc_zeroed<'block, B>(
        &self,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MallocOperation::builder(self.context, self.unknown_location)
                    .addr(result_type)
                    .zero_init(Attribute::unit(self.context))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.malloc always produces one result")
            .into()
    }

    /// Like [`Self::emit_sol_malloc_sized`] but zero-initialises the elements.
    /// `new T[](n)` / `new bytes(n)` allocate a fresh dynamic memory aggregate
    /// that Solidity guarantees is zeroed; a plain `sol.malloc` would expose
    /// dirty memory (e.g. a preceding mapping-key `keccak`).
    pub fn emit_sol_malloc_sized_zeroed<'block, B>(
        &self,
        result_type: Type<'context>,
        size: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MallocOperation::builder(self.context, self.unknown_location)
                    .addr(result_type)
                    .size(size)
                    .zero_init(Attribute::unit(self.context))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.malloc always produces one result")
            .into()
    }

    /// Emits a `sol.copy` between two references.
    ///
    /// Use for source-level assignments that cross data locations (e.g. a
    /// state-variable initializer copying a memory string literal into the
    /// declared storage slot).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_copy<'block, B>(
        &self,
        src: Value<'context, 'block>,
        dst: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            CopyOperation::builder(self.context, self.unknown_location)
                .src(src)
                .dst(dst)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.delete` that recursively clears the storage occupied by the
    /// reference-typed value at `reference` (Solidity `delete x` on an aggregate
    /// storage variable — array, struct, `bytes`/`string`).
    ///
    /// `reference` must be a `Storage` reference (e.g. the result of
    /// [`Self::emit_sol_addr_of`] for the aggregate state variable).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_delete<'block, B>(&self, reference: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            DeleteOperation::builder(self.context, self.unknown_location)
                .reference(reference)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.load` from a pointer with an explicit result type.
    ///
    /// Short-circuits when `address` is already the element (the gep result
    /// for reference-typed elements in `Storage`/`CallData`), returning it
    /// unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_sol_load<'block, B>(
        &self,
        address: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if address.r#type() == result_type {
            return Ok(address);
        }
        Ok(block
            .append_operation(
                LoadOperation::builder(self.context, self.unknown_location)
                    .addr(address)
                    .out(result_type)
                    .build()
                    .into(),
            )
            .result(0)?
            .into())
    }

    /// Emits a `sol.store` to a pointer.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug
    /// in the builder.
    pub fn emit_sol_store<'block, B>(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            StoreOperation::builder(self.context, self.unknown_location)
                .val(value)
                .addr(pointer)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.gep` for array / `bytes` / `string` / struct field
    /// access. `element_type` is the pointee the caller wants to address.
    /// The gep's result type is derived from `(base_address.r#type(),
    /// element_type)` via `GepOp::getResultType` on the C++ side.
    pub fn emit_sol_gep<'block, B>(
        &self,
        base_address: Value<'context, 'block>,
        index: Value<'context, 'block>,
        element_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // SAFETY: `mlirSolGepGetResultType` returns a valid MlirType from
        // `sol::GepOp::getResultType` on the C++ side.
        let address_type = unsafe {
            Type::from_raw(crate::ffi::mlirSolGepGetResultType(
                base_address.r#type().to_raw(),
                element_type.to_raw(),
            ))
        };
        block
            .append_operation(
                GepOperation::builder(self.context, self.unknown_location)
                    .base_addr(base_address)
                    .idx(index)
                    .addr(address_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gep always produces one result")
            .into()
    }

    /// Emits a `sol.map` for mapping value access by key.
    ///
    /// `address_type` is the result address type the caller has computed
    /// (typically `!sol.ptr<value, Storage>` for primitive value types).
    pub fn emit_sol_map<'block, B>(
        &self,
        mapping: Value<'context, 'block>,
        key: Value<'context, 'block>,
        address_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MapOperation::builder(self.context, self.unknown_location)
                    .mapping(mapping)
                    .key(key)
                    .addr(address_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.map always produces one result")
            .into()
    }

    /// Emits a `sol.push` returning a reference to the newly appended slot.
    ///
    /// `address_type` is the result reference type the caller has computed
    /// (typically `!sol.ptr<element, Storage>` for primitive element types).
    pub fn emit_sol_push<'block, B>(
        &self,
        array: Value<'context, 'block>,
        address_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                PushOperation::builder(self.context, self.unknown_location)
                    .inp(array)
                    .addr(address_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.push always produces one result")
            .into()
    }

    /// Emits `sol.length` yielding the element / byte count of an array,
    /// `bytes`, or `string` as a `ui256` (`x.length`).
    pub fn emit_sol_length<'block, B>(
        &self,
        value: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                LengthOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .len(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.length always produces one result")
            .into()
    }

    /// Emits a `sol.push_string` appending a byte `value` to a dynamic
    /// `bytes`/`string` (`bytes.push(x)`). Unlike `sol.push`, this handles the
    /// in-place → out-of-place storage-encoding transition at the 31-byte
    /// boundary, so it is the dedicated lowering for the single-argument
    /// `push` overload on byte arrays.
    pub fn emit_sol_push_string<'block, B>(
        &self,
        array: Value<'context, 'block>,
        value: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            PushStringOperation::builder(self.context, self.unknown_location)
                .addr(array)
                .value(value)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.pop` removing the last element from the array.
    pub fn emit_sol_pop<'block, B>(&self, array: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            PopOperation::builder(self.context, self.unknown_location)
                .inp(array)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.array_lit` constructing an array from `elements` of the
    /// caller-provided `array_type` (typically `!sol.array<N x T, Memory>`).
    pub fn emit_sol_array_lit<'block, B>(
        &self,
        elements: &[Value<'context, 'block>],
        array_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ArrayLitOperation::builder(self.context, self.unknown_location)
                    .ins(elements)
                    .addr(array_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.array_lit always produces one result")
            .into()
    }

    // ==== Calls ====

    /// Emits a `sol.call` operation and returns its first result value, or
    /// `None` if the callee is `void`. Use [`Self::emit_sol_call_results`]
    /// when all results are needed.
    ///
    /// # Errors
    ///
    /// Returns an error if the call operation result cannot be extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_call<'block, B>(
        &self,
        callee: &str,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        block: &B,
    ) -> anyhow::Result<Option<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let results = self.emit_sol_call_results(callee, operands, result_types, block)?;
        Ok(results.into_iter().next())
    }

    /// Emits a `sol.call` operation and returns all of its result values in
    /// declaration order. Use [`Self::emit_sol_call`] when only the first
    /// result is needed.
    ///
    /// # Errors
    ///
    /// Returns an error if any of the call operation results cannot be
    /// extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a
    /// bug in the builder.
    pub fn emit_sol_call_results<'block, B>(
        &self,
        callee: &str,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        block: &B,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(
            CallOperation::builder(self.context, self.unknown_location)
                .callee(FlatSymbolRefAttribute::new(self.context, callee))
                .outs(result_types)
                .operands(operands)
                .build()
                .into(),
        );
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index)?.into());
        }
        Ok(results)
    }

    /// Emits a `sol.func_constant` producing a reference to an internal
    /// function `name` with the given `func_ref` type.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_func_constant<'block, B>(
        &self,
        name: &str,
        func_ref_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                FuncConstantOperation::builder(self.context, self.unknown_location)
                    .addr(func_ref_type)
                    .sym(FlatSymbolRefAttribute::new(self.context, name))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.func_constant always produces one result")
            .into()
    }

    /// Emits a `sol.default_func_constant` — the zero/uninitialized value for
    /// an internal function pointer (calling it reverts).
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_default_func_constant<'block, B>(
        &self,
        func_ref_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                DefaultFuncConstantOperation::builder(self.context, self.unknown_location)
                    .addr(func_ref_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.default_func_constant always produces one result")
            .into()
    }

    /// Emits a `sol.icall` (indirect call through an internal function
    /// pointer) and returns its result values.
    ///
    /// # Errors
    ///
    /// Returns an error if a result cannot be retrieved.
    pub fn emit_sol_icall<'block, B>(
        &self,
        callee: Value<'context, 'block>,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        block: &B,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(
            ICallOperation::builder(self.context, self.unknown_location)
                .outs(result_types)
                .callee(callee)
                .callee_operands(operands)
                .build()
                .into(),
        );
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index)?.into());
        }
        Ok(results)
    }

    /// Emits a `sol.ext_func_constant` building an external function
    /// reference from an address value and a 4-byte selector.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_ext_func_constant<'block, B>(
        &self,
        address: Value<'context, 'block>,
        selector: u32,
        ext_func_ref_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ExtFuncConstantOperation::builder(self.context, self.unknown_location)
                    .addr(address)
                    .selector(IntegerAttribute::new(
                        IntegerType::new(self.context, TypeFactory::SELECTOR_BIT_WIDTH).into(),
                        selector as i64,
                    ))
                    .result(ext_func_ref_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.ext_func_constant always produces one result")
            .into()
    }

    /// Emits `sol.gasleft` yielding all remaining gas as a `ui256` — both the
    /// `gasleft()` built-in and the default gas an external call forwards
    /// without an explicit `{gas: ...}`.
    pub fn emit_sol_gas_left<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                GasLeftOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gas always produces one result")
            .into()
    }

    /// Emits a `sol.ext_icall` (external call through an external function
    /// reference), forwarding all remaining gas and the given `value`.
    /// Returns the decoded result values.
    ///
    /// # Errors
    ///
    /// Returns an error if a result cannot be retrieved.
    pub fn emit_sol_ext_icall<'block, B>(
        &self,
        callee: Value<'context, 'block>,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        value: Value<'context, 'block>,
        static_call: bool,
        block: &B,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // Forward all remaining gas (`gas()` / `gasleft()`), the default for
        // an external call without an explicit `{gas: ...}` option.
        let gas: Value<'context, 'block> = self.emit_sol_gas_left(block);
        // `sol.ext_icall` results are `(i1 status, decoded-returns...)`. We
        // prepend the status type and drop it from the values we hand back —
        // a non-try call reverts internally on failure, so the status is
        // always true here.
        let mut out_types = Vec::with_capacity(result_types.len() + 1);
        out_types.push(self.types.i1);
        out_types.extend_from_slice(result_types);
        // A call to a `view`/`pure` function lowers to `STATICCALL`, which
        // reverts if the callee attempts a state change (matching solc).
        let mut operation_builder = ExtICallOperation::builder(self.context, self.unknown_location)
            .outs(&out_types)
            .callee(callee)
            .callee_operands(operands)
            .gas(gas)
            .value(value);
        if static_call {
            operation_builder = operation_builder.static_call(Attribute::unit(self.context));
        }
        let operation = block.append_operation(operation_builder.build().into());
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index + 1)?.into());
        }
        Ok(results)
    }

    /// Emits a `sol.ext_icall` with `try_call` set, used to lower
    /// `try expr { ... } catch { ... }`. Returns `(status, results)` where
    /// `status` is the i1 success flag (false on revert, no auto-revert) and
    /// `results` are the decoded return values (valid only when `status`).
    ///
    /// # Errors
    ///
    /// Returns an error if a result cannot be retrieved.
    pub fn emit_sol_ext_icall_try<'block, B>(
        &self,
        callee: Value<'context, 'block>,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        value: Value<'context, 'block>,
        block: &B,
    ) -> anyhow::Result<(Value<'context, 'block>, Vec<Value<'context, 'block>>)>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let gas: Value<'context, 'block> = self.emit_sol_gas_left(block);
        let mut out_types = Vec::with_capacity(result_types.len() + 1);
        out_types.push(self.types.i1);
        out_types.extend_from_slice(result_types);
        let operation = block.append_operation(
            ExtICallOperation::builder(self.context, self.unknown_location)
                .outs(&out_types)
                .callee(callee)
                .callee_operands(operands)
                .gas(gas)
                .value(value)
                .try_call(Attribute::unit(self.context))
                .build()
                .into(),
        );
        let status: Value<'context, 'block> = operation
            .result(0)
            .expect("sol.ext_icall try produces a status result")
            .into();
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index + 1)?.into());
        }
        Ok((status, results))
    }

    // ==== Comparisons ====

    /// Emits a `sol.cmp` comparison returning `i1`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_cmp<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        predicate: CmpPredicate,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let predicate_attribute = IntegerAttribute::new(
            IntegerType::new(self.context, solx_utils::BIT_LENGTH_X64 as u32).into(),
            predicate as i64,
        );
        block
            .append_operation(
                CmpOperation::builder(self.context, self.unknown_location)
                    .predicate(predicate_attribute.into())
                    .lhs(lhs)
                    .rhs(rhs)
                    .result(self.types.i1)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cmp always produces one result")
            .into()
    }

    // ==== Arithmetic ====

    /// Emits a checked addition `sol.cadd` (reverts on overflow).
    pub fn emit_sol_cadd<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CAddOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cadd always produces one result")
            .into()
    }

    /// Emits a wrapping addition `sol.add`.
    pub fn emit_sol_add<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                AddOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.add always produces one result")
            .into()
    }

    /// Emits a checked subtraction `sol.csub` (reverts on underflow).
    pub fn emit_sol_csub<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CSubOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.csub always produces one result")
            .into()
    }

    /// Emits a wrapping subtraction `sol.sub`.
    pub fn emit_sol_sub<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                SubOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.sub always produces one result")
            .into()
    }

    /// Emits a checked multiplication `sol.cmul` (reverts on overflow).
    pub fn emit_sol_cmul<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CMulOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cmul always produces one result")
            .into()
    }

    /// Emits a wrapping multiplication `sol.mul`.
    pub fn emit_sol_mul<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MulOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.mul always produces one result")
            .into()
    }

    /// Emits a checked division `sol.cdiv` (reverts on division by zero).
    pub fn emit_sol_cdiv<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CDivOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cdiv always produces one result")
            .into()
    }

    /// Emits a wrapping division `sol.div`.
    pub fn emit_sol_div<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                DivOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.div always produces one result")
            .into()
    }

    /// Emits a modulo `sol.mod` (no checked variant exists).
    pub fn emit_sol_mod<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ModOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.mod always produces one result")
            .into()
    }

    /// Emits a checked exponentiation `sol.cexp` (reverts on overflow).
    pub fn emit_sol_cexp<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CExpOperation::builder(self.context, self.unknown_location)
                    .result(lhs.r#type())
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cexp always produces one result")
            .into()
    }

    /// Emits a wrapping exponentiation `sol.exp`.
    pub fn emit_sol_exp<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ExpOperation::builder(self.context, self.unknown_location)
                    .result(lhs.r#type())
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.exp always produces one result")
            .into()
    }

    // ==== Bitwise ====

    /// Emits a bitwise and `sol.and`.
    pub fn emit_sol_and<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                AndOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.and always produces one result")
            .into()
    }

    /// Emits a bitwise or `sol.or`.
    pub fn emit_sol_or<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OrOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.or always produces one result")
            .into()
    }

    /// Emits a bitwise xor `sol.xor`.
    pub fn emit_sol_xor<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                XorOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.xor always produces one result")
            .into()
    }

    /// Emits a left shift `sol.shl`.
    pub fn emit_sol_shl<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ShlOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.shl always produces one result")
            .into()
    }

    /// Emits a right shift `sol.shr` (arithmetic for signed operands).
    pub fn emit_sol_shr<'block, B>(
        &self,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ShrOperation::builder(self.context, self.unknown_location)
                    .lhs(lhs)
                    .rhs(rhs)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.shr always produces one result")
            .into()
    }

    /// Emits a bitwise not `sol.not`.
    pub fn emit_sol_not<'block, B>(
        &self,
        value: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                NotOperation::builder(self.context, self.unknown_location)
                    .value(value)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.not always produces one result")
            .into()
    }

    // ==== Precompiles ====

    /// Emits `sol.keccak256` over a memory buffer, yielding a `fixedbytes<32>`.
    pub fn emit_sol_keccak256<'block, B>(
        &self,
        address: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                Keccak256Operation::builder(self.context, self.unknown_location)
                    .addr(address)
                    .result(self.types.fixed_bytes(32))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.keccak256 always produces one result")
            .into()
    }

    /// Emits the `sol.sha256` precompile over a memory buffer (`fixedbytes<32>`).
    pub fn emit_sol_sha256<'block, B>(
        &self,
        data: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                Sha256Operation::builder(self.context, self.unknown_location)
                    .data(data)
                    .result(self.types.fixed_bytes(32))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.sha256 always produces one result")
            .into()
    }

    /// Emits the `sol.ripemd160` precompile over a memory buffer
    /// (`fixedbytes<20>`).
    pub fn emit_sol_ripemd160<'block, B>(
        &self,
        data: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                Ripemd160Operation::builder(self.context, self.unknown_location)
                    .data(data)
                    .result(self.types.fixed_bytes(20))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.ripemd160 always produces one result")
            .into()
    }

    /// Emits the `sol.ecrecover` precompile, yielding the signer `address`.
    pub fn emit_sol_ecrecover<'block, B>(
        &self,
        hash: Value<'context, 'block>,
        v: Value<'context, 'block>,
        r: Value<'context, 'block>,
        s: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                EcrecoverOperation::builder(self.context, self.unknown_location)
                    .hash(hash)
                    .v(v)
                    .r(r)
                    .s(s)
                    .result(self.types.sol_address)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.ecrecover always produces one result")
            .into()
    }

    /// Emits `sol.addmod` (`(x + y) % m` without intermediate overflow).
    pub fn emit_sol_addmod<'block, B>(
        &self,
        x: Value<'context, 'block>,
        y: Value<'context, 'block>,
        modulus: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                AddModOperation::builder(self.context, self.unknown_location)
                    .x(x)
                    .y(y)
                    .r#mod(modulus)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.addmod always produces one result")
            .into()
    }

    /// Emits `sol.mulmod` (`(x * y) % m` without intermediate overflow).
    pub fn emit_sol_mulmod<'block, B>(
        &self,
        x: Value<'context, 'block>,
        y: Value<'context, 'block>,
        modulus: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                MulModOperation::builder(self.context, self.unknown_location)
                    .x(x)
                    .y(y)
                    .r#mod(modulus)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.mulmod always produces one result")
            .into()
    }

    // ==== Environment globals ====

    /// Emits `sol.caller` (`msg.sender`), the immediate caller's address.
    pub fn emit_sol_caller<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CallerOperation::builder(self.context, self.unknown_location)
                    .addr(self.types.sol_address)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.caller always produces one result")
            .into()
    }

    /// Emits `sol.callvalue` (`msg.value`), the wei sent with the current call.
    pub fn emit_sol_call_value<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CallValueOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.callvalue always produces one result")
            .into()
    }

    /// Emits `sol.sig` (`msg.sig`), the first four calldata bytes (`bytes4`).
    pub fn emit_sol_sig<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                SigOperation::builder(self.context, self.unknown_location)
                    .val(self.types.fixed_bytes(4))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.sig always produces one result")
            .into()
    }

    /// Emits `sol.get_calldata` (`msg.data`), the full calldata buffer.
    pub fn emit_sol_call_data<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                GetCallDataOperation::builder(self.context, self.unknown_location)
                    .addr(self.types.string(solx_utils::DataLocation::CallData))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.get_calldata always produces one result")
            .into()
    }

    /// Emits `sol.origin` (`tx.origin`), the transaction's original sender.
    pub fn emit_sol_origin<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                OriginOperation::builder(self.context, self.unknown_location)
                    .addr(self.types.sol_address)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.origin always produces one result")
            .into()
    }

    /// Emits `sol.gasprice` (`tx.gasprice`), the transaction's gas price.
    pub fn emit_sol_gas_price<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                GasPriceOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gasprice always produces one result")
            .into()
    }

    /// Emits `sol.timestamp` (`block.timestamp`), the current block's time.
    pub fn emit_sol_timestamp<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                TimestampOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.timestamp always produces one result")
            .into()
    }

    /// Emits `sol.blocknumber` (`block.number`), the current block number.
    pub fn emit_sol_block_number<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                BlockNumberOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.blocknumber always produces one result")
            .into()
    }

    /// Emits `sol.coinbase` (`block.coinbase`), the current block miner.
    pub fn emit_sol_coinbase<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CoinbaseOperation::builder(self.context, self.unknown_location)
                    .addr(self.types.sol_address)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.coinbase always produces one result")
            .into()
    }

    /// Emits `sol.chainid` (`block.chainid`), the current chain identifier.
    pub fn emit_sol_chain_id<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ChainIdOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.chainid always produces one result")
            .into()
    }

    /// Emits `sol.basefee` (`block.basefee`), the current block's base fee.
    pub fn emit_sol_base_fee<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                BaseFeeOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.basefee always produces one result")
            .into()
    }

    /// Emits `sol.gaslimit` (`block.gaslimit`), the current block's gas limit.
    pub fn emit_sol_gas_limit<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                GasLimitOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gaslimit always produces one result")
            .into()
    }

    /// Emits `sol.blobbasefee` (`block.blobbasefee`), the block's blob base fee.
    pub fn emit_sol_blob_base_fee<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                BlobBaseFeeOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.blobbasefee always produces one result")
            .into()
    }

    /// Emits `sol.difficulty` (`block.difficulty`), a deprecated alias for
    /// `block.prevrandao`.
    pub fn emit_sol_difficulty<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                DifficultyOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.difficulty always produces one result")
            .into()
    }

    /// Emits `sol.prevrandao` (`block.prevrandao`), the beacon-chain randomness.
    pub fn emit_sol_prev_randao<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                PrevRandaoOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.prevrandao always produces one result")
            .into()
    }

    /// Emits `sol.balance` (`address.balance`), the wei balance of `address`.
    pub fn emit_sol_balance<'block, B>(
        &self,
        address: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                BalanceOperation::builder(self.context, self.unknown_location)
                    .cont_addr(address)
                    .out(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.balance always produces one result")
            .into()
    }

    /// Emits `sol.code_hash` (`address.codehash`), the code hash of `address`.
    pub fn emit_sol_code_hash<'block, B>(
        &self,
        address: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CodeHashOperation::builder(self.context, self.unknown_location)
                    .cont_addr(address)
                    .out(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.code_hash always produces one result")
            .into()
    }

    /// Emits `sol.code` (`address.code`), the deployed bytecode of `address`
    /// as a `bytes memory` value.
    pub fn emit_sol_code<'block, B>(
        &self,
        address: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                CodeOperation::builder(self.context, self.unknown_location)
                    .cont_addr(address)
                    .out(self.types.sol_string_memory)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.code always produces one result")
            .into()
    }

    /// Emits a `sol.cast` to an arbitrary target type.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        // `sol.cast` is integer-only; its verifier rejects enum, address,
        // contract, and fixedbytes operands/results. Each of those belongs to a
        // dedicated cast op. Route here — centrally — so every caller (event and
        // ABI encoders, comparisons, value transfers, explicit conversions) gets
        // the right op without repeating the dispatch. No caller can rely on
        // `sol.cast` accepting these types, since the verifier rejects them.
        let is_enum = TypeFactory::is_sol_enum;
        let is_address = TypeFactory::is_sol_address;
        let is_contract = TypeFactory::is_sol_contract;
        let is_fixed_bytes = TypeFactory::is_sol_fixed_bytes;
        let is_byte = TypeFactory::is_sol_byte;
        let src = value.r#type();

        // Enum ↔ integer (`sol.enum_cast` accepts `Sol_Int`, which includes
        // `!sol.enum<N>`); narrowing to an enum range-checks (and may revert).
        if is_enum(src) || is_enum(to_type) {
            return self.emit_sol_enum_cast(value, to_type, block);
        }
        // Contract ↔ contract (inheritance up/downcast, interface) uses the
        // dedicated `sol.contract_cast`; `sol.address_cast` rejects two distinct
        // contract endpoints.
        if is_contract(src) && is_contract(to_type) {
            return self.emit_sol_contract_cast(value, to_type, block);
        }
        // address ↔ {integer, contract, fixedbytes<20>}. `sol.address_cast`
        // requires the integer side to be exactly `ui160`, so a wider/narrower
        // integer bridges through `ui160` (then a plain `sol.cast` resizes it).
        // `contract` and `fixedbytes<20>` endpoints cast directly.
        if is_address(src) || is_address(to_type) {
            let ui160 = self.types.ui160;
            if is_address(src) {
                if is_contract(to_type) || is_fixed_bytes(to_type) || to_type == ui160 {
                    return self.emit_sol_address_cast(value, to_type, block);
                }
                let as_160 = self.emit_sol_address_cast(value, ui160, block);
                return self.emit_sol_cast(as_160, to_type, block);
            }
            if is_contract(src) || is_fixed_bytes(src) || src == ui160 {
                return self.emit_sol_address_cast(value, to_type, block);
            }
            let as_160 = self.emit_sol_cast(value, ui160, block);
            return self.emit_sol_address_cast(as_160, to_type, block);
        }
        // Dynamic `bytes`/`string` → `bytesN`: take the leading N bytes via the
        // dedicated op. `sol.bytes_cast` is integer/byte/fixedbytes-only and
        // rejects a dynamic-bytes (`!sol.string`) operand, so route it here first.
        if TypeFactory::is_sol_reference(src) && is_fixed_bytes(to_type) {
            return block
                .append_operation(
                    DynBytesToFixedBytesOperation::builder(self.context, self.unknown_location)
                        .inp(value)
                        .out(to_type)
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.dyn_bytes_to_fixedbytes produces one result")
                .into();
        }
        // byte / bytesN ↔ {byte, bytesN, integer}.
        if is_fixed_bytes(src) || is_fixed_bytes(to_type) || is_byte(src) || is_byte(to_type) {
            return self.emit_sol_bytes_cast(value, to_type, block);
        }
        // Reference types (array / struct / string / bytes / mapping) differ
        // only by data location; a reference→reference cast routes through
        // `sol.data_loc_cast`.
        if TypeFactory::is_sol_reference(src) && TypeFactory::is_sol_reference(to_type) {
            return self.emit_sol_data_loc_cast(value, to_type, block);
        }
        block
            .append_operation(
                CastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.cast always produces one result")
            .into()
    }

    /// Emits a `sol.enum_cast` between an integer and an enum type (either
    /// direction). Returns the input unchanged when the types already match.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_enum_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        block
            .append_operation(
                EnumCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.enum_cast always produces one result")
            .into()
    }

    /// Emits a `sol.contract_cast` between two contract types (inheritance
    /// up/downcast or interface). Returns the input unchanged when the types
    /// already match.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_contract_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        block
            .append_operation(
                ContractCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.contract_cast always produces one result")
            .into()
    }

    /// Emits a `sol.data_loc_cast` converting a reference-typed value between
    /// data locations (e.g. a storage array to a memory array). Returns the
    /// input unchanged when the types already match.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_data_loc_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        block
            .append_operation(
                DataLocCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.data_loc_cast always produces one result")
            .into()
    }

    /// Emits a `sol.bytes_cast` between byte / fixedbytes / integer types.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_bytes_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        // `fixedbytes<N>` and a full 256-bit machine word share the same
        // representation, but `sol.bytes_cast` only accepts the width-matched
        // pair `fixedbytes<N>` ↔ `ui(8*N)`. When a `bytesN` value flows to or
        // from a full 256-bit word (inline-assembly reads/writes, ABI heads,
        // storage words), reinterpret its raw left-aligned representation via
        // `sol.conv_cast` rather than shifting it as a value conversion.
        let src = value.r#type();
        let is_word256 =
            |ty: Type<'context>| IntegerType::try_from(ty).is_ok_and(|int| int.width() == 256);
        let bytes_into_word = TypeFactory::fixed_bytes_width(src).is_some_and(|width| width != 32)
            && is_word256(to_type);
        let word_into_bytes = is_word256(src)
            && TypeFactory::fixed_bytes_width(to_type).is_some_and(|width| width != 32);
        if bytes_into_word || word_into_bytes {
            return self.emit_sol_conv_cast(value, to_type, block);
        }
        block
            .append_operation(
                BytesCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.bytes_cast always produces one result")
            .into()
    }

    /// Emits a `sol.conv_cast` — a no-op reinterpret between types that share
    /// the same machine representation (e.g. a `bytesN` value and the full
    /// 256-bit EVM stack word). Unlike [`Self::emit_sol_bytes_cast`] it applies
    /// no shift, preserving the value's native alignment.
    pub fn emit_sol_conv_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        if value.r#type() == to_type {
            return value;
        }
        block
            .append_operation(
                ConvCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.conv_cast always produces one result")
            .into()
    }

    /// Emits a `sol.address_cast` to convert between address and integer types.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_address_cast<'block, B>(
        &self,
        value: Value<'context, 'block>,
        to_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                AddressCastOperation::builder(self.context, self.unknown_location)
                    .inp(value)
                    .out(to_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.address_cast always produces one result")
            .into()
    }

    // ==== State variables ====

    /// Emits a `sol.state_var` declaration inside a contract body.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_state_var<'block, B>(
        &self,
        name: &str,
        slot: U256,
        byte_offset: u32,
        element_type: Type<'context>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let slot_attribute: IntegerAttribute =
            Attribute::parse(self.context, &format!("{slot} : i256"))
                .expect("valid slot literal")
                .try_into()
                .expect("slot literal is an integer attribute");
        let byte_offset_attribute = IntegerAttribute::new(
            IntegerType::new(self.context, solx_utils::BIT_LENGTH_X32 as u32).into(),
            byte_offset as i64,
        );
        block.append_operation(
            StateVarOperation::builder(self.context, self.unknown_location)
                .sym_name(StringAttribute::new(self.context, name))
                .r#type(TypeAttribute::new(element_type))
                .slot(slot_attribute)
                .byte_offset(byte_offset_attribute)
                .build()
                .into(),
        );
    }

    /// Emits a `sol.addr_of` returning a `!sol.ptr<ui256, Storage>`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_addr_of<'block, B>(
        &self,
        name: &str,
        result_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                AddrOfOperation::builder(self.context, self.unknown_location)
                    .var(FlatSymbolRefAttribute::new(self.context, name))
                    .addr(result_type)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.addr_of always produces one result")
            .into()
    }

    // ==== Shared helpers ====

    /// Shared helper for emitting a constant operation with an attribute.
    ///
    /// # Errors
    ///
    /// Returns an error if the MLIR operation cannot be constructed.
    fn emit_constant_operation<'block, B>(
        &self,
        attribute: Attribute<'context>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                ConstantOperation::builder(self.context, self.unknown_location)
                    .value(attribute)
                    .result(result_type)
                    .build()
                    .into(),
            )
            .result(0)?
            .into())
    }
}
