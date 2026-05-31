# Experimental Slang Frontend

solx is gaining an alternative, Rust-native frontend built on
[Slang](https://github.com/NomicFoundation/slang). It is **experimental** and
under active development; the stable pipeline (solc → Yul/EVM assembly → LLVM)
is unaffected by it.

```
Solidity → Slang (parse + bind) → Sol-dialect MLIR → LLVM IR → EVM bytecode
```

## Build gating

The Slang frontend lives in two crates that are **excluded from
`default-members`** and only compile under the `slang` feature:

- `solx-slang` — walks the Slang CST and emits Sol-dialect MLIR.
- `solx-mlir` — the Sol MLIR dialect bindings (via `melior`) and the IR builder.

A compile-time assertion enforces that exactly one frontend is active, and the
`solx-mlir` output field on a contract is `Option<…>` behind `#[cfg(feature =
"mlir")]`, so nothing leaks into the stable JSON output. Build and test it with:

```bash
cargo build --no-default-features --features slang --target-dir target-slang
cargo test-slang
```

## Architecture notes

- **Per-contract module model.** The driver iterates `source_unit.contracts()`
  and emits one MLIR module per contract. (Libraries are a separate CST variant
  and are not yet emitted as deployable objects.)
- **Sol dialect.** Solidity constructs lower to a dedicated `sol.*` MLIR dialect
  (`sol.func`, `sol.call`, `sol.cmp`, `sol.gep`, `sol.malloc`, the cast ops, …),
  which a `SolToYul` conversion pass lowers toward the Yul/EVM dialects.
- **Centralized cast routing.** `Builder::emit_sol_cast` is the single dispatcher
  that selects the right cast op (`sol.enum_cast`, `sol.address_cast`,
  `sol.contract_cast`, `sol.bytes_cast`, `sol.data_loc_cast`, or the integer-only
  `sol.cast`) for a source/target type pair.
- **Type introspection (interim).** Sol-dialect type predicates are currently
  implemented by matching the `AsmPrinter` textual form (`!sol.enum`,
  `!sol.fixedbytes`, …), centralized in `TypeFactory::is_sol_*`. These are a
  stopgap until typed C-FFI predicates (`solxIs*Type`) are exposed; they are the
  one place a change to the MLIR type printer would need to be reflected.

## Status and known limitations

Unsupported constructs **fail to compile with a clear diagnostic** rather than
miscompiling, except where noted under "semantic gaps" below.

### Not yet implemented (clean compile error)

- `bytes.push(x)` (the no-argument `bytes.push()` is supported).
- `abi.encodeWithSignature(sig, …)` with a non-literal signature.
- `abi.decode` into non-elementary types when Slang does not type the
  type-argument (arrays/structs/user types fall back to the binder; elementary
  types — `uintN`/`intN`/`bytes`/`bytesN`/`bool`/`address`/`string` — are
  reconstructed from the type-list argument).
- `abi.decode` of a storage `bytes` payload (needs a storage→memory copy first).
- Array-literal state-variable initializers (`uint[] constant a = [1, 2, 3]`).
- `verbatim` in inline assembly.
- Public/`delegatecall` libraries as deployable objects.

### Semantic gaps (may diverge from solc — under development)

These currently compile but do not yet match solc semantics in all cases:

- **`immutable` state variables** are laid out as ordinary storage slots rather
  than baked into the deployed bytecode. Behaviour can differ on chains/harnesses
  with non-persistent state and on gas.
- **Inline-assembly `leave`** is treated as a no-op (the inline strategy has no
  function frame to return from); statements after a mid-body `leave` are still
  emitted.
- **Data locations.** Reference-typed state variables and calldata-nested
  references do not always carry the precise `Memory`/`Storage`/`CallData`
  location; `bytes`/`string` share one MLIR type; `bytesN` string-literal
  materialization left-aligns regardless of a storage destination's packing.
- **`stateVar.slot` / `.offset`** in assembly emit the slot/offset from the
  frontend's storage layout; this must continue to agree with the backend's
  storage allocator (verified for `layout at N` and field packing).
- **`delete` on reference-type storage** is not yet a recursive clear.
- **Mixed-signedness comparison** widens via a bit-preserving cast; rely on
  solc's type checker to reject the cases where this would matter.

## Testing

- Unit/MLIR `lit` tests: `cargo test-slang` and `solx-mlir/tests/lit/`
  (FileCheck on emitted MLIR — these do **not** execute bytecode).
- End-to-end execution is validated against the upstream Solidity semantic tests
  via `solx-tester` (REVM); the `ci:slang` label enables the Slang test jobs in
  CI.
