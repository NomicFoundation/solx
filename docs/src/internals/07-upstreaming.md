# Upstreaming analysis — what belongs in solx-llvm, slang, and melior

This catalogues work currently carried in the solx Rust workspace (mostly
`solx-slang` and `solx-mlir`, on `dev-experimental`) that belongs in one of the
three first-class upstream repositories. Items are concrete (file:line) and
prioritised by leverage. Each section was produced by a deep single-agent pass
and independently cross-validated by a multi-model review panel (Claude /
Gemini) with a citation validator; the panel's corrections are folded in below
and tagged *(panel correction)*.

The cross-cutting finding: **most catalogued "known limitations" are not missing
backend capability** — the Sol/Yul dialects, SolToYul lowerings, and the EVM
target are far more complete than the frontend assumes. Several documented gaps
are pure frontend-wiring tasks where the backend op already exists (the
`bytes.push` → `sol.push_string` and inline-asm transient/`mcopy` fixes landed
exactly this way). The two places the panel found this claim *over-stated* are
called out inline (reference-type `delete`, and `operand_segment_sizes`).

---

## solx-llvm (LLVM fork: EVM target + Sol/Yul MLIR dialects)

### Genuine gaps that crash or block
1. **`sol.data_loc_cast` only lowers to a Memory destination** — `SolToYul.cpp:2320-2338`
   falls to `llvm_unreachable("NYI")` for `dst = Storage/Transient/CallData`;
   nested NYIs at `:2165` (Calldata store), `:2219` (multi-dim), `:2273` (String
   source). It aborts the whole compiler process — not a diagnostic. The frontend
   reverse-engineered the boundary and guards it (`arithmetic.rs:299-306`,
   `call/mod.rs:107`). Fix: dispatch non-Memory dsts through `evm::Builder::genCopy`
   (already supports Storage), or at minimum `notifyMatchFailure`. **Blocks the
   largest array/struct/storage cluster.** (M, rebuild)
2. **`delete` of reference-type storage needs a backend op** *(panel correction —
   previously listed "already complete")*. The recursive `genClearStorageValue`
   logic (`EVMUtil.cpp:1985-2118`) is a **local `std::function` lambda** scoped to
   one lowering, not a reusable `evm::Builder` method, and there is **no
   `Sol_DeleteOp`/`Sol_ClearOp`** in `SolOps.td`. `genMemAlloc` also asserts a
   Memory data-location (`EVMUtil.cpp:1138` array, `:1224` struct), which is why
   the frontend bails on array/struct `delete` (`arithmetic.rs:299-305`). Fix:
   promote the lambda to an `evm::Builder` method + add a `sol.delete`/`sol.clear`
   op. (M, rebuild) — *bytes/string `delete` already works via `sol.malloc`+`sol.copy`.*
3. **`sol.cast::fold` is disabled** — `SolOps.cpp:70-80` returns `{}` because
   `constFoldCastOp` does an unchecked `cast<IntegerAttr>` that fires on solx's
   signedness/width combos (self-documented in-tree: "~140 aborts"). Fix: a
   defensive integer-only folder. Optimisation only; every cast still lowers. (M, rebuild)
4. **`verbatim` Yul builtin** — no `Yul_VerbatimOp`/lowering; frontend bails
   (`intrinsic.rs:896-900`). Needs a dialect op carrying opaque bytes + an
   `LLVM::InlineAsmOp` lowering. (M, rebuild)
5. **Packed/multi-dim array `data_loc_cast` + packed-store NYIs** —
   `SolToYul.cpp:2140,2219`; plus a `LengthOpLowering` abort on a `bytes` operand
   at `:2365` (small standalone fix — add a `sol::BytesType` branch). (S–L, rebuild)
6. **mcopy pre-Cancun legalization** — `YulToStandard.cpp:1077-1088` lowers
   `yul.mcopy` to `MCOPY` unconditionally (TODO). Pre-Cancun targets get an invalid
   opcode silently. Fix: version-gated loop/identity fallback. (S–M, rebuild)
7. **Latent `llvm_unreachable` crash table** (not yet frontend-guarded; surfaces
   as coverage grows): `EVMUtil.cpp:883,895,1318,1434,2174,2848,2915,3202,3471`;
   `SolToYul.cpp:1530,1735,1858,3974`. Tied to #1 and the missing verifiers (#10). (L, rebuild)

### Dialect-semantics / ergonomics that delete *load-bearing* Rust workarounds
8. **`BytesCastOp::areCastCompatible` width rigidity → now load-bearing** —
   `SolOps.cpp:134-159` rejects width-mismatched `fixedbytes<N>↔ui256`, forcing
   `emit_sol_bytes_cast` (`builder/mod.rs:1568-1584`) to *route* via `sol.conv_cast`
   based on Rust string-width parsing. The reinterpret-vs-shift decision should be
   a dialect concern (relax the verifier with defined no-shift semantics, or a
   width/alignment-aware `CastOpInterface`). (S–M, rebuild)
9. **Typed C-FFI type predicates** (`solxIsFixedBytesType`, `solxGetFixedBytesWidth`,
   `solxIsEnumType`, …) — `TypeFactory::is_sol_*` (`type_factory/mod.rs:48-98`)
   matches `format!("{ty}")` AsmPrinter text; `fixed_bytes_width` now *gates cast
   routing* (#8) and `delete` dispatch, so AsmPrinter drift silently miscompiles.
   Implementations are trivial `isa<>`/`cast<>` one-liners. (S, rebuild)
10. **Op verifiers** (`SolOps.td:24` all-ops TODO, `:335` map, `:494` emit; repeated
    `AnyType→Sol_PtrTy/Sol_StringType` AsmPrinter FIXMEs at `:244,291,315,443,1095,1105`)
    — make frontend mis-emission fail at `module.verify()` instead of a deep crash;
    also fixes the printer bug that undermines #9. (M, rebuild)

### Already complete in solx-llvm — fix is in the *frontend*
- **`immutable`**: `Sol_ImmutableOp`/`Sol_LoadImmutableOp`/`evm::lowerSetImmutables`
  exist; frontend lays immutables as storage slots (`contract/mod.rs:379-403`).
- **Reference-type indexed event args**: `EmitOpLowering` hashes the packed
  encoding (`SolToYul.cpp:3454-3464`); the `event.rs` "not supported" comment is stale.
- **`bytes.push(x)`, transient `tload/tstore`, `mcopy` (Cancun+)**: wired this session.

---

## slang (`slang_solidity_v2` — Rust parser/binder)

1. **Type a type-name used in value position — the `abi.decode` `Void` flagship,
   a latent miscompile** *(panel — severity upgraded)*. `get_type()` is `Void` for
   the type-list elements of `abi.decode(payload, (T))`, so `abi.rs:60-126`
   reconstructs elementary types from keyword `unparse()` text, and
   `abi_decode_result_types` (`abi.rs:137-179`) falls back at `:174` to decoding any
   non-elementary position as a single `ui256` word — **silently wrong bytecode**
   for arrays/structs/UDVTs (latent only because `ui256`-width UDVTs survive it).
   Slang should type type-name expressions; the frontend should meanwhile **bail,
   not guess `ui256`**. (M)
2. **Bare function-reference typed `ext_func_ref` instead of internal `func_ref`
   — a slang typing bug** *(new; panel-confirmed)*. A bare identifier resolving to
   a `public` function is an *internal* pointer, but slang types it `ext_func_ref`
   from the declaration's visibility. solx overrides this at four sites
   (`expression/mod.rs:338-364,507-524,558-605,676-696`). Without the overrides,
   solx would emit `sol.ext_icall` (an external CALL) for an internal callback — a
   miscompile. Slang should type a function-ref *expression* by its access form. (S–M)
3. **Fold/size literal subexpressions** — `1 << 100` is typed `ui8` (type of `1`).
   solx defensively folds (`expression/mod.rs:700-741`) and recomputes widths
   (`type_conversion.rs:260-273`). Needs `LiteralType::mobile_type()` (slang#1793). (M)
4. **Always-concrete data locations + struct-member accessors** — `Inherited`
   forces the `inherited_location` param through every arm of `resolve_slang_type`
   (`type_conversion.rs:39-169`), with hard panics at `data_location.rs:64`,
   `access.rs:191`, `built_in/mod.rs:1283`. The `Struct` arm manually walks
   `members()` and uses the lone `unsafe` FFI `mlirSolGetEltType` (`member.rs:77-82`).
   Needs concrete expression-type locations + `StructType::member_types(loc)` /
   `member_index()`. (M)
5. **Canonical signature for *internal* functions** — gated to externally-visible
   functions, so internal fns fall back to AST text (`function/mod.rs:417-450`), a
   name-mangling hazard: `a.b.T`/`c.d.T`→`T`, and `mapping(uint=>uint)`/`mapping(address=>uint)`
   →`mapping`. Extend `compute_canonical_signature()` to internal/private. (S)
6. **Library / `using-for` callable classification** — `library.rs:25-91` runs a
   bespoke CST `Visitor` inferring "library call" from `Internal`/`Private`
   visibility. Needs `is_library_function()` / a callable-kind API. (S–M)
7. **Receiver-type-aware built-in-member classification** — `built_in/mod.rs:1207`
   catch-all conflates struct-field / unimplemented-builtin / unknown. (M)
8. **Binder-owned flags solx recomputes**: contract payability
   (`contract/mod.rs:59-65`), enum ordinal (3 sites). Add `is_payable()`,
   `EnumMember::ordinal()`. (`is_reference_type` is the model done right.) (S)

*Not slang's responsibility:* the multi-element-tuple `unimplemented!()`
(`type_conversion.rs:218`) is a **solx codegen TODO** (slang provides the tuple
type; the Sol dialect has no flat multi-value type); `verbatim` is solx-llvm.

---

## melior (Rust MLIR bindings — `~/src/melior`)

1. **`operand_segment_sizes` not written by the generated builder — silent invalid
   IR (MUST)** *(panel-promoted from the solx-llvm list)*. For `AttrSizedOperandSegments`
   ops the macro *generates a reader* (`macro/.../generation/element_accessor.rs:77-112`)
   but `generate_build_fn` (`macro/.../generation/operation_builder.rs:279-328`)
   never writes the attribute, so the verifier rejects the op. solx hand-sets it for
   `sol.encode` (`abi.rs:34-43`) and `sol.new` (`built_in/mod.rs:1433-1436`). Fix:
   accumulate per-group counts and emit the segment-size attr in the builder. (M)
2. **Op-naming forces 39 manual aliases** *(corrected: 39 in one file, not "~40
   across 9")* — `dialect!` hard-codes `XxxOperation` (`macro/.../operation.rs:83`),
   so `assembly/mod.rs:51-89` hand-writes 39 `…Operation as YulXxxOp` aliases (the
   Yul ops collide with Sol `Div/Mod/Return/Revert`, forcing renames). Add
   `operation_name_prefix`/`operation_name_suffix` to `DialectInput`. (S–M)
3. **Stale-`.td` rerun footgun — `YulInterfaces.td` is unmirrored** *(corrected:
   the missing include is `YulInterfaces.td` — included by `YulOps.td:13`; the
   previously-named `SolEnums.td` does not exist — Sol enums are inline in
   `SolBase.td`)*. The macro reads tablegen via `env!("LLVM_INCLUDE_DIRECTORY")` and
   emits no `cargo:rerun-if-changed`; `build.rs:35-46` hand-lists 5 `.td` files but
   not `YulInterfaces.td`, so edits to it ship stale wrappers. Fix: a `melior_build`
   helper emitting the transitive rerun set. (S → M)
4. **Wide-integer `IntegerAttribute` constructor — the C-API already exists**
   *(corrected)*. `IntegerAttribute::new` takes only `i64`, so `builder/mod.rs:301,1675`
   round-trip 256-bit constants through `Attribute::parse(format!("{v} : i256"))`.
   `mlirIntegerAttrGetFromWords` is already present in mlir-sys; melior only needs a
   safe `IntegerAttribute::from_words(ty, &[u64])` wrapper. (S)

*Not melior's problem:* `solxCreate*Type`/`solxCreate*Attr` + `mlirSol*` inference
are Sol-specific glue (keep in solx); `ffi::block_parent_region` duplicates
melior's `BlockLike::parent_region()` — solx cleanup, delete the shim.

---

## Suggested sequencing

- **melior** (smallest, self-contained, own test suite — fully verifiable): #1
  `operand_segment_sizes` (removes a silent-invalid-IR trap), #4 `from_words` (the
  C-API already exists), #2 naming knob, #3 rerun helper.
- **slang** (binder depth): start with the additive accessors — #5 internal
  canonical signatures, #8 payability/ordinal; the flagship #1/#3 and the #2 typing
  bug are deeper type-system work.
- **solx-llvm** (each verify cycle is an LLVM rebuild): #1 `data_loc_cast`
  non-Memory unblocks the largest cluster; #2 `sol.delete` op closes reference
  `delete`; #8/#9 (`BytesCast` + C-FFI predicates) remove the load-bearing string
  matching; #5 `LengthOp` bytes branch is a quick standalone.
