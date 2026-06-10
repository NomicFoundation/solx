//!
//! MLIR builder for Sol dialect emission.
//!
//! Contains the [`Builder`] type with cached MLIR types and emission methods
//! for Sol dialect operations: contracts, functions, constants, control flow,
//! memory, comparisons, calls, state variables, and EVM context intrinsics.
//!

pub mod try_fallback_kind;
pub mod type_factory;
pub mod yul;

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Identifier;
use melior::ir::Location;
use melior::ir::Operation;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::ValueLike;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationBuilder;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::TypeLike;
use num::BigInt;
use ruint::aliases::U256;

use crate::CmpPredicate;
use crate::StateMutability;
use crate::context::builder::type_factory::TypeFactory;
use crate::ods::sol::AddrOfOperation;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::ArrayLitOperation;
use crate::ods::sol::AssertOperation;
use crate::ods::sol::BareCallOperation;
use crate::ods::sol::BareDelegateCallOperation;
use crate::ods::sol::BareStaticCallOperation;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CallOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::CmpOperation;
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
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::DynBytesToFixedBytesOperation;
use crate::ods::sol::EnumCastOperation;
use crate::ods::sol::ExtCallOperation;
use crate::ods::sol::ExtFuncAddrOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtFuncSelectorOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::GepOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::IfOperation;
use crate::ods::sol::LoadOperation;
use crate::ods::sol::MallocOperation;
use crate::ods::sol::MapOperation;
use crate::ods::sol::PopOperation;
use crate::ods::sol::PushOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::ReturnOperation;
use crate::ods::sol::RevertOperation;
use crate::ods::sol::StateVarOperation;
use crate::ods::sol::StoreOperation;
use crate::ods::sol::StringLitOperation;
use crate::ods::sol::TryOperation;
use crate::ods::sol::WhileOperation;
use crate::ods::sol::YieldOperation;

use crate::context::builder::try_fallback_kind::TryFallbackKind;

/// Cached MLIR types and emission methods for building MLIR operations.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
    /// Type factory: pre-cached common types and parameterized constructors.
    pub types: TypeFactory<'context>,
}

impl<'context> Builder<'context> {
    /// Creates a new builder with pre-cached types.
    pub fn new(context: &'context melior::Context) -> Self {
        Self {
            context,
            unknown_location: Location::unknown(context),
            types: TypeFactory::new(context),
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
        id: Option<i64>,
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

        // An internal function pointer (`sol.func_constant`) lowers in SolToYul
        // to the i256 constant `id`, and the `sol.icall` dispatch switches over
        // every same-signature function's `id`; both read this attribute, so a
        // referenceable function must carry a unique `id` (its slang node id).
        if let Some(function_id) = id {
            builder = builder.id(IntegerAttribute::new(
                IntegerType::new(self.context, 64).into(),
                function_id,
            ));
        }

        // The fallback dispatcher in SolToYul reads `orig_fn_type` to recover the
        // pre-lowering Sol signature, so a fallback (like a selector-bearing
        // function or the constructor) must carry it; without it the pass
        // dereferences a null type.
        if selector.is_some()
            || matches!(
                kind,
                Some(crate::FunctionKind::Constructor | crate::FunctionKind::Fallback)
            )
        {
            builder = builder.orig_fn_type(TypeAttribute::new(function_type.into()));
        }

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

    /// Emits a `sol.string_lit` from raw `bytes` that need not be valid UTF-8 (a
    /// `hex"..."` literal, an escaped `"\xff"`). The `value` attribute is built
    /// from the bytes via the C API (melior's `StringAttribute::new` takes a
    /// UTF-8 `&str`), and the op via the raw `OperationBuilder` because the
    /// generated `.value()` setter requires a `StringAttribute`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_string_lit_bytes<'block, B>(
        &self,
        bytes: &[u8],
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // SAFETY: `solxCreateStringAttr` builds a `StringAttr` from the pointer
        // and length of the live byte slice.
        let value_attribute = unsafe {
            Attribute::from_raw(crate::ffi::solxCreateStringAttr(
                self.context.to_raw(),
                bytes.as_ptr(),
                bytes.len(),
            ))
        };
        block
            .append_operation(
                OperationBuilder::new("sol.string_lit", self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), value_attribute)])
                    .add_results(&[self.types.sol_string_memory])
                    .build()
                    .expect("valid sol.string_lit"),
            )
            .result(0)
            .expect("sol.string_lit always produces one result")
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

    /// Emits a `sol.try` carrying the external call's success `status` and four
    /// regions — success, panic, error, fallback. A clause that is absent
    /// produces an empty region; the op's lowering performs the returndata-size
    /// guard, the selector switch over `Error(string)` / `Panic(uint256)`, the
    /// payload decode (delivered as each region's block argument), and the raw
    /// re-revert when no clause matches, so the frontend emits no returndata or
    /// selector ops itself.
    ///
    /// Returns `(success, panic, error, fallback)` entry blocks; the three catch
    /// blocks are `Some` exactly when their clause is present (an absent clause
    /// left an empty region). The panic block carries the decoded panic code
    /// (`ui256`), the error block the decoded reason (`string<Memory>`), and a
    /// [`TryFallbackKind::Bytes`] fallback block the raw returndata
    /// (`string<Memory>`), each as block argument 0. The caller binds those,
    /// emits each body, and terminates every region with `sol.yield`.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_try<'block>(
        &self,
        status: Value<'context, 'block>,
        has_panic: bool,
        has_error: bool,
        fallback: TryFallbackKind,
        block: &BlockRef<'context, 'block>,
    ) -> (
        BlockRef<'context, 'block>,
        Option<BlockRef<'context, 'block>>,
        Option<BlockRef<'context, 'block>>,
        Option<BlockRef<'context, 'block>>,
    )
    where
        'context: 'block,
    {
        let success_region = Region::new();
        success_region.append_block(Block::new(&[]));

        let panic_region = Region::new();
        if has_panic {
            panic_region.append_block(Block::new(&[(self.types.ui256, self.unknown_location)]));
        }

        let error_region = Region::new();
        if has_error {
            error_region.append_block(Block::new(&[(
                self.types.sol_string_memory,
                self.unknown_location,
            )]));
        }

        let fallback_region = Region::new();
        match fallback {
            TryFallbackKind::None => {}
            TryFallbackKind::Parameterless => {
                fallback_region.append_block(Block::new(&[]));
            }
            TryFallbackKind::Bytes => {
                fallback_region.append_block(Block::new(&[(
                    self.types.sol_string_memory,
                    self.unknown_location,
                )]));
            }
        }

        let operation = block.append_operation(
            TryOperation::builder(self.context, self.unknown_location)
                .status(status)
                .success_region(success_region)
                .panic_region(panic_region)
                .error_region(error_region)
                .fallback_region(fallback_region)
                .build()
                .into(),
        );

        let success = operation
            .region(0)
            .expect("sol.try has a success region")
            .first_block()
            .expect("success region has a block");
        let panic = has_panic.then(|| {
            operation
                .region(1)
                .expect("sol.try has a panic region")
                .first_block()
                .expect("panic region has a block")
        });
        let error = has_error.then(|| {
            operation
                .region(2)
                .expect("sol.try has an error region")
                .first_block()
                .expect("error region has a block")
        });
        let fallback = (!matches!(fallback, TryFallbackKind::None)).then(|| {
            operation
                .region(3)
                .expect("sol.try has a fallback region")
                .first_block()
                .expect("fallback region has a block")
        });

        (success, panic, error, fallback)
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

    /// Allocates a stack slot for `element_type` and zero-initialises it,
    /// returning the slot address.
    ///
    /// An integer type is initialised with a typed `0`. A non-integer type is a
    /// LOUD `unimplemented!`: silently leaving the slot uninitialised would be a
    /// bug-masking fallback.
    ///
    /// # Panics
    ///
    /// Panics on a non-integer `element_type` — zero-initialisation of address,
    /// fixed-bytes, and memory-resident types is not yet supported.
    pub fn emit_zero_initialized_alloca<'block, B>(
        &self,
        element_type: Type<'context>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let pointer = self.emit_sol_alloca(element_type, block);
        if IntegerType::try_from(element_type).is_ok() {
            let zero = self.emit_sol_constant(0, element_type, block);
            self.emit_sol_store(zero, pointer, block);
        } else {
            unimplemented!("zero-initialization for non-integer type {element_type}");
        }
        pointer
    }

    /// Emits a `sol.return` whose operands are loaded from the per-return slots:
    /// each named-return slot is loaded, and a typed zero is materialised where
    /// no slot was allocated (an unnamed return). Shared by the implicit
    /// end-of-body return and an explicit bare `return;`.
    pub fn emit_return_from_slots<'block, B>(
        &self,
        result_types: &[Type<'context>],
        return_slots: &[Option<Value<'context, 'block>>],
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let mut values: Vec<Value<'context, 'block>> = Vec::with_capacity(result_types.len());
        for (index, result_type) in result_types.iter().enumerate() {
            let value = match return_slots.get(index).copied().flatten() {
                Some(pointer) => self
                    .emit_sol_load(pointer, *result_type, block)
                    .expect("named return slot loads with the declared type"),
                None => self.emit_sol_constant(0, *result_type, block),
            };
            values.push(value);
        }
        self.emit_sol_return(&values, block);
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

    /// Emits a `sol.malloc` of a fixed aggregate, zero-initialising it — the
    /// default value of a freshly-allocated aggregate.
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

    /// Emits a `sol.malloc` of a dynamic aggregate of `size` elements/bytes,
    /// zero-initialised — `new T[](n)` / `new bytes(n)` / `new string(n)`.
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

    /// Emits a `sol.push_string` appending the byte `value` to the `bytes`
    /// reference `address` in place. Unlike `sol.push`, the value is passed
    /// directly and the op yields no slot reference (it has no result), since a
    /// packed `bytes` element is not separately addressable. Built via the raw
    /// `OperationBuilder` because the op has no generated ODS binding yet.
    pub fn emit_sol_push_string<'block, B>(
        &self,
        address: Value<'context, 'block>,
        value: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            OperationBuilder::new("sol.push_string", self.unknown_location)
                .add_operands(&[address, value])
                .build()
                .expect("valid sol.push_string"),
        );
    }

    /// Emits a `sol.delete` recursively clearing the reference-typed storage
    /// aggregate at `reference` to its zero value (`delete x` for arrays,
    /// strings/bytes, and structs). The op's lowering performs the deep clear.
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
        // contract, and fixedbytes operands/results — each belongs to a
        // dedicated cast op. Dispatch by type kind here, centrally, so every
        // caller (event/ABI encoders, comparisons, value transfers, explicit
        // conversions) gets the right op without repeating the classification.
        let source = value.r#type();
        // Enum ↔ integer (`sol.enum_cast` accepts the integer-backed enum;
        // narrowing to an enum range-checks and may revert).
        if TypeFactory::is_sol_enum(source) || TypeFactory::is_sol_enum(to_type) {
            return self.emit_sol_enum_cast(value, to_type, block);
        }
        // Contract ↔ contract (inheritance up/downcast, interface).
        if TypeFactory::is_sol_contract(source) && TypeFactory::is_sol_contract(to_type) {
            return self.emit_sol_contract_cast(value, to_type, block);
        }
        // address ↔ {integer, contract, fixedbytes<20>}. `sol.address_cast`
        // requires the integer side to be exactly `ui160`, so a wider/narrower
        // integer bridges through `ui160` (then a plain `sol.cast` resizes it).
        if TypeFactory::is_sol_address(source) || TypeFactory::is_sol_address(to_type) {
            let ui160 = self.types.ui160;
            if TypeFactory::is_sol_address(source) {
                if TypeFactory::is_sol_contract(to_type)
                    || TypeFactory::is_sol_fixed_bytes(to_type)
                    || to_type == ui160
                {
                    return self.emit_sol_address_cast(value, to_type, block);
                }
                let as_160 = self.emit_sol_address_cast(value, ui160, block);
                return self.emit_sol_cast(as_160, to_type, block);
            }
            if TypeFactory::is_sol_contract(source)
                || TypeFactory::is_sol_fixed_bytes(source)
                || source == ui160
            {
                return self.emit_sol_address_cast(value, to_type, block);
            }
            let as_160 = self.emit_sol_cast(value, ui160, block);
            return self.emit_sol_address_cast(as_160, to_type, block);
        }
        // Dynamic `bytes`/`string` → `bytesN`: take the leading N bytes via the
        // dedicated op (`sol.bytes_cast` rejects a `!sol.string` operand).
        if TypeFactory::is_sol_reference(source) && TypeFactory::is_sol_fixed_bytes(to_type) {
            return block
                .append_operation(
                    DynBytesToFixedBytesOperation::builder(self.context, self.unknown_location)
                        .inp(value)
                        .out(to_type)
                        .build()
                        .into(),
                )
                .result(0)
                .expect("sol.dyn_bytes_to_fixedbytes always produces one result")
                .into();
        }
        // byte / bytesN ↔ {byte, bytesN, integer}.
        if TypeFactory::is_sol_fixed_bytes(source)
            || TypeFactory::is_sol_fixed_bytes(to_type)
            || TypeFactory::is_sol_byte(source)
            || TypeFactory::is_sol_byte(to_type)
        {
            return self.emit_sol_bytes_cast(value, to_type, block);
        }
        // Reference types (array / struct / string / bytes / mapping) differ
        // only by data location; a reference→reference cast routes through
        // `sol.data_loc_cast`.
        if TypeFactory::is_sol_reference(source) && TypeFactory::is_sol_reference(to_type) {
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

    /// Emits a `sol.contract_cast` between two contract/interface types
    /// (inheritance up/downcast).
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

    /// Emits a `sol.data_loc_cast` reinterpreting a reference value at a
    /// different data location (e.g. a storage array assigned to a memory one).
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

    /// Emits a `sol.conv_cast` — a representation-preserving reinterpretation
    /// that the conversion pipeline rewrites to the remapped value. It bridges
    /// the Sol-typed variable environment to the Yul world at the inline-assembly
    /// boundary: a Solidity local's `!sol.ptr<T, Stack>` is reinterpreted as the
    /// `!llvm.ptr` that Yul `llvm.load`/`llvm.store` operate on.
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

    /// Emits a `sol.enum_cast` bridging an integer ordinal to (or from) an enum
    /// type. Both sides share an integer representation, so this is a
    /// representation-preserving cast that records the change of static type
    /// (e.g. `type(E).min` materialised as the enum's lowest member).
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

    /// Emits a `sol.func_constant` producing an internal function pointer
    /// (`!sol.func_ref<…>`) to the function named `name`.
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

    /// Emits a `sol.default_func_constant` — the zero value of an internal
    /// function pointer (`!sol.func_ref<…>`), a default-initialised pointer that
    /// reverts when called.
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

    /// Emits a `sol.icall` — an indirect call through an internal function
    /// pointer `callee` — and returns its result values.
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

    /// Emits a `sol.ext_func_constant` packing a callee `address` and a 4-byte
    /// `selector` into an `!sol.ext_func_ref<…>` external function reference —
    /// the callee value of an external call.
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

    /// Emits a `sol.ext_func_selector` extracting the 4-byte selector
    /// (`!sol.fixedbytes<4>`) from an external function-reference value.
    pub fn emit_sol_ext_func_selector<'block, B>(
        &self,
        func: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ExtFuncSelectorOperation::builder(self.context, self.unknown_location)
                    .func(func)
                    .result(self.types.fixed_bytes(4))
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.ext_func_selector always produces one result")
            .into()
    }

    /// Emits a `sol.ext_func_addr` extracting the address (`!sol.address`) from
    /// an external function-reference value.
    pub fn emit_sol_ext_func_addr<'block, B>(
        &self,
        func: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(
                ExtFuncAddrOperation::builder(self.context, self.unknown_location)
                    .func(func)
                    .result(self.types.sol_address)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.ext_func_addr always produces one result")
            .into()
    }

    /// Emits `sol.gasleft` yielding all remaining gas as a `ui256` — the default
    /// gas forwarded by an external call without an explicit `{gas: ...}`, and
    /// the gas of a bare low-level call.
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
            .expect("sol.gasleft always produces one result")
            .into()
    }

    /// Emits a `sol.ext_icall` (external call through an external function
    /// reference), forwarding all remaining gas and the given `value`. ABI
    /// encoding of `operands` and decoding of the results are implicit in the
    /// op's lowering (driven by the callee's `ext_func_ref` signature). Returns
    /// the decoded result values.
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
        // Forward all remaining gas (`gasleft()`), the default for an external
        // call without an explicit `{gas: ...}` option.
        let gas: Value<'context, 'block> = self.emit_sol_gas_left(block);
        // `sol.ext_icall` results are `(i1 status, decoded-returns...)`. Prepend
        // the status type and drop it from the values handed back — a non-try
        // call reverts internally on failure, so the status is always true here.
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

    /// Emits a `sol.ext_icall` with `try_call` set — the `try` form. Unlike the
    /// plain [`Self::emit_sol_ext_icall`], a failing callee yields a `false`
    /// status (the first result) instead of reverting, so the caller can run a
    /// `catch` handler. Returns `(status, decoded-returns)`.
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

    /// Emits a `sol.lib_addr` yielding the deployed address of the library named
    /// by the link-reference symbol `name` (`"<file>:<Library>"`) — a link-time
    /// placeholder the linker resolves.
    pub fn emit_sol_lib_addr<'block, B>(&self, name: &str, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // Built via the raw `OperationBuilder`: the op's `name` `StrAttr` collides
        // with a melior-reserved builder setter, so the generated binding cannot
        // set it.
        block
            .append_operation(
                OperationBuilder::new("sol.lib_addr", self.unknown_location)
                    .add_attributes(&[(
                        Identifier::new(self.context, "name"),
                        StringAttribute::new(self.context, name).into(),
                    )])
                    .add_results(&[self.types.sol_address])
                    .build()
                    .expect("valid sol.lib_addr"),
            )
            .result(0)
            .expect("sol.lib_addr always produces one result")
            .into()
    }

    /// Emits a `sol.ext_call` with the `delegate_call` + `library_call` flags — an
    /// external library `delegatecall` to `address` (a `sol.lib_addr`). The op
    /// owns the ABI encode, the delegatecall, the revert-bubble on failure, and
    /// the result decode, so the frontend supplies only the typed arguments, the
    /// selector, and the callee's function type; returns the decoded results.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_sol_ext_call_library<'block, B>(
        &self,
        callee: &str,
        arguments: &[Value<'context, 'block>],
        address: Value<'context, 'block>,
        selector: u32,
        callee_type: FunctionType<'context>,
        block: &B,
    ) -> Vec<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let gas = self.emit_sol_gas_left(block);
        let value = self.emit_sol_constant(0, self.types.ui256, block);
        let selector_value = self.emit_sol_constant(i64::from(selector), self.types.ui256, block);
        let return_types: Vec<Type<'context>> = (0..callee_type.result_count())
            .map(|index| {
                callee_type
                    .result(index)
                    .expect("function-type result index in range")
            })
            .collect();
        // `sol.ext_call` has two result groups: the `i1` success `status` and the
        // variadic decoded `outs`. The op's lowering reverts internally on
        // failure, so the status is dropped and only the decoded results return.
        let operation = block.append_operation(
            ExtCallOperation::builder(self.context, self.unknown_location)
                .callee(StringAttribute::new(self.context, callee))
                .ins(arguments)
                .addr(address)
                .gas(gas)
                .val(value)
                .selector(selector_value)
                .delegate_call(Attribute::unit(self.context))
                .library_call(Attribute::unit(self.context))
                .callee_type(TypeAttribute::new(callee_type.into()))
                .status(self.types.i1)
                .outs(&return_types)
                .build()
                .into(),
        );
        let mut results = Vec::with_capacity(return_types.len());
        for index in 0..return_types.len() {
            results.push(
                operation
                    .result(index + 1)
                    .expect("sol.ext_call produces the declared results")
                    .into(),
            );
        }
        results
    }

    // ==== Bare low-level calls ====

    /// Appends a built bare-call operation and returns its `(status, ret_data)`
    /// results: a boolean success flag and the returned bytes in memory. Unlike
    /// `sol.ext_icall`, a bare call does not revert on failure — the caller
    /// inspects the status flag.
    fn emit_sol_bare_call_results<'block, B>(
        &self,
        operation: Operation<'context>,
        block: &B,
    ) -> (Value<'context, 'block>, Value<'context, 'block>)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let operation = block.append_operation(operation);
        let status = operation
            .result(0)
            .expect("a bare call always produces a status")
            .into();
        let ret_data = operation
            .result(1)
            .expect("a bare call always produces return data")
            .into();
        (status, ret_data)
    }

    /// Emits a `sol.bare_call` — a low-level `addr.call{value}(input)` — forwarding
    /// all remaining gas (`gasleft()`).
    pub fn emit_sol_bare_call<'block, B>(
        &self,
        address: Value<'context, 'block>,
        value: Value<'context, 'block>,
        input: Value<'context, 'block>,
        block: &B,
    ) -> (Value<'context, 'block>, Value<'context, 'block>)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let gas = self.emit_sol_gas_left(block);
        let operation = BareCallOperation::builder(self.context, self.unknown_location)
            .addr(address)
            .gas(gas)
            .val(value)
            .inp(input)
            .status(self.types.i1)
            .ret_data(self.types.sol_string_memory)
            .build()
            .into();
        self.emit_sol_bare_call_results(operation, block)
    }

    /// Emits a `sol.bare_delegate_call` — a low-level `addr.delegatecall(input)`,
    /// which carries no value — forwarding all remaining gas (`gasleft()`).
    pub fn emit_sol_bare_delegate_call<'block, B>(
        &self,
        address: Value<'context, 'block>,
        input: Value<'context, 'block>,
        block: &B,
    ) -> (Value<'context, 'block>, Value<'context, 'block>)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let gas = self.emit_sol_gas_left(block);
        let operation = BareDelegateCallOperation::builder(self.context, self.unknown_location)
            .addr(address)
            .gas(gas)
            .inp(input)
            .status(self.types.i1)
            .ret_data(self.types.sol_string_memory)
            .build()
            .into();
        self.emit_sol_bare_call_results(operation, block)
    }

    /// Emits a `sol.bare_static_call` — a low-level `addr.staticcall(input)`,
    /// which carries no value — forwarding all remaining gas (`gasleft()`).
    pub fn emit_sol_bare_static_call<'block, B>(
        &self,
        address: Value<'context, 'block>,
        input: Value<'context, 'block>,
        block: &B,
    ) -> (Value<'context, 'block>, Value<'context, 'block>)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let gas = self.emit_sol_gas_left(block);
        let operation = BareStaticCallOperation::builder(self.context, self.unknown_location)
            .addr(address)
            .gas(gas)
            .inp(input)
            .status(self.types.i1)
            .ret_data(self.types.sol_string_memory)
            .build()
            .into();
        self.emit_sol_bare_call_results(operation, block)
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
        transient: bool,
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
            byte_offset.into(),
        );
        let mut operation = StateVarOperation::builder(self.context, self.unknown_location)
            .sym_name(StringAttribute::new(self.context, name))
            .r#type(TypeAttribute::new(element_type))
            .slot(slot_attribute)
            .byte_offset(byte_offset_attribute);
        // A `transient` variable (EIP-1153) lives in the separate transient
        // slot space; the attribute makes its accesses lower to TLOAD/TSTORE.
        if transient {
            operation = operation.transient(Attribute::unit(self.context));
        }
        block.append_operation(operation.build().into());
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
