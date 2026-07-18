//!
//! Tests for toolchain naming semantics.
//!

use std::collections::BTreeSet;

use crate::role::Role;
use crate::toolchain_matrix::ToolchainMatrix;

#[test]
fn classify_covers_every_toolchain_naming() {
    for (mode, matrix, role, key) in [
        // The solx-tester matrix: per-mode suffixes after the toolchain.
        (
            "01.solx-solx-E-M3B3-0.8.34",
            ToolchainMatrix::Tester,
            Role::Pr,
            "solx-E-M3B3-0.8.34",
        ),
        (
            "00.solx-main-solx-E-M3B3-0.8.34",
            ToolchainMatrix::Tester,
            Role::Main,
            "solx-E-M3B3-0.8.34",
        ),
        // The Foundry/Hardhat matrix: one pipeline token per toolchain.
        (
            "03.solx-legacy",
            ToolchainMatrix::Project,
            Role::Pr,
            "legacy",
        ),
        (
            "02.solx-main-viaIR",
            ToolchainMatrix::Project,
            Role::Main,
            "viaIR",
        ),
        (
            "01.solx-latest-legacy",
            ToolchainMatrix::Project,
            Role::Latest,
            "legacy",
        ),
        (
            "00.solc-0.8.34-legacy",
            ToolchainMatrix::Project,
            Role::Solc,
            "0.8.34-legacy",
        ),
    ] {
        assert_eq!(matrix.classify(mode), (role, key.to_owned()), "{mode}");
    }
}

#[test]
fn renamed_toolchains_match_nothing() {
    for (mode, matrix) in [
        // A renamed PR compiler, never misread as any role.
        ("03.mason-legacy", ToolchainMatrix::Project),
        // A foreign compiler with a `main` token, not the baseline.
        ("02.mason-main-legacy", ToolchainMatrix::Project),
        // A renamed released-solx baseline, must not fall through to the
        // PR role and double the full-matrix totals.
        ("01.solx-released-legacy", ToolchainMatrix::Project),
        // A declared name extended without a token boundary.
        ("03.solxfoo-legacy", ToolchainMatrix::Project),
        ("00.solx-main2-solx-E-M3B3-0.8.34", ToolchainMatrix::Tester),
    ] {
        let (role, key) = matrix.classify(mode);
        assert_eq!(role, Role::Other, "{mode}");
        assert_eq!(key, mode, "{mode}");
    }
}

#[test]
fn pr_and_main_runs_share_a_pairing_key() {
    let (_, pr_key) = ToolchainMatrix::Tester.classify("01.solx-solx-Y-M3B3-0.8.34");
    let (_, main_key) = ToolchainMatrix::Tester.classify("00.solx-main-solx-Y-M3B3-0.8.34");
    assert_eq!(pr_key, main_key);
}

#[test]
fn pipeline_is_derived_from_recognized_tokens() {
    assert_eq!(
        ToolchainMatrix::pipeline_of("02.solx-main-viaIR").as_deref(),
        Some("viaIR")
    );
    assert_eq!(
        ToolchainMatrix::pipeline_of("03.solx-legacy").as_deref(),
        Some("legacy")
    );
    // Tester modes: the codegen is the pipeline, not the trailing
    // solc version.
    assert_eq!(
        ToolchainMatrix::pipeline_of("01.solx-solx-E-M3B3-0.8.34").as_deref(),
        Some("EVMLA")
    );
    assert_eq!(
        ToolchainMatrix::pipeline_of("00.solx-main-solx-Y-M3B3-0.8.34").as_deref(),
        Some("Yul")
    );
    // A new codegen letter is a loud None, never a bogus version column.
    assert_eq!(
        ToolchainMatrix::pipeline_of("01.solx-solx-L-M3B3-0.8.34"),
        None
    );
}

#[test]
fn humanized_keys_spell_out_codegens() {
    assert_eq!(
        ToolchainMatrix::humanize_mode("solx-E-M3B3-0.8.34"),
        "EVMLA M3B3 0.8.34"
    );
    assert_eq!(
        ToolchainMatrix::humanize_mode("solx-Y-M3B3-0.8.34"),
        "Yul M3B3 0.8.34"
    );
    assert_eq!(ToolchainMatrix::humanize_mode("legacy"), "legacy");
    assert_eq!(ToolchainMatrix::humanize_mode(""), "");
}

#[test]
fn comparisons_pair_each_pr_run_with_its_baseline() {
    let toolchains: BTreeSet<String> = [
        "00.solx-main-solx-E-M3B3-0.8.34",
        "01.solx-solx-E-M3B3-0.8.34",
        "00.solx-main-solx-Y-M3B3-0.8.34",
        "01.solx-solx-Y-M3B3-0.8.34",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let pairs: Vec<(String, String)> = ToolchainMatrix::Tester
        .comparisons(&toolchains)
        .into_iter()
        .map(|comparison| (comparison.left, comparison.right))
        .collect();
    assert_eq!(
        pairs,
        [
            (
                "01.solx-solx-E-M3B3-0.8.34".to_owned(),
                "00.solx-main-solx-E-M3B3-0.8.34".to_owned()
            ),
            (
                "01.solx-solx-Y-M3B3-0.8.34".to_owned(),
                "00.solx-main-solx-Y-M3B3-0.8.34".to_owned()
            ),
        ]
    );
}

#[test]
fn comparisons_need_a_pr_run_to_pair_against() {
    let toolchains: BTreeSet<String> = ["00.solx-main-solx-E-M3B3-0.8.34"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    assert!(ToolchainMatrix::Tester.comparisons(&toolchains).is_empty());
}
