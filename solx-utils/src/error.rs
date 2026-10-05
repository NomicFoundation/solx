//!
//! solx shared errors.
//!

/// Error for the combination of memory-unsafe assembly and stack-too-deep.
pub const ERROR_UNSAFE_MEMORY_ASM_STACK_TOO_DEEP: &str = r#"
This contract cannot be compiled due to a combination of a memory-unsafe assembly block and a stack-too-deep error.
The compiler can automatically fix the stack-too-deep error, but only in the absence of memory-unsafe assembly.
Please inspect assembly blocks that have produced warnings for this contract according to the requirements at:

    https://docs.soliditylang.org/en/latest/assembly.html#memory-safety

and then mark it with a memory-safe tag after removing operations that can interfere with normal memory allocation.
Alternatively, if you feel confident, you may suppress this error project-wide by
setting the EVM_DISABLE_MEMORY_SAFE_ASM_CHECK environment variable:

    EVM_DISABLE_MEMORY_SAFE_ASM_CHECK=1 <your build command>

Beware of the memory corruption risks described at the link above!
"#;

/// The environment variable to disable the memory-safe assembly check.
pub const ENV_DISABLE_UNSAFE_MEMORY_ASM_STACK_TOO_DEEP_CHECK: &str =
    "EVM_DISABLE_MEMORY_SAFE_ASM_CHECK";
