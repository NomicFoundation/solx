//!
//! CMake build type.
//!

///
/// CMake build type.
///
/// Used for both LLVM and solc builds.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildType {
    /// The debug build.
    Debug,
    /// The release build.
    Release,
    /// The release with debug info build.
    RelWithDebInfo,
    /// The minimal size release build.
    MinSizeRel,
}

impl std::str::FromStr for BuildType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Debug" => Ok(Self::Debug),
            "Release" => Ok(Self::Release),
            "RelWithDebInfo" => Ok(Self::RelWithDebInfo),
            "MinSizeRel" => Ok(Self::MinSizeRel),
            value => Err(format!("Unsupported build type: `{value}`")),
        }
    }
}

impl std::fmt::Display for BuildType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Debug => write!(f, "Debug"),
            Self::Release => write!(f, "Release"),
            Self::RelWithDebInfo => write!(f, "RelWithDebInfo"),
            Self::MinSizeRel => write!(f, "MinSizeRel"),
        }
    }
}
