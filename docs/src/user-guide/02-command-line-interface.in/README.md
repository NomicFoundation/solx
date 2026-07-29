# CLI documentation examples

The input files behind every example in
[`02-command-line-interface.md`](../02-command-line-interface.md):

- **`Simple.sol`** — used by all Solidity examples. Do not edit it: its keccak256
  (`0x402fe0b3…`) appears verbatim in the documented metadata example, so any change
  invalidates every hash and bytecode example at once.
- **`Simple.yul`** — the `--yul` mode example.
- **`Simple.ll`** — the `--llvm-ir` mode example; derived from `Simple.sol`'s own
  `--emit-llvm-ir` output (the optimized runtime module).

The `docs_examples` CLI test (`solx/tests/cli/docs_examples.rs`) treats the guide
itself as the test definition, via [trycmd](https://docs.rs/trycmd): every
`console` code block in the guide is a case — this directory is copied into a
temporary sandbox, the documented `$ solx …` and `$ ls '<dir>'` commands run in
it, and the lines under each command must match its actual output. `...` on its
own line elides any run of lines, `[..]` matches anything within a line, and a
`? failed` line documents a non-zero exit. The test runs with the rest of the
CLI suite on every PR, so compiler output and documentation cannot drift apart
silently, and newly documented examples are covered automatically.

The sibling `02-command-line-interface.out/` directory exists (with only a
`.keep` file) to switch on trycmd's sandbox; without it, the commands would run
in this directory and pollute the repository. An empty `.out/` does **not**
assert that the sandbox ends up empty — the comparison is subset-matching, so
files present in the sandbox but not listed under `.out/` are simply ignored.

Regeneration after an intentional output change is documented in
[the testing chapter of the developer guide](../../developer-guide/01-testing.md).
