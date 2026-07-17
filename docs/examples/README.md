# CLI documentation examples

The input files behind every example in
[`docs/src/user-guide/02-command-line-interface.md`](../src/user-guide/02-command-line-interface.md):

- **`Simple.sol`** — used by all Solidity examples. Do not edit it: its keccak256
  (`0x402fe0b3…`) appears verbatim in the documented metadata example, so any change
  invalidates every hash and bytecode example at once.
- **`Simple.yul`** — the `--yul` mode example.
- **`Simple.ll`** — the `--llvm-ir` mode example; derived from `Simple.sol`'s own
  `--emit-llvm-ir` output (the optimized runtime module).

The `docs_examples` CLI test (`solx/tests/cli/docs_examples.rs`) treats the guide
itself as the test definition: every ```bash block whose next code block is a
```text block is a case — the documented commands are run exactly as written
(`solx …`, `ls '<dir>'`, and leading `NAME='value'` assignments), and the output
block is checked against what they actually print. `...` elisions are wildcards, and
benchmark timings are compared by label only. The test runs with the rest of the CLI
suite on every PR, so compiler output and documentation cannot drift apart silently,
and newly documented examples are covered automatically.

When the test fails after an intentional output change, regenerate the stale blocks
in place and review the diff:

```bash
SOLX_DOCS_BLESS=1 cargo test -p solx --test mod docs_examples
```

Blocks that cannot be regenerated mechanically — structural changes, or output that
embeds the working directory (e.g. paths inside DWARF) and must stay behind `...` —
are reported for a manual update instead. Version strings, hashes, and the CBOR
trailer change on every version or solc-fork bump, so those PRs are expected to
carry a blessed documentation update.
