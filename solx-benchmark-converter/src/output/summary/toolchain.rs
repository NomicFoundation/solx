//!
//! Toolchain naming semantics for the integration summary.
//!
//! The benchmark data identifies runs only by mode strings like
//! `02.solx-main-legacy` or `01.solx-solx-E-M3B3-0.8.34`; everything the
//! summary knows about which run is the PR, which is the `main` baseline, and
//! how they pair up is derived here — and nowhere else. Roles come from the
//! declared per-matrix toolchain tables below, so a renamed toolchain matches
//! nothing and renders as a loud harness error instead of a silently
//! misclassified baseline.
//!

///
/// The role a toolchain plays in the comparison.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Role {
    /// The current commit under test.
    Pr,
    /// The `main`-branch build the PR is compared against.
    Main,
    /// The latest released solx, a full-matrix baseline.
    Latest,
    /// Upstream solc, a full-matrix baseline.
    Solc,
    /// Unrecognized naming — surfaced as a harness error, never dropped.
    Other,
}

///
/// Which comparison matrix a suite's benchmark comes from — the two harnesses
/// name their toolchains differently.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolchainMatrix {
    /// solx-tester: a `main` baseline and the PR build.
    Tester,
    /// Foundry/Hardhat projects: solc, released solx, `main`, and the PR build.
    Project,
}

impl ToolchainMatrix {
    /// The declared toolchain names, exactly as CI assigns them.
    fn toolchains(self) -> &'static [(&'static str, Role)] {
        match self {
            Self::Tester => &[("00.solx-main", Role::Main), ("01.solx", Role::Pr)],
            Self::Project => &[
                ("00.solc", Role::Solc),
                ("01.solx-latest", Role::Latest),
                ("02.solx-main", Role::Main),
                ("03.solx", Role::Pr),
            ],
        }
    }

    ///
    /// Classifies a run's mode string into a role and its pairing key.
    ///
    /// The longest declared toolchain name matching up to a token boundary
    /// wins; the pairing key is the remainder, so a PR run pairs with its main
    /// counterpart. A mode matching no declared name is `Other`.
    ///
    pub fn classify(self, mode: &str) -> (Role, String) {
        let matched = self
            .toolchains()
            .iter()
            .filter(|(name, _)| {
                mode == *name || (mode.starts_with(name) && mode.as_bytes()[name.len()] == b'-')
            })
            .max_by_key(|(name, _)| name.len());
        match matched {
            Some((name, role)) => (*role, mode[name.len()..].trim_start_matches('-').to_owned()),
            None => (Role::Other, mode.to_owned()),
        }
    }
}

/// The compilation pipeline a mode belongs to: the project suites'
/// `legacy`/`viaIR` token or the tester's codegen token (spelled out — the
/// trailing token there is the solc version). `None` for unrecognized
/// tokens, surfaced as a harness error upstream — a silent fallback would
/// group a new codegen's data under a bogus column.
pub fn pipeline_of(mode: &str) -> Option<String> {
    for token in mode.split('-') {
        if matches!(token, "legacy" | "viaIR") {
            return Some(token.to_owned());
        }
        if let Some(codegen) = codegen_name(token) {
            return Some(codegen.to_owned());
        }
    }
    None
}

/// The spelled-out name of a tester codegen token.
fn codegen_name(token: &str) -> Option<&'static str> {
    match token {
        "E" => Some("EVMLA"),
        "Y" => Some("Yul"),
        _ => None,
    }
}

///
/// A pairing key rendered for humans: the redundant `solx` token dropped and
/// the codegen shorthands spelled out (`E` → EVMLA, `Y` → Yul).
///
pub fn humanize_mode(key: &str) -> String {
    key.split('-')
        .filter(|token| *token != "solx" && !token.is_empty())
        .map(|token| codegen_name(token).unwrap_or(token))
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use crate::output::summary::toolchain::Role;
    use crate::output::summary::toolchain::ToolchainMatrix;
    use crate::output::summary::toolchain::humanize_mode;
    use crate::output::summary::toolchain::pipeline_of;

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
            // A renamed PR compiler — never misread as any role.
            ("03.mason-legacy", ToolchainMatrix::Project),
            // A foreign compiler with a `main` token — not the baseline.
            ("02.mason-main-legacy", ToolchainMatrix::Project),
            // A renamed released-solx baseline — must not fall through to the
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
        assert_eq!(pipeline_of("02.solx-main-viaIR").as_deref(), Some("viaIR"));
        assert_eq!(pipeline_of("03.solx-legacy").as_deref(), Some("legacy"));
        // Tester modes: the codegen is the pipeline, not the trailing
        // solc version.
        assert_eq!(
            pipeline_of("01.solx-solx-E-M3B3-0.8.34").as_deref(),
            Some("EVMLA")
        );
        assert_eq!(
            pipeline_of("00.solx-main-solx-Y-M3B3-0.8.34").as_deref(),
            Some("Yul")
        );
        // A new codegen letter is a loud None, never a bogus version column.
        assert_eq!(pipeline_of("01.solx-solx-L-M3B3-0.8.34"), None);
    }

    #[test]
    fn humanized_keys_spell_out_codegens() {
        assert_eq!(humanize_mode("solx-E-M3B3-0.8.34"), "EVMLA M3B3 0.8.34");
        assert_eq!(humanize_mode("solx-Y-M3B3-0.8.34"), "Yul M3B3 0.8.34");
        assert_eq!(humanize_mode("legacy"), "legacy");
        assert_eq!(humanize_mode(""), "");
    }
}
