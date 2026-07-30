//!
//! CLI tests for the user guide examples.
//!
//! Every ```console block in the guide is a [trycmd](https://docs.rs/trycmd)
//! case. Regeneration is documented in `docs/src/developer-guide/01-testing.md`.
//!

/// trycmd resolves only registered binaries (it never searches `PATH`),
/// so external commands like `ls` must be resolved here and registered.
fn find_in_path(name: &str) -> std::path::PathBuf {
    let paths = std::env::var_os("PATH").expect("PATH is not set");
    std::env::split_paths(&paths)
        .flat_map(|directory| [directory.join(name), directory.join(format!("{name}.exe"))])
        .find(|path| path.is_file())
        .unwrap_or_else(|| panic!("`{name}` not found in PATH"))
}

#[test]
fn docs_examples() {
    let guide = "../docs/src/user-guide/02-command-line-interface.md";

    // trycmd exits green when it recognizes zero cases, so a restructuring
    // that breaks fence recognition must fail here instead of passing silently.
    let source = std::fs::read_to_string(guide).expect("the CLI user guide is readable");
    assert!(
        source.contains("```console"),
        "no ```console blocks found in {guide}; the harness would run nothing"
    );

    trycmd::TestCases::new()
        .register_bin(
            "solx",
            assert_cmd::cargo::cargo_bin!(env!("CARGO_PKG_NAME")).to_path_buf(),
        )
        .register_bin("ls", find_in_path("ls"))
        .env("LC_ALL", "C")
        .case(guide);
}
