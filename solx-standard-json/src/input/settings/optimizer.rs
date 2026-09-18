//!
//! The `solc --standard-json` input settings optimizer.
//!

///
/// The `solc --standard-json` input settings optimizer.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Optimizer {
    /// Whether the solc optimizer is enabled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// The LLVM optimization mode (0/1/2/3/s/z).
    #[serde(
        default = "Optimizer::default_mode",
        skip_serializing_if = "Option::is_none"
    )]
    pub mode: Option<char>,
    /// Whether to try to recompile with -Oz if the bytecode is too large.
    #[serde(
        default = "Optimizer::default_size_fallback",
        skip_serializing_if = "Option::is_none"
    )]
    pub size_fallback: Option<bool>,

    /// The `solc` optimizer run estimate. Accepted because Hardhat emits it, and unused: solx
    /// optimizes at the LLVM level, driven by `mode`. Never forwarded to `solc`.
    #[serde(default, skip_serializing)]
    pub runs: Option<u64>,
    /// The `solc` optimizer step settings, accepted and unused for the same reason as `runs`.
    #[serde(default, skip_serializing)]
    pub details: Option<serde_json::Value>,
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            enabled: None,
            mode: Self::default_mode(),
            size_fallback: Self::default_size_fallback(),
            runs: None,
            details: None,
        }
    }
}

impl Optimizer {
    ///
    /// The default optimization mode.
    ///
    pub fn default_mode() -> Option<char> {
        Some('3')
    }

    ///
    /// The default flag for the size fallback.
    ///
    pub fn default_size_fallback() -> Option<bool> {
        Some(false)
    }
}
