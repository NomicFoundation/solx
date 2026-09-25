# EVM Instructions Reference

This chapter describes how the LLVM EVM backend models EVM instructions and lowers LLVM IR into final opcode sequences.

## Instruction Definitions

The EVM instruction set is defined in the LLVM backend via TableGen.

- It contains opcode definitions, pattern mappings, and EVM-specific pseudo-instructions.
- It covers roughly 180 instruction forms once TableGen expansions are considered (for example `DUP1..16`, `SWAP1..16`, and `PUSH` families).
- Instructions are modeled around `i256` values, matching the EVM word size.

## Address Space Model

The backend uses explicit LLVM address spaces to model EVM memory regions:

| Address space | Value | Meaning |
|---|---:|---|
| `AS_STACK` | `0` | Compiler-managed stack memory model |
| `AS_HEAP` | `1` | EVM linear memory (`MLOAD`, `MSTORE`, `MCOPY`) |
| `AS_CALL_DATA` | `2` | Call data region |
| `AS_RETURN_DATA` | `3` | Return data region |
| `AS_CODE` | `4` | Code segment |
| `AS_STORAGE` | `5` | Persistent storage |
| `AS_TSTORAGE` | `6` | Transient storage |

These constants are defined in the EVM backend header.

## Core Instruction Categories

### Arithmetic

Arithmetic opcodes map directly to `i256` operations or EVM intrinsics:

- `ADD`, `MUL`, `SUB`, `DIV`, `SDIV`, `MOD`, `SMOD`
- `ADDMOD`, `MULMOD`, `EXP`, `SIGNEXTEND`

For example, `ADD` is selected from LLVM `add i256` patterns.

### Memory

Memory instructions operate on the heap address space:

- `MLOAD` maps to a load from `AS_HEAP`
- `MSTORE` maps to a store into `AS_HEAP`
- `MCOPY` lowers memory copy operations in heap memory

### Storage

Storage instructions map to storage address spaces:

- `SLOAD`, `SSTORE` use `AS_STORAGE`
- `TLOAD`, `TSTORE` use `AS_TSTORAGE`

### Control Flow

Control flow instructions are selected from LLVM branch forms:

- `JUMP` maps from unconditional `br`
- `JUMPI` maps from conditional `br i1`

The backend also uses helper pseudos (for example `JUMP_UNLESS`) that are lowered before emission.

### Stack

EVM stack manipulation opcodes are emitted as needed:

- `DUP1..DUP16`
- `SWAP1..SWAP16`
- `POP`

They are introduced and optimized by stackification passes rather than directly authored in frontend IR.

### Cryptographic

`SHA3`/`KECCAK256` is represented through EVM-specific intrinsic plumbing:

- Machine instruction: `KECCAK256`
- LLVM intrinsic path: `llvm.evm.sha3`

## Runtime Library (`evm-stdlib.ll`)

The backend links helper wrappers from the EVM runtime standard library:

- `__addmod`
- `__mulmod`
- `__signextend`
- `__exp`
- `__byte`
- `__sdiv`
- `__div`
- `__smod`
- `__mod`
- `__shl`
- `__shr`
- `__sar`
- `__sha3`

These wrappers forward to corresponding `llvm.evm.*` intrinsics.

## Stackification Pipeline

The late codegen pipeline converts virtual-register machine IR to valid EVM stack code:

1. `EVMSingleUseExpression`: reorders machine instructions into expression-friendly form.
2. `EVMBackwardPropagationStackification`: performs backward propagation stackification from register form.
3. `EVMStackSolver` and `EVMStackShuffler`: compute and emit low-cost `DUP`/`SWAP`/spill-reload sequences.
4. `EVMPeephole`: runs late peephole optimizations before final emission.

## Stack Depth Limit

The EVM stack itself can hold up to 1024 items, but `DUP` and `SWAP` instructions can only reach the top 16 positions. The backend enforces this depth-16 manipulation reach.

This limit is exposed by `EVMSubtarget::stackDepthLimit()`.

## Pseudo-Instructions

Several pseudos are used during lowering and removed or expanded before final bytecode:

- `PUSHDEPLOYADDRESS`: materializes deploy-time address usage for libraries.
- `SELECT`: models conditional value selection.
- `CONST_I256`: represents immediate constants before stackification.
- `COPY_I256`: temporary register-copy form before stackification.
