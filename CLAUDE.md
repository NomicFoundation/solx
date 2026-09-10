Human documentation starts at the [README](./README.md) and continues under [docs/src](./docs/src/). The [review skill](./.claude/skills/review/SKILL.md) reviews a change against this file.

## Architecture

- **solx** (this repo) is the Rust workspace: the CLI, the Slang frontend, the dialect binding and codegen.
- **Slang** is the parser and binder, a git dependency pinned by `rev` in [`Cargo.toml`](./Cargo.toml).
- [**solx-llvm**](./solx-llvm/) (submodule) is a fork of LLVM with the Sol and Yul MLIR dialects and an EVM target backend.
- [**solx-solidity**](./solx-solidity/) (submodule) is a fork of solc, kept for its [`test/libsolidity/semanticTests`](./solx-solidity/test/libsolidity/semanticTests/), which the tester runs.

```
Solidity → Slang (parse, bind) → Sol-dialect MLIR → Sol→Yul→Standard passes → LLVM IR → LLVM optimizer → EVM bytecode
```

### Workspace Crates

| Crate | Purpose |
|---|---|
| [`solx`](./solx/) | CLI entry point |
| [`solx-core`](./solx-core/) | Pipeline orchestration |
| [`solx-slang`](./solx-slang/) | Lowers the Slang AST to Sol-dialect MLIR |
| [`solx-mlir`](./solx-mlir/) | The Sol and Yul dialect binding over melior: values, places, types, blocks |
| [`solx-codegen-evm`](./solx-codegen-evm/) | LLVM IR to EVM bytecode: optimization, assembly, linking |
| [`solx-standard-json`](./solx-standard-json/) | solc-compatible JSON I/O protocol |
| [`solx-utils`](./solx-utils/) | Shared types: contract names, EVM versions, hashing, CBOR metadata |
| [`solx-dev`](./solx-dev/) | Developer tool: builds LLVM, runs project tests |
| [`solx-tester`](./solx-tester/) | Test runner over the REVM corpus |
| [`solx-compiler-downloader`](./solx-compiler-downloader/) | Downloads and verifies compiler binaries |
| [`solx-benchmark-converter`](./solx-benchmark-converter/) | Benchmark analysis and comparison |

## Build Commands

Build `solx-dev`, then LLVM with MLIR, then solx:

```bash
cargo build --release --bin solx-dev
./target/release/solx-dev llvm build --enable-mlir --enable-utils --build-type RelWithDebInfo   # target-llvm/target-final/
cargo build-slang              # target/debug/solx
cargo build-slang --release    # target/release/solx
```

## Testing

```bash
cargo test-slang                                        # unit and CLI tests of solx-slang, solx-mlir and solx
cargo clippy-slang --all-targets
cargo build-slang --release && cargo run-tester-slang   # the REVM corpus at -O M3B3 against target/release/solx
cargo run-tester-slang --path tests/solidity/simple/default.sol   # one test
```

LIT runs the fixtures under [`solx-mlir/tests/lit/`](./solx-mlir/tests/lit/) against `target/debug/solx`:

```bash
export PATH="$PWD/target-llvm/target-final/bin:$PWD/target/debug:$PATH"
PYTHONPATH=solx-llvm/llvm/utils/lit python3 target-llvm/target-final/bin/llvm-lit solx-mlir/tests/lit/
```

## Slang Frontend

`solx-slang` lowers the Slang AST to Sol-dialect MLIR through `solx-mlir`. The rules below are the design law of that frontend and the conventions a reviewer would otherwise repeat by hand.

### Ground truth

1. Ground truth is legacy solc: `solc --asm`, `--bin` and `--storage-layout` define behavior. solx is never evidence about itself.

2. The compiler shapes the tests, never the reverse. No emission code exists to keep a fixture passing. A fixture that stops matching a correct change is rewritten.

### Validation and failure

1. Codegen does no validation. Slang owns every check on the program. The frontend lowers whatever Slang accepts and never re-implements a solc rule.

2. Emission has no error path: no `Result`, no diagnostics. A construct not lowered yet is `unimplemented!`. Any other panic names the Slang guarantee that turned out broken. Input Slang accepts is never a panic site.

3. Dispatch is decided up front. The handler for a construct is resolved and called. There is no try-this-then-that and no "not applicable" return value.

### Slang as the source of truth

1. Slang is the single source of semantic truth. Types, selectors, signatures, layout, linearisation and name resolution come from its API and are never recomputed or approximated here, by the compiler or by the tester.

2. Type facts come from the dialect's own predicates, never from matching printed type text.

### Emission model

1. Codegen is one traversal. A definition is materialized the first time something names it, marked before its body is emitted, and nothing walks the program ahead of emission to collect or pre-register.

2. A module defines at least what its bodies reference and the pass pipeline removes the excess. The frontend never computes "exactly what is needed".

3. Caches are memos filled on first use, living on the object that owns the fact. A cache that must be complete before a phase is a hidden pre-pass.

4. A fact the emitting context can derive from a relationship it already sees is derived there, never threaded down as a parameter.

### Scopes and modules

1. Lowering is methods on the scope that owns the mutable state of that level: source unit, contract, function, assembly block. The AST node is input, the scope is the receiver, and each level's derived facts are computed once when the scope opens.

2. The lowering layer has no traits, macros, free functions or namespace types. One mechanism only.

3. Modules mirror what a construct is, not how Slang's grammar spells it. A group of files has one obvious reason to exist.

4. The dialect crate knows nothing about Slang, and vocabulary only the frontend uses lives in the frontend.

### Ops and dialects

1. Every dialect op has exactly one home: the entity whose operation it is. Values cast and compare, places load and store, blocks branch. Nothing builds an op outside its home.

2. Solidity emits Sol ops, inline assembly emits Yul ops. A Sol value enters Yul only through the bridge ops, and emitting one dialect's operation with the other's op is wrong even when the result is equivalent.

3. Conversion has three separate layers: raw op emitters that do one thing, value-level policy that decides which to call, and expression-level helpers that lower an expression to a target type. Every site that needs a typed expression goes through the helpers.

### Domains

1. A domain lands complete over the types that exist when it lands. Later domains extend the earlier mechanisms for the types they introduce, so every mechanism is designed once as an extension point.

### Fixtures

1. A LIT fixture pins op shape. A tester case under [`tests/solidity/`](./tests/solidity/) pins behavior.

2. A fixture has one RUN line, `solx --emit-mlir=sol %s | FileCheck %s`, and no prose. The CHECKs are the whole statement.

3. One fixture per construct. A new case joins the fixture that owns its construct.

4. Every CHECK must fail when the input line it matches is deleted.

5. Every fixture source line is observed by some CHECK.

6. A base the fixture does not observe is `abstract`.

### Naming and docs

1. One concept has one name across the frontend.

2. A lowering method is named after the node it lowers. The receiver already says which dialect.

3. An imported type that clashes with a local one takes the prefix of the crate it came from.

4. No contractions in names: `identifier`, not `id`; `message`, not `msg`; `context`, not `ctx`. Initialisms read as words stay short, such as `abi`, `mlir`, `ods`, `url`, `api`, and unit symbols such as `ms`.

5. A doc says why the item exists or what is non-obvious, never its name again.

6. A `//` comment carries only a constraint the code cannot express.

### Pull requests

1. A PR body is short and human-readable: one or two sentences saying what the PR delivers. Nothing else: no headings, lists, tables, file lists or narration.
