//!
//! Checks that every example in the CLI user guide produces the output the
//! guide shows. The guide is a [trycmd](https://docs.rs/trycmd) test file:
//! each ```console block runs as documented — `$ solx …` invokes the binary
//! under test, `$ ls '<dir>'` lists its artifacts — and the lines that
//! follow each command must match its actual output. `...` on its own line
//! elides any run of lines; `[..]` matches anything within a line. The input
//! files live in `02-command-line-interface.in/` next to the guide and are
//! copied into a temporary sandbox before the commands run.
//!
//! On mismatch, rerun with `TRYCMD=overwrite` to rewrite the stale blocks in
//! place, then review the documentation diff. Overwriting preserves `...`
//! line elisions, but inline `[..]` wildcards in rewritten regions are
//! expanded to the literal output and must be restored by hand — watch for
//! benchmark timings and for working-directory paths hex-encoded inside
//! DWARF output, which differ on every run.
//!

#[test]
fn docs_examples() {
    trycmd::TestCases::new()
        .register_bin(
            "solx",
            assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")).to_path_buf(),
        )
        .env("LC_ALL", "C")
        .case("../docs/src/user-guide/02-command-line-interface.md");
}
