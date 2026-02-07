# Binary Layout and Linking

This chapter describes how **solx** models deploy/runtime bytecode objects, dependency data, and post-compilation linking.

## Contract Object Model

EVM contracts have two code segments:

- **Deploy code (init code)**: runs only during contract creation.
- **Runtime code**: returned by deploy code and stored as the contract's permanent code.

Deploy code typically builds runtime bytes in memory and executes `RETURN(offset, size)`.

## `solc` JSON Assembly Layout

In legacy assembly JSON, the object is split into top-level deploy code and nested runtime code:

- Top-level `.code`: deploy instruction stream.
- `.data["0"]`: runtime object.
- `.data[<hex>]`: additional referenced data objects (for example constructor-time dependencies).

Conceptually:

```json
{
  ".code": [ /* deploy instructions */ ],
  ".data": {
    "0": { /* runtime assembly object */ },
    "ab12...": { /* dependency object or hash */ }
  }
}
```

The EVM assembly layer exposes this as `Assembly { code, data }`, with `runtime_code()` reading `data["0"]`.

## Dependencies and `CREATE` / `CREATE2`

Factory-style deploy code can reference other contract objects. In assembly, this is represented via data entries and push-style aliases:

- `PUSH [$]` (`PUSH_DataOffset`) for object offset
- `PUSH #[$]` (`PUSH_DataSize`) for object size
- `PUSH data` (`PUSH_Data`) for raw dependency chunks

These operands are resolved during assembly preprocessing before LLVM lowering.

## Deploy Stub Shape

The minimal deploy stub pattern is:

1. Load runtime size (`datasize`).
2. Load runtime offset (`dataoffset`).
3. Copy bytes from code section to memory.
4. Return copied bytes.

The EVM codegen emits this canonical form in `minimal_deploy_code()` using:

- `llvm.evm.datasize(metadata !"...")`
- `llvm.evm.dataoffset(metadata !"...")`
- `llvm.memcpy` from `addrspace(4)` (code) to `addrspace(1)` (heap)
- `llvm.evm.return`

## `datasize` / `dataoffset` Builtins

Yul builtins `datasize(<object>)` and `dataoffset(<object>)` lower to EVM intrinsics with metadata object names.

In **solx**, these are translated to LLVM intrinsics:

- `llvm.evm.datasize`
- `llvm.evm.dataoffset`

This is how deploy stubs reference embedded runtime/dependency objects without hardcoding absolute byte offsets.

## Metadata Hash and CBOR Tail

Runtime bytecode may include CBOR metadata appended at the end.

- The payload can include compiler version info and optional metadata hash fields.
- Hash behavior is configurable with `--metadata-hash` (for example `ipfs`).
- CBOR appending can be disabled with `--no-cbor-metadata`.

In the build pipeline, metadata bytes are appended to runtime objects before final assembly/linking.

## Library Linking

Library references are resolved at link time:

- The linker patches linker symbols with final addresses.
- If a symbol is unresolved, **solx** records its offsets and emits placeholders in hex output.
- Placeholder format follows the common pattern `__$<keccak-256-digest>$__`.

Standard JSON output reports unresolved positions through `evm.*.linkReferences` so external tooling can link later.

## Dependency Resolution and Path Aliasing

The assembly preprocessor performs a normalization pass over all contracts before lowering:

1. Hash deploy and runtime sub-objects.
2. Build `hash -> full contract path` mapping.
3. Rewrite `.data` entries from embedded objects to stable path references (`Data::Path`).
4. Build index mappings for deploy and runtime dependency tables.
5. Replace instruction aliases (`PUSH_DataOffset`, `PUSH_DataSize`, `PUSH_Data`) with resolved identifiers.

Two details are important:

- Entry `"0"` is always treated as runtime code and mapped to `<contract>.runtime`.
- Hex indices are normalized to 32-byte (64 hex char) aliases before lookup, so short keys and padded keys resolve consistently.

This path aliasing step gives deterministic dependency identifiers for later object assembly and linking.
