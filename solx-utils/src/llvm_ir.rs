//!
//! Common LLVM IR strings.
//!

/// Unsafe assembly marker metadata key.
pub const UNSAFE_ASM_METADATA_KEY: &str = "evm.hasunsafeasm";

/// Spill area offset report metadata key, attached by the LLVM backend to
/// modules where stack spilling occurred. The single `i64` operand is the
/// memory guard value the spill area starts at.
pub const SPILL_AREA_OFFSET_METADATA_KEY: &str = "evm-spill-area-offset";

/// Spill area size report metadata key, attached by the LLVM backend to
/// modules where stack spilling occurred. The single `i64` operand is the
/// total spill area size in bytes.
pub const SPILL_AREA_SIZE_METADATA_KEY: &str = "evm-spill-area-size";
