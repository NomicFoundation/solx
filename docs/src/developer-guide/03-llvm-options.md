# LLVM Options

This guide documents LLVM backend options available in **solx** through the `--llvm-options` flag.

## Usage

Pass options as a single string after `=`:

```bash
solx contract.sol --llvm-options='-option1 value1 -option2 value2'
```

## EVM Backend Options

These options are specific to the custom LLVM EVM backend and affect compilation behavior directly.

### `-evm-stack-region-size <value>`

Sets the stack spill region size in bytes. The compiler uses this region to spill values that cannot remain on the EVM stack (stack-too-deep mitigation). Normally set automatically based on optimizer settings. Requires `-evm-stack-region-offset` to be set as well.

### `-evm-stack-region-offset <value>`

Sets the stack spill region memory offset. Normally set automatically to match the solc user memory offset.

### `-evm-metadata-size <value>`

Sets the metadata size hint used by the backend for gas and code size tradeoff decisions.

## Standard LLVM Diagnostic Options

Standard LLVM diagnostic options can be passed through `--llvm-options` and their output is printed to stderr. Some options (such as `-debug` and `-debug-only`) require LLVM built with assertions enabled (`-DLLVM_ENABLE_ASSERTIONS=ON`). When building from source, pass `--enable-assertions` to `solx-dev llvm --build`.

### `-time-passes`

Print timing information for each LLVM pass.

```bash
solx contract.sol --bin --llvm-options='-time-passes'
```

### `-stats`

Print statistics from LLVM passes (number of transformations applied, etc.).

### `-print-after-all`

Print LLVM IR after every optimization pass. Produces very large output (tens of thousands of lines) but useful for tracing pass behavior.

### `-print-before-all`

Print LLVM IR before every optimization pass.

### `-debug-only=<pass-name>`

Enable debug output for a specific LLVM pass. Note that `--llvm-debug-logging` controls pass-builder logging specifically, not the general LLVM `DEBUG()` macro categories.

## CLI Debug Flags

These are top-level **solx** flags (not passed through `--llvm-options`):

| Flag | Effect |
|---|---|
| `--llvm-verify-each` | Run IR verifier after each LLVM pass. Silent on success; produces an error if verification fails. |
| `--llvm-debug-logging` | Enable pass-builder debug logging. Shows which passes and analyses run, with instruction counts. |

See the [Debugging](./02-debugging.md) guide for the full set of diagnostic flags.
