//!
//! MLIR builder for Sol dialect emission.
//!
//! Contains the [`Builder`] type with cached MLIR types and emission methods
//! for Sol dialect operations: contracts, functions, constants, control flow,
//! memory, comparisons, calls, state variables, and EVM context intrinsics.
//!

pub mod type_factory;

use melior::dialect::ods::scf::IfOperation as ScfIfOperation;
use melior::dialect::ods::scf::YieldOperation as ScfYieldOperation;
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

use crate::CmpPredicate;
use crate::StateMutability;
use crate::context::builder::type_factory::TypeFactory;
use crate::ods::sol::AddrOfOperation;
use crate::ods::sol::AddressCastOperation;
use crate::ods::sol::AllocaOperation;
use crate::ods::sol::BreakOperation;
use crate::ods::sol::CallOperation;
use crate::ods::sol::CastOperation;
use crate::ods::sol::CmpOperation;
use crate::ods::sol::ConditionOperation;
use crate::ods::sol::ConstantOperation;
use crate::ods::sol::ContinueOperation;
use crate::ods::sol::ContractOperation;
use crate::ods::sol::DoWhileOperation;
use crate::ods::sol::ForOperation;
use crate::ods::sol::FuncOperation;
use crate::ods::sol::IfOperation;
use crate::ods::sol::LoadOperation;
use crate::ods::sol::RequireOperation;
use crate::ods::sol::ReturnOperation;
use crate::ods::sol::RevertOperation;
use crate::ods::sol::StateVarOperation;
use crate::ods::sol::StoreOperation;
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

    /// Emits a `sol.constant` of the given type from a decimal string.
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_decimal_str<'block, B>(
        &self,
        value: &str,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("{value} : {result_type}"))
            .ok_or_else(|| anyhow::anyhow!("invalid {result_type} decimal literal: {value}"))?;
        self.emit_constant_operation(attribute, result_type, block)
    }

    /// Emits a `sol.constant` of the given type from a hex string (without `0x` prefix).
    ///
    /// # Errors
    ///
    /// Returns an error if the string cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_from_hex_str<'block, B>(
        &self,
        hexadecimal: &str,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let attribute = Attribute::parse(self.context, &format!("0x{hexadecimal} : {result_type}"))
            .ok_or_else(|| anyhow::anyhow!("invalid {result_type} hex literal: 0x{hexadecimal}"))?;
        self.emit_constant_operation(attribute, result_type, block)
    }

    /// Emits an all-ones `sol.constant` for the given integer type.
    ///
    /// # Errors
    ///
    /// Returns an error if the constant cannot be parsed as an MLIR integer attribute.
    pub fn emit_sol_constant_all_ones<'block, B>(
        &self,
        integer_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let bit_width = TypeFactory::integer_bit_width(integer_type);
        let all_ones_hex = "f".repeat(bit_width as usize / 4);
        self.emit_sol_constant_from_hex_str(&all_ones_hex, integer_type, block)
    }

    // ==== Terminators ====

    /// Emits a `sol.revert` with an empty signature (no error data).
    // TODO(sol-dialect): mark `sol.revert` as `IsTerminator` like `sol.return`
    // so callers don't need to append `llvm.unreachable` after it.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_revert<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            RevertOperation::builder(self.context, self.unknown_location)
                .signature(StringAttribute::new(self.context, ""))
                .args(&[])
                .build()
                .into(),
        );
    }

    /// Emits a `sol.require` conditional revert with an empty signature.
    ///
    /// Reverts if `condition` is false. Not a terminator — execution continues
    /// after this op when the condition is true.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed, indicating a bug in the builder.
    pub fn emit_sol_require<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            RequireOperation::builder(self.context, self.unknown_location)
                .cond(condition)
                .msg(StringAttribute::new(self.context, ""))
                .args(&[])
                .build()
                .into(),
        );
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

    /// Emits a value-producing `scf.if` with then and else regions.
    ///
    /// Returns `(then_block, else_block)`. Each region must be terminated
    /// with `emit_scf_yield` passing a value matching the result type.
    /// The operation result is the yielded value from the taken branch.
    ///
    /// # Errors
    ///
    /// Returns an error if the operation result cannot be extracted.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_scf_if<'block>(
        &self,
        condition: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &BlockRef<'context, 'block>,
    ) -> anyhow::Result<(
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        Value<'context, 'block>,
    )>
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
            ScfIfOperation::builder(self.context, self.unknown_location)
                .results(&[result_type])
                .condition(condition)
                .then_region(then_region)
                .else_region(else_region)
                .build()
                .into(),
        );

        let result = operation.result(0)?.into();
        let then_ref = operation
            .region(0)
            .expect("scf.if has then region")
            .first_block()
            .expect("then region has a block");
        let else_ref = operation
            .region(1)
            .expect("scf.if has else region")
            .first_block()
            .expect("else region has a block");
        Ok((then_ref, else_ref, result))
    }

    /// Emits a `scf.yield` region terminator with a value.
    ///
    /// # Panics
    ///
    /// Panics if the MLIR operation cannot be constructed.
    pub fn emit_scf_yield<'block, B>(&self, operands: &[Value<'context, 'block>], block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            ScfYieldOperation::builder(self.context, self.unknown_location)
                .results(operands)
                .build()
                .into(),
        );
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

    /// Emits a `sol.load` from a pointer with an explicit result type.
    ///
    /// Use this when the pointer element type is known at emission time.
    ///
    /// # Errors
    ///
    /// Returns an error if the load operation result cannot be extracted.
    pub fn emit_sol_load<'block, B>(
        &self,
        pointer: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                LoadOperation::builder(self.context, self.unknown_location)
                    .addr(pointer)
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

    // ==== Calls ====

    /// Emits a `sol.call` operation.
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
        let operation = block.append_operation(
            CallOperation::builder(self.context, self.unknown_location)
                .callee(FlatSymbolRefAttribute::new(self.context, callee))
                .outs(result_types)
                .operands(operands)
                .build()
                .into(),
        );
        if result_types.is_empty() {
            Ok(None)
        } else {
            // TODO: return all results for multi-return functions
            Ok(Some(operation.result(0)?.into()))
        }
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
    pub fn emit_sol_state_var<'block, B>(&self, name: &str, slot: u64, block: &B)
    where
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
            0,
        );
        block.append_operation(
            StateVarOperation::builder(self.context, self.unknown_location)
                .sym_name(StringAttribute::new(self.context, name))
                .r#type(TypeAttribute::new(self.types.ui256))
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
