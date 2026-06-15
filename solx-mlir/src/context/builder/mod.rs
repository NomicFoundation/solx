//!
//! The Sol-dialect construction handle, and the emission methods still homed
//! on it.
//!
//! [`Builder`] carries the `{context, location}` the `sol_op!` macros read. The
//! methods on it (contracts, functions, reverts, requires, try, and the call
//! family) are the ops not yet dissolved onto their owning nodes.
//!

pub mod try_fallback_kind;
pub mod yul;

use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Location;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::attribute::FlatSymbolRefAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::StringAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::FunctionType;
use melior::ir::r#type::IntegerType;

use crate::StateMutability;
use crate::ods::sol::CallOperation;
use crate::ods::sol::ContractOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::RevertOperation;
use crate::ods::sol::TryOperation;

use crate::context::builder::try_fallback_kind::TryFallbackKind;

/// Emission methods for building MLIR operations.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub context: &'context melior::Context,
    /// Cached unknown source location.
    pub unknown_location: Location<'context>,
}

impl<'context> Builder<'context> {
    /// Creates a new builder with pre-cached types.
    pub fn new(context: &'context melior::Context) -> Self {
        Self {
            context,
            unknown_location: Location::unknown(context),
        }
    }

    /// Emits a `sol.contract` operation with a body region.
    ///
    /// Returns the body block inside the contract region for appending
    /// function definitions.
    pub fn emit_sol_contract<'block>(
        &self,
        name: &str,
        kind: crate::ContractKind,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let body_region = Region::new();
        let body_block = Block::new(&[]);
        body_region.append_block(body_block);

        // `solxCreateContractKindAttr` returns a valid MlirAttribute.
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

        // `solxCreateStateMutabilityAttr` returns a valid MlirAttribute.
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
            // `solxCreateFunctionKindAttr` returns a valid MlirAttribute.
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
                IntegerType::new(self.context, crate::Type::SELECTOR_BIT_WIDTH).into(),
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
        // pre-conversion Sol signature, so a fallback (like a selector-bearing
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

    /// Emits a `sol.require` conditional revert with an optional message.
    ///
    /// Reverts if `condition` is false. When `msg` is `Some`, the revert
    /// includes the string as a revert reason. Not a terminator — execution
    /// continues after this op when the condition is true.
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

    /// Emits a `sol.try` carrying the external call's success `status` and four
    /// regions — success, panic, error, fallback. A clause that is absent
    /// produces an empty region; the op's conversion performs the returndata-size
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
            panic_region.append_block(Block::new(&[(
                crate::Type::unsigned(self.context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
                self.unknown_location,
            )]));
        }

        let error_region = Region::new();
        if has_error {
            error_region.append_block(Block::new(&[(
                crate::Type::string(self.context, solx_utils::DataLocation::Memory).into_mlir(),
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
                    crate::Type::string(self.context, solx_utils::DataLocation::Memory).into_mlir(),
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

    /// Emits a `sol.call` operation and returns all of its result values in
    /// declaration order.
    pub fn emit_sol_call_results<'block, B>(
        &self,
        callee: &str,
        operands: &[Value<'context, 'block>],
        result_types: &[Type<'context>],
        block: &B,
    ) -> Vec<Value<'context, 'block>>
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
            results.push(
                operation
                    .result(index)
                    .expect("sol.call produces its declared result count")
                    .into(),
            );
        }
        results
    }
}
