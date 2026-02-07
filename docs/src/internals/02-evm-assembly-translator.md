# EVM Assembly Translator

The EVM assembly translator converts legacy EVM assembly (the default `solc` output) into LLVM IR via an intermediate representation called **Ethereal IR (EthIR)**. The Yul pipeline (`--via-ir`) bypasses this translator entirely.

## Why EthIR?

EVM assembly is stack-based with dynamic jumps, making it difficult to translate directly to LLVM IR which requires explicit control flow graphs. EthIR bridges this gap by:

1. **Tracking stack state** to identify jump destinations at compile time
2. **Cloning blocks** reachable from predecessors with different stack states
3. **Reconstructing control flow** from stack-based jumps into a static CFG
4. **Resolving function calls** using metadata from the solc fork

## Translation Pipeline

```text
Solidity source
    │
    ▼
solc (solx-solidity fork)
    │  Emits EVM assembly JSON + extraMetadata
    ▼
Assembly parsing
    │  Parses instructions, resolves dependencies
    ▼
Block construction
    │  Groups instructions between Tag labels
    ▼
EthIR traversal
    │  DFS with stack simulation, block cloning
    ▼
LLVM IR generation
    │  Creates LLVM functions, basic blocks, instructions
    ▼
LLVM optimizer
    │
    ▼
EVM bytecode (via LLVM EVM backend)
```

## Key Data Structures

### Assembly

The `Assembly` struct represents the raw solc output. It contains:

- **code**: Flat list of instructions (deploy code)
- **data["0"]**: Nested assembly for runtime code
- **data[hex]**: Referenced data entries — sub-assemblies, hashes, or resolved contract paths (for CREATE/CREATE2)

Each instruction has a `name` (opcode), optional `value` (operand), and optional `source` location.

### EtherealIR

The top-level container holding:

- **entry_function**: The main contract function (deploy + runtime)
- **defined_functions**: Internal functions discovered during traversal

### Function

The `Function` struct is the core of the translator. It contains:

- **blocks**: `BTreeMap<BlockKey, Vec<Block>>` — maps each block tag to one or more instances (clones for different stack states)
- **block_hash_index**: `HashMap<BlockKey, HashSet<u64>>` — fast duplicate detection by stack hash
- **stack_size**: Maximum stack height observed, used to size LLVM stack allocations

### Block

Each `Block` represents a sequence of instructions between two Tag labels:

- **key**: `BlockKey` (code segment + tag number)
- **instance**: Clone index (0, 1, 2... for blocks visited with different stack states)
- **elements**: Instructions with full stack state snapshots
- **initial_stack / stack**: Stack state at entry and after processing

### Stack Elements

The stack tracks six kinds of values:

| Variant | Description | Example |
|---------|-------------|---------|
| `Value(String)` | Runtime value (opaque) | Result of `ADD`, `MLOAD` |
| `Constant(BigUint)` | Compile-time 256-bit constant | `0x60`, `0xFFFF` |
| `Tag(u64)` | Block tag (jump target) | Tag 42 |
| `Path(String)` | Contract dependency path | `"SubContract"` |
| `Data(String)` | Hex data chunk | `"deadbeef"` |
| `ReturnAddress(usize)` | Function return marker | Return with 2 outputs |

## Block Cloning and Stack Hashing

The same block may be reached via different code paths with different stack contents. Since the stack determines jump targets (a `JUMP` pops its destination from the stack), the translator must handle each unique stack state separately.

### How It Works

1. When entering a block, the translator computes a **stack hash** using `XxHash3_64`
2. The hash considers only `Tag` elements — tags determine control flow, while constants and runtime values affect only data flow
3. The pair `(BlockKey, stack_hash)` uniquely identifies a block instance
4. If this pair has been visited before, the block is skipped (cycle detection)
5. Otherwise, a new block instance is created

```text
Block "process" reached with stack [T_10, V_x]:  → instance 0
Block "process" reached with stack [T_20, V_y]:  → instance 1 (different tag)
Block "process" reached with stack [T_10, V_z]:  → instance 0 (same hash, reused)
```

### Stack Hash Algorithm

```rust
fn hash(&self) -> u64 {
    let mut hasher = XxHash3_64::default();
    for element in self.elements.iter() {
        match element {
            Element::Tag(tag) => hasher.write(&tag.to_le_bytes()),
            _ => hasher.write_u8(0),
        }
    }
    hasher.finish()
}
```

Only `Tag` values contribute to the hash. This is intentional: two stack states with the same tags but different runtime values will follow the same control flow path.

## Traversal Algorithm

The `Function::traverse()` method performs a depth-first traversal of blocks, simulating EVM execution:

```text
traverse(blocks, extra_metadata):
    queue ← [(entry_block, empty_stack)]
    visited ← {}

    while queue is not empty:
        (block_key, stack) ← queue.pop()
        hash ← stack.hash()

        if (block_key, hash) in visited:
            continue
        visited.add((block_key, hash))

        block ← blocks[block_key].clone_with(stack)
        for instruction in block:
            simulate_instruction(instruction, stack)
            if instruction is JUMP/JUMPI:
                queue.push((target_tag, stack))
```

### Instruction Simulation

For each instruction, the translator:

1. Pops the required number of inputs from the simulated stack
2. Computes the output (compile-time if possible, runtime value otherwise)
3. Pushes the result onto the stack
4. For control flow instructions, queues successor blocks

### Compile-Time Constant Folding

Arithmetic operations on known values are folded at compile time:

| Operands | Result |
|----------|--------|
| `Constant + Constant` | `Constant` (computed) |
| `Tag + Constant` | `Tag` (if result is valid block) |
| `Tag + Tag` | `Tag` (if result is valid block) |
| Any other combination | `Value` (runtime, opaque) |

This is critical for resolving jump targets: solc often computes jump destinations via `PUSH tag` + arithmetic.

## Function Call Detection

The translator identifies function calls using **extra metadata** from the solc fork. The `extraMetadata` JSON field lists all user-defined functions with their:

- Entry tag (in deploy and/or runtime code)
- Input parameter count
- Output return value count
- Function name and AST node ID

When a `JUMP` targets a known function entry:

1. The stack is split: return address, arguments, and remaining caller state
2. A `RecursiveCall` pseudo-instruction replaces the JUMP
3. A new `Function` is created and recursively traversed from the entry block
4. The caller's stack receives `output_size` opaque return values

```text
Before JUMP to function "add(uint,uint)":
  Stack: [... | return_tag | arg1 | arg2 | function_entry_tag]

After call detection:
  Instruction: RecursiveCall add(uint,uint), input=2, output=1
  Caller stack: [... | return_value]
  Callee: new Function traversed from entry tag
```

## LLVM IR Generation

After traversal, the translator generates LLVM IR in several phases:

### 1. Function Declaration

- **Entry function**: Uses the pre-declared contract entry point
- **Defined functions**: Creates private LLVM functions with `N × i256` parameters and return values (multiple returns use LLVM struct types)

### 2. Stack Variable Allocation

For each function, `stack_size` stack slots are allocated as LLVM `alloca` instructions. These represent the simulated EVM stack as addressable memory:

```llvm
%stack_0 = alloca i256    ; bottom of stack
%stack_1 = alloca i256
...
%stack_N = alloca i256    ; top of stack
```

For defined functions, slot 0 is reserved for the return address marker, and input parameters are stored starting from slot 1.

### 3. Basic Block Creation

Each `(BlockKey, instance)` pair becomes an LLVM `BasicBlock`:

```llvm
block_runtime_42/0:       ; tag 42, first instance
  ...
block_runtime_42/1:       ; tag 42, second instance (different stack state)
  ...
```

### 4. Instruction Translation

Each EthIR element calls `into_llvm()` to generate LLVM instructions. Stack operations map to loads/stores on the allocated stack variables:

| EVM Operation | LLVM Translation |
|---------------|-----------------|
| `PUSH 0x42` | `store i256 66, ptr %stack_N` |
| `DUP2` | `%v = load i256, ptr %stack_(N-2); store i256 %v, ptr %stack_(N+1)` |
| `ADD` | `%a = load ...; %b = load ...; %r = add i256 %a, %b; store ...` |
| `MLOAD` | `%ptr = load ...; %v = load i256, ptr addrspace(1) %ptr; store ...` |
| `JUMP` | `br label %target_block` |
| `JUMPI` | `%cond = ...; br i1 %cond, label %taken, label %fallthrough` |

## solc Fork Modifications

The EVM assembly translator relies on several modifications in the `solx-solidity` fork. The most relevant to this pipeline are:

- **`extraMetadata` output**: reports all user-defined functions with entry tags, parameter counts, and AST IDs. Without this, the translator cannot distinguish function calls from arbitrary jumps.
- **Dispatch tables for function pointers**: indirect calls are lowered to static dispatch tables instead of dynamic jumps.
- **`DUPX` / `SWAPX` instructions**: extend stack access beyond depth 16, eliminating "stack too deep" errors.
- **Disabled optimizer**: the solc optimizer is disabled to preserve function boundaries and metadata validity. All optimization is handled by the LLVM backend.

For the full list of fork modifications, see [Limitations and Differences from solc](../user-guide/04-limitations.md#solc-fork-modifications).

