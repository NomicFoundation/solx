//!
//! EVM target warning.
//!

use crate::CodeSegment;

///
/// EVM target warning.
///
#[derive(Debug, thiserror::Error, Clone, serde::Serialize, serde::Deserialize)]
pub enum Warning {
    /// Deploy code size warning.
    #[error(
        "{0} bytecode size is {found}B that exceeds the EVM limit of {1}B",
        CodeSegment::Deploy,
        CodeSegment::Deploy.size_limit()
    )]
    DeployCodeSize {
        /// Bytecode size.
        found: usize,
    },

    /// Runtime code size warning.
    #[error(
        "{0} bytecode size is {found}B that exceeds the EVM limit of {1}B",
        CodeSegment::Runtime,
        CodeSegment::Runtime.size_limit()
    )]
    RuntimeCodeSize {
        /// Bytecode size.
        found: usize,
    },
}

impl Warning {
    /// The `solc` deploy code size limit warning code.
    pub const CODE_DEPLOY_CODE_SIZE: &'static str = "3860";

    /// The `solc` runtime code size limit warning code.
    pub const CODE_RUNTIME_CODE_SIZE: &'static str = "5574";

    /// The `solc` warning code on `type(...).runtimeCode` of a contract with an assembly constructor.
    pub const CODE_RUNTIME_CODE_ASSEMBLY_CONSTRUCTOR: &'static str = "6417";

    ///
    /// The code size warning of `code_segment` whose bytecode is `found` bytes long.
    ///
    pub fn code_size(code_segment: CodeSegment, found: usize) -> Self {
        match code_segment {
            CodeSegment::Deploy => Self::DeployCodeSize { found },
            CodeSegment::Runtime => Self::RuntimeCodeSize { found },
        }
    }

    ///
    /// Warning code.
    ///
    /// Mimic `solc` warning codes where possible for compatibility.
    ///
    pub fn code(&self) -> Option<&'static str> {
        match self {
            Self::DeployCodeSize { .. } => Some(Self::CODE_DEPLOY_CODE_SIZE),
            Self::RuntimeCodeSize { .. } => Some(Self::CODE_RUNTIME_CODE_SIZE),
        }
    }
}
