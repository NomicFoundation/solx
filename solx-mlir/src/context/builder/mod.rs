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
use crate::ods::sol::AddrOfOperation;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::ArrayLitOperation;
use crate::ods::sol::AssertOperation;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::BytesCastOperation;
use crate::ods::sol::CallOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConditionOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ContinueOperation;
use crate::ods::sol::ContractOperation;
use crate::ods::sol::CopyOperation;
use crate::ods::sol::DataLocCastOperation;
use crate::ods::sol::DefaultFuncConstantOperation;
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::ExtFuncConstantOperation;
use crate::ods::sol::ExtICallOperation;
use crate::ods::sol::FuncConstantOperation;
use crate::ods::sol::GasLeftOperation;
use crate::ods::sol::ICallOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::GepOperation;
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
use crate::ods::sol::WhileOperation;
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

        if selector.is_some() || matches!(kind, Some(crate::FunctionKind::Constructor)) {
            builder = builder.orig_fn_type(TypeAttribute::new(function_type.into()));
        }

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
        block: &B,
    ) -> anyhow::Result<Vec<Value<'context, 'block>>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // Forward all remaining gas (`gas()` / `gasleft()`), the default for
        // an external call without an explicit `{gas: ...}` option.
        let gas: Value<'context, 'block> = block
            .append_operation(
                GasLeftOperation::builder(self.context, self.unknown_location)
                    .val(self.types.ui256)
                    .build()
                    .into(),
            )
            .result(0)
            .expect("sol.gas always produces one result")
            .into();
        // `sol.ext_icall` results are `(i1 status, decoded-returns...)`. We
        // prepend the status type and drop it from the values we hand back —
        // a non-try call reverts internally on failure, so the status is
        // always true here.
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
                .build()
                .into(),
        );
        let mut results = Vec::with_capacity(result_types.len());
        for index in 0..result_types.len() {
            results.push(operation.result(index + 1)?.into());
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
