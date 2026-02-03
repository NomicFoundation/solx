//!
//! The `solc --standard-json` input settings optimizer.
//!

///
/// The `solc --standard-json` input settings optimizer.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
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
}

impl Default for Optimizer {
    fn default() -> Self {
        Self {
            enabled: None,
            mode: Self::default_mode(),
            size_fallback: Self::default_size_fallback(),
        }
    }
}

impl Optimizer {
    ///
    /// A shortcut constructor for solx (LLVM mode).
    ///
    pub fn new(mode: char, size_fallback: bool) -> Self {
        Self {
            enabled: None,
            mode: Some(mode),
            size_fallback: Some(size_fallback),
        }
    }

    ///
    /// A shortcut constructor for solc.
    ///
    pub fn new_solc(enabled: bool) -> Self {
        Self {
            enabled: Some(enabled),
            mode: None,
            size_fallback: None,
        }
    }

    ///
    /// A shortcut constructor with all options.
    ///
    pub fn new_full(
        enabled: Option<bool>,
        mode: Option<char>,
        size_fallback: Option<bool>,
    ) -> Self {
        Self {
            enabled,
            mode,
            size_fallback,
        }
    }

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
