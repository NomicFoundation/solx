//!
//! Toolchain naming semantics for the integration summary.
//!
//! The benchmark data identifies runs only by mode strings like
//! `02.solx-main-legacy` or `01.solx-solx-E-M3B3-0.8.34`; everything the
//! summary knows about which run is the PR, which is the `main` baseline, and
//! how they pair up is derived here — and nowhere else. When the naming
//! convention drifts (a renamed compiler, a new baseline), this is the file
//! to update; unclassifiable data renders as a loud harness error upstream.
//!

///
/// The role a toolchain plays in the comparison.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Role {
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
/// Classifies a run's mode string into a role and its pairing key.
///
/// The pairing key is every token after the leading `NN.solx`/`NN.solc`
/// identifier, minus the role markers `main`/`latest`, so a PR run pairs with
/// its main counterpart.
///
pub(crate) fn classify(mode: &str) -> (Role, String) {
    let mut tokens = mode.split('-');
    let head = tokens.next().unwrap_or("");
    let rest: Vec<&str> = tokens.collect();

    let role = if head.ends_with(".solc") {
        Role::Solc
    } else if rest.contains(&"latest") {
        Role::Latest
    } else if rest.contains(&"main") {
        Role::Main
    } else if head.ends_with(".solx") {
        Role::Pr
    } else {
        Role::Other
    };

    let key = rest
        .into_iter()
        .filter(|t| *t != "main" && *t != "latest")
        .collect::<Vec<_>>()
        .join("-");
    (role, key)
}

/// The compilation pipeline (`legacy` / `viaIR`) a mode belongs to, or its
/// trailing token otherwise.
pub(crate) fn pipeline_of(mode: &str) -> String {
    mode.rsplit('-').next().unwrap_or("").to_owned()
}

///
/// A pairing key rendered for humans: the redundant `solx` token dropped and
/// the codegen shorthands spelled out (`E` → EVMLA, `Y` → Yul).
///
pub(crate) fn humanize_mode(key: &str) -> String {
    let tokens: Vec<&str> = key
        .split('-')
        .filter(|token| *token != "solx" && !token.is_empty())
        .map(|token| match token {
            "E" => "EVMLA",
            "Y" => "Yul",
            other => other,
        })
        .collect();
    if tokens.is_empty() {
        key.to_owned()
    } else {
        tokens.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::Role;
    use super::classify;
    use super::humanize_mode;
    use super::pipeline_of;

    #[test]
    fn classify_covers_every_toolchain_naming() {
        for (mode, role, key) in [
            // The solx-tester matrix: per-mode suffixes after the toolchain.
            ("01.solx-solx-E-M3B3-0.8.34", Role::Pr, "solx-E-M3B3-0.8.34"),
            (
                "00.solx-main-solx-E-M3B3-0.8.34",
                Role::Main,
                "solx-E-M3B3-0.8.34",
            ),
            // The Foundry/Hardhat matrix: one pipeline token per toolchain.
            ("03.solx-legacy", Role::Pr, "legacy"),
            ("02.solx-main-viaIR", Role::Main, "viaIR"),
            ("01.solx-latest-legacy", Role::Latest, "legacy"),
            ("00.solc-0.8.34-legacy", Role::Solc, "0.8.34-legacy"),
            // A renamed compiler matches nothing — never misread as the PR.
            ("03.mason-legacy", Role::Other, "legacy"),
        ] {
            assert_eq!(classify(mode), (role, key.to_owned()), "{mode}");
        }
    }

    #[test]
    fn pr_and_main_runs_share_a_pairing_key() {
        let (_, pr_key) = classify("01.solx-solx-Y-M3B3-0.8.34");
        let (_, main_key) = classify("00.solx-main-solx-Y-M3B3-0.8.34");
        assert_eq!(pr_key, main_key);
    }

    #[test]
    fn pipeline_is_the_trailing_token() {
        assert_eq!(pipeline_of("02.solx-main-viaIR"), "viaIR");
        assert_eq!(pipeline_of("03.solx-legacy"), "legacy");
    }

    #[test]
    fn humanized_keys_spell_out_codegens() {
        assert_eq!(humanize_mode("solx-E-M3B3-0.8.34"), "EVMLA M3B3 0.8.34");
        assert_eq!(humanize_mode("solx-Y-M3B3-0.8.34"), "Yul M3B3 0.8.34");
        assert_eq!(humanize_mode("legacy"), "legacy");
        assert_eq!(humanize_mode(""), "");
    }
}
