//!
//! Yul-dialect emission for inline assembly.
//!
//! Inline-assembly (`assembly { … }`) blocks lower to the **Yul** dialect, never
//! the Sol dialect (dialect separation): the backend pipeline is Sol → Yul →
//! Standard, so a Yul opcode or Yul control-flow construct is emitted as its own
//! `yul.*` op rather than a behaviourally-equivalent `sol.*` op. Every Yul value
//! is the signless `i256` word. Yul local variables live in `llvm.alloca` slots
//! (loaded/stored as `i256`); a Solidity variable referenced from assembly is
//! reached by reinterpreting its `!sol.ptr<…, Stack>` as `!llvm.ptr` via a
//! `sol.conv_cast` and then a plain `llvm.load`/`llvm.store`.
//!
//! These methods mirror what `solc`'s own MLIR backend emits for the same
//! source (`--mlir-action=print-init --mlir-target=evm`).
//!

use melior::dialect::llvm;
use melior::dialect::llvm::AllocaOptions;
use melior::dialect::llvm::LoadStoreOptions;
use melior::ir::Attribute;
use melior::ir::Block;
use melior::ir::BlockLike;
use melior::ir::BlockRef;
use melior::ir::Region;
use melior::ir::RegionLike;
use melior::ir::TypeLike;
use melior::ir::Value;
use melior::ir::attribute::DenseElementsAttribute;
use melior::ir::attribute::IntegerAttribute;
use melior::ir::attribute::TypeAttribute;
use melior::ir::operation::OperationLike;
use melior::ir::r#type::IntegerType;
use melior::ir::r#type::RankedTensorType;
use num::BigInt;

use crate::Builder;
use crate::ods::yul;

/// Byte alignment of a 256-bit word slot — the alignment `solc` emits on every
/// `llvm.alloca`/`llvm.load`/`llvm.store` of a Yul value.
const WORD_ALIGNMENT: i64 = 32;

impl<'context> Builder<'context> {
    /// Emits a `yul.constant` materialising the 256-bit word `value`.
    pub fn emit_yul_constant<'block, B>(&self, value: &BigInt, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        // `yul_word_attribute` builds the i256 integer attribute via the FFI
        // (a 256-bit value exceeds `IntegerAttribute::new`'s `i64`); the typed
        // `.value()` setter wants an `IntegerAttribute`, which it is.
        let value_attribute = IntegerAttribute::try_from(self.yul_word_attribute(value))
            .expect("yul.constant value is an i256 integer attribute");
        block
            .append_operation(
                yul::ConstantOperation::builder(self.context, self.unknown_location)
                    .value(value_attribute)
                    .out(
                        crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    )
                    .build()
                    .into(),
            )
            .result(0)
            .expect("yul.constant always produces one result")
            .into()
    }

    /// Emits a `yul.cmp` comparison, producing the `i256` word `1` or `0`.
    pub fn emit_yul_cmp<'block, B>(
        &self,
        predicate: crate::YulCmpPredicate,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
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
                yul::CmpOperation::builder(self.context, self.unknown_location)
                    .predicate(predicate_attribute.into())
                    .lhs(lhs)
                    .rhs(rhs)
                    .out(
                        crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    )
                    .build()
                    .into(),
            )
            .result(0)
            .expect("yul.cmp always produces one result")
            .into()
    }

    /// Emits the stack slot for a Yul local variable: a `yul.constant 1`
    /// element count followed by an `llvm.alloca` of one `i256`. Returns the
    /// `!llvm.ptr` slot address.
    pub fn emit_yul_local_alloca<'block, B>(&self, block: &B) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        let count = self.emit_yul_constant(&BigInt::from(1u32), block);
        block
            .append_operation(llvm::alloca(
                self.context,
                count,
                crate::Type::llvm_ptr(self.context).into_mlir(),
                self.unknown_location,
                AllocaOptions::new()
                    .align(Some(self.word_alignment_attribute()))
                    .elem_type(Some(TypeAttribute::new(
                        crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD)
                            .into_mlir(),
                    ))),
            ))
            .result(0)
            .expect("llvm.alloca always produces one result")
            .into()
    }

    /// Emits an `llvm.load` of a Yul word from an `!llvm.ptr` slot.
    pub fn emit_yul_local_load<'block, B>(
        &self,
        pointer: Value<'context, 'block>,
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block
            .append_operation(llvm::load(
                self.context,
                pointer,
                crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
                self.unknown_location,
                LoadStoreOptions::new().align(Some(self.word_alignment_attribute())),
            ))
            .result(0)
            .expect("llvm.load always produces one result")
            .into()
    }

    /// Emits an `llvm.store` of a Yul word into an `!llvm.ptr` slot.
    pub fn emit_yul_local_store<'block, B>(
        &self,
        value: Value<'context, 'block>,
        pointer: Value<'context, 'block>,
        block: &B,
    ) where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(llvm::store(
            self.context,
            value,
            pointer,
            self.unknown_location,
            LoadStoreOptions::new().align(Some(self.word_alignment_attribute())),
        ));
    }

    /// Emits a `yul.if`. Yul `if` has no `else` clause, so the else region is
    /// empty. Returns the then-region block for the caller to fill (terminating
    /// it with `yul.yield`).
    pub fn emit_yul_if<'block>(
        &self,
        condition: Value<'context, 'block>,
        block: &BlockRef<'context, 'block>,
    ) -> BlockRef<'context, 'block> {
        let then_region = Region::new();
        then_region.append_block(Block::new(&[]));
        let else_region = Region::new();

        block
            .append_operation(
                yul::IfOperation::builder(self.context, self.unknown_location)
                    .cond(condition)
                    .then_region(then_region)
                    .else_region(else_region)
                    .results(&[])
                    .build()
                    .into(),
            )
            .region(0)
            .expect("yul.if has then region")
            .first_block()
            .expect("then region has a block")
    }

    /// Emits a `yul.for` with condition, body, and step regions. Returns
    /// `(cond_block, body_block, step_block)`. The condition region terminates
    /// with `yul.condition`, the body and step regions with `yul.yield`.
    pub fn emit_yul_for<'block>(
        &self,
        block: &BlockRef<'context, 'block>,
    ) -> (
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
        BlockRef<'context, 'block>,
    ) {
        let cond_region = Region::new();
        cond_region.append_block(Block::new(&[]));
        let body_region = Region::new();
        body_region.append_block(Block::new(&[]));
        let step_region = Region::new();
        step_region.append_block(Block::new(&[]));

        let operation = block.append_operation(
            yul::ForOperation::builder(self.context, self.unknown_location)
                .cond(cond_region)
                .body(body_region)
                .step(step_region)
                .init_args(&[])
                .results(&[])
                .build()
                .into(),
        );
        let cond_block = operation
            .region(0)
            .expect("yul.for has cond region")
            .first_block()
            .expect("cond region has a block");
        let body_block = operation
            .region(1)
            .expect("yul.for has body region")
            .first_block()
            .expect("body region has a block");
        let step_block = operation
            .region(2)
            .expect("yul.for has step region")
            .first_block()
            .expect("step region has a block");
        (cond_block, body_block, step_block)
    }

    /// Emits a `yul.switch` over `selector` with one region per `case_values`
    /// entry plus a default region. Returns `(default_block, case_blocks)`; each
    /// region terminates with `yul.yield`.
    pub fn emit_yul_switch<'block>(
        &self,
        selector: Value<'context, 'block>,
        case_values: &[BigInt],
        block: &BlockRef<'context, 'block>,
    ) -> (BlockRef<'context, 'block>, Vec<BlockRef<'context, 'block>>) {
        let default_region = Region::new();
        default_region.append_block(Block::new(&[]));
        let mut case_regions = Vec::with_capacity(case_values.len());
        for _ in case_values {
            let case_region = Region::new();
            case_region.append_block(Block::new(&[]));
            case_regions.push(case_region);
        }

        let case_attributes: Vec<Attribute<'context>> = case_values
            .iter()
            .map(|value| self.yul_word_attribute(value))
            .collect();
        let cases_type = RankedTensorType::new(
            &[case_values.len() as u64],
            crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD).into_mlir(),
            None,
        )
        .into();
        let cases = DenseElementsAttribute::new(cases_type, &case_attributes)
            .expect("valid i256 switch-case elements");

        let operation = block.append_operation(
            yul::SwitchOperation::builder(self.context, self.unknown_location)
                .arg(selector)
                .cases(cases.into())
                .default_region(default_region)
                .case_regions(case_regions)
                .results(&[])
                .build()
                .into(),
        );
        let default_block = operation
            .region(0)
            .expect("yul.switch has default region")
            .first_block()
            .expect("default region has a block");
        let case_blocks = (0..case_values.len())
            .map(|index| {
                operation
                    .region(index + 1)
                    .expect("yul.switch has the case region")
                    .first_block()
                    .expect("case region has a block")
            })
            .collect();
        (default_block, case_blocks)
    }

    /// Emits a `yul.condition` loop-condition terminator carrying the raw word
    /// `condition` (non-zero is true).
    pub fn emit_yul_condition<'block, B>(&self, condition: Value<'context, 'block>, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            yul::ConditionOperation::builder(self.context, self.unknown_location)
                .condition(condition)
                .args(&[])
                .build()
                .into(),
        );
    }

    /// Emits a `yul.yield` region terminator.
    pub fn emit_yul_yield<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            yul::YieldOperation::builder(self.context, self.unknown_location)
                .operands(&[])
                .build()
                .into(),
        );
    }

    /// Emits a `yul.break` loop terminator.
    pub fn emit_yul_break<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            yul::BreakOperation::builder(self.context, self.unknown_location)
                .build()
                .into(),
        );
    }

    /// Emits a `yul.continue` loop terminator.
    pub fn emit_yul_continue<'block, B>(&self, block: &B)
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        block.append_operation(
            yul::ContinueOperation::builder(self.context, self.unknown_location)
                .build()
                .into(),
        );
    }

    /// Builds the signless `i256` integer attribute for a Yul word `value`.
    fn yul_word_attribute(&self, value: &BigInt) -> Attribute<'context> {
        let (sign, words) = value.to_u64_digits();
        // `i256` type and the borrowed little-endian word slice.
        unsafe {
            Attribute::from_raw(crate::ffi::solxCreateIntegerAttr(
                crate::Type::signless(self.context, solx_utils::BIT_LENGTH_FIELD)
                    .into_mlir()
                    .to_raw(),
                sign == num::bigint::Sign::Minus,
                words.len(),
                words.as_ptr(),
            ))
        }
    }

    /// The `alignment = 32 : i64` attribute carried by every Yul-word
    /// `llvm.alloca`/`llvm.load`/`llvm.store`.
    fn word_alignment_attribute(&self) -> IntegerAttribute<'context> {
        IntegerAttribute::new(
            IntegerType::new(self.context, solx_utils::BIT_LENGTH_X64 as u32).into(),
            WORD_ALIGNMENT,
        )
    }
}
