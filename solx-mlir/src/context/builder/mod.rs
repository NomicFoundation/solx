//!
//! MLIR builder for emission methods.
//!
//! Contains the [`Builder`] type and shared methods used by both the
//! LLVM and Sol dialect submodules to emit operations.
//!

pub mod llvm;
pub mod sol;

use melior::ir::Attribute;
use melior::ir::BlockLike;
use melior::ir::Identifier;
use melior::ir::Location;
use melior::ir::Type;
use melior::ir::Value;
use melior::ir::operation::OperationBuilder;

/// Cached MLIR types and emission methods for building MLIR operations.
///
/// Holds interned MLIR objects that are created once during [`Context`](crate::context::Context)
/// construction and reused across all emission calls.
pub struct Builder<'context> {
    /// The MLIR context with all dialects and translations registered.
    pub(crate) context: &'context melior::Context,
    /// Cached `i256` type (MLIR interns types, but avoids repeated lookups).
    pub(crate) i256_type: Type<'context>,
    /// Cached `i1` type.
    pub(crate) i1_type: Type<'context>,
    /// Cached `!sol.ptr<i256, Stack>` type for alloca operations.
    pub(crate) sol_ptr_type: Type<'context>,
    /// Cached unknown source location.
    pub(crate) unknown_location: Location<'context>,
}

impl<'context> Builder<'context> {
    /// Maximum number of 32-bit limbs in a 256-bit integer (256 / 32).
    const MAX_LIMB_COUNT: usize = solx_utils::BIT_LENGTH_FIELD / solx_utils::BIT_LENGTH_X32;

    /// Bit width of each limb for wide constant decomposition.
    const LIMB_BIT_WIDTH: i64 = solx_utils::BIT_LENGTH_X32 as i64;

    /// Returns a reference to the melior context.
    pub fn context(&self) -> &'context melior::Context {
        self.context
    }

    /// Returns an unknown source location.
    pub fn location(&self) -> Location<'context> {
        self.unknown_location
    }

    /// Returns the EVM word type (`i256`).
    pub fn i256(&self) -> Type<'context> {
        self.i256_type
    }

    /// Returns the EVM boolean type (`i1`).
    pub fn i1(&self) -> Type<'context> {
        self.i1_type
    }

    /// Emits an `i256` constant from 32-bit limbs in little-endian order.
    ///
    /// Each limb is at most `u32::MAX`, so it fits in a positive `i64` without
    /// sign-extension issues.
    ///
    /// # Errors
    ///
    /// Returns an error if `limbs` is empty or exceeds the maximum limb count.
    pub fn emit_i256_from_limbs<'block, B>(
        &self,
        limbs: &[u32],
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        anyhow::ensure!(
            !limbs.is_empty() && limbs.len() <= Self::MAX_LIMB_COUNT,
            "limb count {} is out of range 1..={}",
            limbs.len(),
            Self::MAX_LIMB_COUNT,
        );
        let mut result = self.emit_i256_constant(limbs[0] as i64, block);
        for (i, &limb) in limbs.iter().enumerate().skip(1) {
            if limb == 0 {
                continue;
            }
            let limb_val = self.emit_i256_constant(limb as i64, block);
            let shift = self.emit_i256_constant(i as i64 * Self::LIMB_BIT_WIDTH, block);
            let shifted =
                self.emit_binary_operation(Self::SHL, limb_val, shift, self.i256_type, block)?;
            result =
                self.emit_binary_operation(Self::OR, result, shifted, self.i256_type, block)?;
        }
        Ok(result)
    }

    /// Shared helper for emitting a two-operand operation with one result.
    ///
    /// # Errors
    ///
    /// Returns an error if the MLIR operation cannot be constructed.
    pub fn emit_binary_operation<'block, B>(
        &self,
        operation_name: &str,
        lhs: Value<'context, 'block>,
        rhs: Value<'context, 'block>,
        result_type: Type<'context>,
        block: &B,
    ) -> anyhow::Result<Value<'context, 'block>>
    where
        B: BlockLike<'context, 'block>,
        'context: 'block,
    {
        Ok(block
            .append_operation(
                OperationBuilder::new(operation_name, self.unknown_location)
                    .add_operands(&[lhs, rhs])
                    .add_results(&[result_type])
                    .build()?,
            )
            .result(0)
            .expect("binary operation always produces one result")
            .into())
    }

    /// Shared helper for emitting a constant operation with an attribute.
    ///
    /// # Errors
    ///
    /// Returns an error if the MLIR operation cannot be constructed.
    pub fn emit_constant_operation<'block, B>(
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
