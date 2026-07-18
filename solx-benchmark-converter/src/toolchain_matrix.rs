//!
//! Toolchain naming semantics for the integration summary.
//!
//! The benchmark data identifies runs only by mode strings like
//! `02.solx-main-legacy` or `01.solx-solx-E-M3B3-0.8.34`; everything the
//! summary knows about which run is the PR, which is the `main` baseline, and
//! how they pair up is derived here and nowhere else. Roles come from the
//! declared per-matrix toolchain tables below, so a renamed toolchain matches
//! nothing and renders as a loud harness error instead of a silently
//! misclassified baseline.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::comparison::Comparison;
use crate::role::Role;

///
/// Which comparison matrix a suite's benchmark comes from. The two harnesses
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

    ///
    /// The PR-vs-baseline diff comparisons over a set of toolchain columns: the
    /// PR run of each pairing key paired with every baseline run sharing that
    /// key. For the tester matrix that is the `main` baseline; the report
    /// renders each pair as a `(PR - baseline) / baseline` column.
    ///
    pub fn comparisons(self, toolchains: &BTreeSet<String>) -> Vec<Comparison> {
        let mut groups: BTreeMap<String, Vec<(Role, String)>> = BTreeMap::new();
        for toolchain in toolchains {
            let (role, key) = self.classify(toolchain);
            groups
                .entry(key)
                .or_default()
                .push((role, toolchain.clone()));
        }

        let mut comparisons = Vec::new();
        for group in groups.into_values() {
            let Some(pr) = group.iter().find(|(role, _)| *role == Role::Pr) else {
                continue;
            };
            for (role, baseline) in &group {
                if !matches!(role, Role::Pr | Role::Other) {
                    comparisons.push(Comparison::new(pr.1.clone(), baseline.clone()));
                }
            }
        }
        comparisons
    }

    /// The compilation pipeline a mode belongs to: the project suites'
    /// `legacy`/`viaIR` token or the tester's codegen token spelled out. The
    /// trailing token there is the solc version. `None` for unrecognized
    /// tokens, surfaced as a harness error upstream, since a silent fallback
    /// would group a new codegen's data under a bogus column.
    pub fn pipeline_of(mode: &str) -> Option<String> {
        for token in mode.split('-') {
            if matches!(token, "legacy" | "viaIR") {
                return Some(token.to_owned());
            }
            if let Some(codegen) = Self::codegen_name(token) {
                return Some(codegen.to_owned());
            }
        }
        None
    }

    ///
    /// A pairing key rendered for humans: the redundant `solx` token dropped
    /// and the codegen shorthands spelled out.
    ///
    pub fn humanize_mode(key: &str) -> String {
        key.split('-')
            .filter(|token| *token != "solx" && !token.is_empty())
            .map(|token| Self::codegen_name(token).unwrap_or(token))
            .collect::<Vec<&str>>()
            .join(" ")
    }

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

    /// The spelled-out name of a tester codegen token.
    fn codegen_name(token: &str) -> Option<&'static str> {
        match token {
            "E" => Some("EVMLA"),
            "Y" => Some("Yul"),
            _ => None,
        }
    }
}
