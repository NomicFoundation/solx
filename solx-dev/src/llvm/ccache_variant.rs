//!
//! Compiler cache variants.
//!

///
/// The list compiler cache variants to be used as constants.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CcacheVariant {
    /// Standard ccache.
    Ccache,
    /// Mozilla's sccache.
    Sccache,
}

impl std::str::FromStr for CcacheVariant {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ccache" => Ok(Self::Ccache),
            "sccache" => Ok(Self::Sccache),
            value => Err(format!("Unsupported ccache variant: `{value}`")),
        }
    }
}

impl std::fmt::Display for CcacheVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Ccache => write!(f, "ccache"),
            Self::Sccache => write!(f, "sccache"),
        }
    }
}
