# Architecture

**solx** is an LLVM-based compiler that translates Solidity source code into optimized EVM bytecode.

## Components

The compiler consists of three repositories:

1. [solx](https://github.com/NomicFoundation/solx) — The main compiler executable and Rust crates that translate Yul and EVM assembly to LLVM IR.
2. [solx-solidity](https://github.com/NomicFoundation/solx-solidity) — An LLVM-friendly fork of the Solidity compiler that emits Yul and EVM assembly.
3. [solx-llvm](https://github.com/matter-labs/solx-llvm) — A fork of the LLVM framework with an EVM target backend.

## Compilation Pipeline

```text
                        ┌─────────────────────────────────────────────┐
                        │                  Frontend                   │
┌──────────┐            │  ┌────────────────┐       ┌──────────────┐  │
│ Solidity │ ────────── │  │ solx-solidity  │ ───── │     solx     │  │
│  source  │            │  │                │       │              │  │
└──────────┘            │  │ Parsing,       │ Yul / │ Yul & EVM    │  │
                        │  │ semantic       │ EVM   │ assembly     │  │
                        │  │ analysis       │ asm   │ translation  │  │
                        │  └────────────────┘       └──────────────┘  │
                        └─────────────────────────────────────────────┘
                                                           │
                                                        LLVM IR
                                                           │
                                                           ▼
                        ┌─────────────────────────────────────────────┐
                        │                 Middle-end                  │
                        │  ┌────────────────────────────────────────┐ │
                        │  │           LLVM Optimizer               │ │
                        │  │                                        │ │
                        │  │  IR transformations and optimizations  │ │
                        │  └────────────────────────────────────────┘ │
                        └─────────────────────────────────────────────┘
                                                           │
                                                     Optimized IR
                                                           │
                                                           ▼
                        ┌─────────────────────────────────────────────┐
                        │                  Backend                    │
                        │  ┌────────────────────────────────────────┐ │
                        │  │         solx-llvm EVM Target           │ │
                        │  │                                        │ │
                        │  │  Instruction selection, register       │ │
                        │  │  allocation, code emission             │ │
                        │  └────────────────────────────────────────┘ │
                        └─────────────────────────────────────────────┘
                                                           │
                                                           ▼
                                                   ┌──────────────┐
                                                   │ EVM bytecode │
                                                   └──────────────┘
```

### Frontend

The frontend transforms Solidity source code into LLVM IR:

1. **solx-solidity** parses the Solidity source, performs semantic analysis, and emits either Yul or EVM assembly.
2. **solx** reads the Yul or EVM assembly and translates it into LLVM IR.

### Middle-end

The LLVM optimizer applies a series of IR transformations to improve code quality and performance. These optimizations are target-independent and work on the LLVM IR representation.

### Backend

The **solx-llvm** EVM target converts optimized LLVM IR into EVM bytecode. This includes:

- Instruction selection (mapping IR operations to EVM opcodes)
- Register allocation (managing the EVM stack)
- Stackification (converting register-based code to stack-based EVM operations)
- Code emission (generating the final bytecode)

## Why a Fork of solc?

The **solx-solidity** fork includes modifications to make the Solidity compiler output compatible with LLVM IR generation. The upstream **solc** compiler is designed to emit EVM bytecode directly, but **solx** needs intermediate representations (Yul or EVM assembly) that can be translated to LLVM IR.

The fork maintains compatibility with upstream **solc** and tracks its releases.
