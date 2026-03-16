//!
//! Shared constant emission helpers for the MLIR builder.
//!
//! These methods are used by both the LLVM and Sol dialect modules
//! to emit constant operations from pre-built attributes and limb
//! decompositions.
//!

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Identifier;
use melior::ir::Value;
use melior::ir::operation::OperationBuilder;

use crate::builder::MlirContext;

impl<'context> MlirContext<'context> {
    /// Maximum number of 32-bit limbs in a 256-bit integer (256 / 32).
    const MAX_LIMB_COUNT: usize = 8;

    /// Emits an `llvm.mlir.constant` from a pre-built attribute.
    pub(crate) fn emit_constant_from_attribute<'block, B>(
        &self,
        attribute: Attribute<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        self.emit_constant_operation(crate::ops::MLIR_CONSTANT, attribute, block)
    }

    /// Emits an `i256` constant from 32-bit limbs in little-endian order.
    ///
    /// Each limb is at most `u32::MAX`, so it fits in a positive `i64` without
    /// sign-extension issues.
    pub(crate) fn emit_i256_from_limbs<'block, B>(
        &self,
        limbs: &[u32],
        block: &B,
    ) -> Value<'context, 'block>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        debug_assert!(!limbs.is_empty() && limbs.len() <= Self::MAX_LIMB_COUNT);
        let mut result = self.emit_i256_constant(limbs[0] as i64, block);
        for (i, &limb) in limbs.iter().enumerate().skip(1) {
            if limb == 0 {
                continue;
            }
            let limb_val = self.emit_i256_constant(limb as i64, block);
            let shift = self.emit_i256_constant(i as i64 * Self::LIMB_BIT_WIDTH, block);
            let shifted = self
                .emit_llvm_operation(crate::ops::SHL, limb_val, shift, self.i256_type, block)
                .expect("llvm.shl operation is well-formed");
            result = self
                .emit_llvm_operation(crate::ops::OR, result, shifted, self.i256_type, block)
                .expect("llvm.or operation is well-formed");
        }
        result
    }

    /// Emits a `sol.constant` from a pre-built MLIR attribute.
    pub(crate) fn emit_sol_constant_from_parsed_attribute<'block, B>(
        &self,
        attribute: Attribute<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        self.emit_constant_operation(crate::ops::sol::CONSTANT, attribute, block)
    }

    /// Shared helper for emitting a constant operation with an attribute.
    pub(crate) fn emit_constant_operation<'block, B>(
        &self,
        operation_name: &str,
        attribute: Attribute<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(operation_name, self.unknown_location)
                    .add_attributes(&[(Identifier::new(self.context, "value"), attribute)])
                    .add_results(&[self.i256_type])
                    .build()
                    .expect("constant operation is well-formed"),
            )
            .result(0)?
            .into())
    }
}
