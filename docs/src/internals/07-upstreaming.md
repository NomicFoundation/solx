# Upstreaming analysis — what belongs in solx-llvm, slang, and melior

This catalogues work currently carried in the solx Rust workspace (mostly
`solx-slang` and `solx-mlir`, on `dev-experimental`) that arguably belongs in
one of the three first-class upstream repositories. It is the output of a
three-way audit; items are concrete (file:line) and prioritised by leverage.

The cross-cutting finding: **most catalogued "known limitations" are not missing
backend capability** — the Sol/Yul dialects, SolToYul lowerings, and the EVM
target are far more complete than the frontend assumes. Several documented gaps
are pure frontend-wiring tasks where the backend op already exists and is
registered (the `bytes.push` → `sol.push_string` fix landed exactly this way).

---

## solx-llvm (LLVM fork: EVM target + Sol/Yul MLIR dialects)

### Genuine gaps that crash or block
1. **`sol.data_loc_cast` only lowers to a Memory destination** — `SolToYul.cpp:2338`
   falls to `llvm_unreachable("NYI")` for `dst = Storage/Transient/CallData` and
   `StringType` storage sources (related NYIs at `:2273`, `:2165`). A frontend
   cannot route around an `llvm_unreachable` — it aborts the whole compiler.
   `evm::Builder::genCopy` already supports storage destinations, so this is
   reusable. **Blocks the largest array/struct/storage cluster.** (M, rebuild)
2. **`sol.cast::fold` is disabled** — `SolOps.cpp:70-80` returns `{}` because the
   upstream `constFoldCastOp` trips an assertion (~140 aborts). Fixing the folder
   re-enables a bytecode-size optimisation on every contract. (M, rebuild)
3. **`verbatim` Yul builtin** — no `Yul_VerbatimOp`, no lowering; frontend bails
   (`intrinsic.rs`). Needs a dialect op carrying the literal bytes. (M, rebuild)
4. **Packed/multi-dim array `data_loc_cast` + packed-store NYIs** — `SolToYul.cpp:2140,2219`.
5. **mcopy pre-Cancun legalization** — `YulToStandard.cpp:1082` TODO; emit a
   loop/identity-precompile fallback when the target lacks `MCOPY`.

### Ergonomics that would delete Rust workarounds
6. **`BytesCastOp::areCastCompatible` width rigidity** — forces the frontend's
   `fixedbytes<N>↔ui256` → `sol.conv_cast` routing (`builder/mod.rs:emit_sol_bytes_cast`).
   The conv_cast choice is *semantically* correct (raw reinterpret vs value
   shift) but the dispatch should be a dialect concern, not Rust string-width
   arithmetic. (S–M, rebuild)
7. **Typed C-FFI type predicates** (`solxIsFixedBytesType`, `solxGetFixedBytesWidth`,
   `solxIsEnumType`, …) — today `TypeFactory::is_sol_*` matches `format!("{ty}")`
   AsmPrinter output, which silently breaks if a type's assembly format changes.
8. **`operand_segment_sizes` builders** for `AttrSizedOperandSegments` ops
   (`sol.encode`) — removes the hand-rolled attribute plumbing in `abi.rs:34-43`.
9. **Op verifiers** (`SolOps.td:24,335` TODOs; `AnyType→Sol_PtrTy` FIXMEs) — catch
   frontend mis-emission at `module.verify()` instead of miscompiling.

### Already complete in solx-llvm — fix is in the *frontend*, not here
- **`immutable`**: `Sol_ImmutableOp`, `Sol_LoadImmutableOp`, `DataLocation::Immutable`,
  the `SetImmutableOp` chain, and `evm::lowerSetImmutables` all exist. The frontend
  lays immutables as ordinary storage slots (`contract/mod.rs:379-403`) and never
  emits `sol.immutable`. **Frontend wiring.**
- **`bytes.push(x)`**: `Sol_PushStringOp` + lowering existed — wired up this session.
- **Reference-type indexed event args**: `EmitOpLowering` already hashes the packed
  encoding; the "not supported by solc-MLIR" comment (`event.rs`) is stale.
- **`delete` of ref-type storage**: clearing primitives (`genClearStorageArrayTail`,
  `genClearStringStorageTail`) already exist.
- **transient storage / mcopy opcodes**: fully wired end-to-end (used this session).

---

## slang (`slang_solidity_v2` — Rust parser/binder)

1. **Type a type-name used in value position** (the `abi.decode` `Void` flagship)
   — `abi.rs:60-179` re-parses elementary keywords' `unparse()` text because
   `get_type()` is `Void` for the type-list elements. Binder should give the
   named type; deletes ~75 lines and uncaps `abi.decode` past elementary types.
2. **Fold/size literal subexpressions** — `mod.rs:616-621`, `type_conversion.rs:260-273`.
   `1 << 100` is typed `ui8` (type of `1`); solx folds defensively and recomputes
   bit widths. Needs `RationalNumberType` result typing + `LiteralType::mobile_type()`
   (slang#1793).
3. **Resolve struct-member / aggregate types through the binder** — `type_conversion.rs:145-169`
   walks `members()` and threads an `inherited_location` param through *every*
   `resolve_slang_type` arm to compensate for `Inherited` data locations.
4. **Always-concrete data locations** (string/bytes, calldata-nested, `Inherited`)
   — `access.rs:189-199`, `built_in/mod.rs:1266-1271` treat `Inherited` as
   `unreachable!`.
5. **Receiver-type-aware built-in-member classification** — the `built_in/mod.rs`
   member dispatch ends in a catch-all that conflates struct-field / unimplemented
   / unknown.
6. **Canonical signature/selector for *internal* functions** — `function/mod.rs:396-450`
   re-`unparse()`s types for internal fns because `compute_canonical_signature()`
   is gated to externally-visible ones (name-mangling risk).
7. **Library / `using-for` callable classification** — `library.rs` runs a bespoke
   CST `Visitor` inferring "library call" from `Internal`/`Private` visibility.
8. **Binder-owned flags solx recomputes**: contract payability (`contract/mod.rs:59-65`),
   enum member index (3 sites). (`is_reference_type` is the model done right.)

---

## melior (`NomicFoundation/melior` — Rust MLIR bindings)

1. **Op-naming forces ~40 manual aliases** — `dialect!` names ops `XxxOperation`
   (`macro/.../operation.rs:80-84`), so `solx-slang` hand-writes
   `use ods::yul::XxxOperation as YulXxxOp` across 9 files. Add an
   `operation_name_prefix/suffix` knob or emit dialect-qualified aliases.
2. **Stale `.td` expansion footgun** — the macro reads tablegen via
   `env!("LLVM_INCLUDE_DIRECTORY")` but emits no `cargo:rerun-if-changed`, so `.td`
   edits don't re-expand until `ods.rs` is touched. `build.rs:33-46` already
   hand-lists five `.td` files. Melior should provide a helper that prints the
   (transitive) rerun-if-changed set, or document the pattern.
3. **No `BigInt`/wide-integer attribute constructor** — `IntegerAttribute::new`
   takes only `i64`, so `builder/mod.rs:300,1651` round-trip 256-bit constants
   through the textual MLIR parser. Add `from_limbs`/`from_str_radix`.

### Not melior's problem
- `solxCreate*Type`/`solxCreate*Attr` FFI and `mlirSol*` type inference are
  Sol-specific glue — keep in solx.
- `ffi::block_parent_region` duplicates melior's existing
  `BlockLike::parent_region()` — solx cleanup, delete the shim.

---

## Suggested sequencing

The highest-leverage frontend-only items (no LLVM rebuild) are **`immutable`
wiring** and the remaining **data-location frontend wiring** once solx-llvm
item #1 lands. The highest-leverage solx-llvm item is **#1 (data_loc_cast
non-Memory destinations)** — it converts compiler aborts into working lowerings
for the biggest array/struct/storage cluster. For melior, **#1 (naming)** and
**#2 (rerun-if-changed)** are cheap and remove standing boilerplate/footguns.
