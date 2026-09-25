# Architecture

**solx** is an LLVM-based compiler that translates Solidity source code into optimized EVM bytecode.

## Components

The compiler consists of three repositories:

1. [solx](https://github.com/NomicFoundation/solx) — The main compiler executable and Rust crates that lower the Solidity AST to MLIR and drive the LLVM backend.
2. [slang](https://github.com/NomicFoundation/slang) — The Solidity parser and binder.
3. [solx-llvm](https://github.com/NomicFoundation/solx-llvm) — A fork of the LLVM framework with the Sol and Yul MLIR dialects and an EVM target backend.

## Compilation Pipeline

```text
                        ┌─────────────────────────────────────────────┐
                        │                  Frontend                   │
┌──────────┐            │  ┌────────────────┐       ┌──────────────┐  │
│ Solidity │ ────────── │  │     Slang      │ ───── │     solx     │  │
│  source  │            │  │                │       │              │  │
└──────────┘            │  │ Parsing,       │ bound │ Sol-dialect  │  │
                        │  │ binding        │ AST   │ MLIR, Sol→Yul│  │
                        │  │                │       │ →LLVM passes │  │
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

1. **Slang** parses and binds the Solidity source.
2. **solx** lowers the bound AST to Sol-dialect MLIR, and the Sol→Yul→Standard passes translate it into LLVM IR.

### Middle-end

The LLVM optimizer applies a series of IR transformations to improve code quality and performance. These optimizations are target-independent and work on the LLVM IR representation.

### Backend

The **solx-llvm** EVM target converts optimized LLVM IR into EVM bytecode. This includes:

- Instruction selection (mapping IR operations to EVM opcodes)
- Register allocation (managing the EVM stack)
- Stackification (converting register-based code to stack-based EVM operations)
- Code emission (generating the final bytecode)
