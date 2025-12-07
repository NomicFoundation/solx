//!
//! The LLVM projects to enable during the build.
//!

///
/// The list of LLVM projects used as constants.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Project {
    /// The Clang compiler.
    CLANG,
    /// LLD, the LLVM linker.
    LLD,
    /// The LLVM debugger.
    LLDB,
    /// The MLIR compiler.
    MLIR,
}

impl std::str::FromStr for Project {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "clang" => Ok(Self::CLANG),
            "lld" => Ok(Self::LLD),
            "lldb" => Ok(Self::LLDB),
            "mlir" => Ok(Self::MLIR),
            value => Err(format!("Unsupported LLVM project to enable: `{value}`")),
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::CLANG => write!(f, "clang"),
            Self::LLD => write!(f, "lld"),
            Self::LLDB => write!(f, "lldb"),
            Self::MLIR => write!(f, "mlir"),
        }
    }
}
