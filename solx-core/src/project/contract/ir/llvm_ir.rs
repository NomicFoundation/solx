//!
//! The contract LLVM IR source code.
//!

///
/// The contract LLVM IR source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LLVMIR {
    /// LLVM IR source code.
    pub source: String,
    /// Dependencies of the LLVM IR translation unit.
    pub dependencies: solx_utils::Dependencies,
}

impl LLVMIR {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(path: String, code_segment: solx_utils::CodeSegment, mut source: String) -> Self {
        source.push(char::from(0));

        let runtime_code_identifier = format!("{path}.{}", solx_utils::CodeSegment::Runtime);
        let dependencies = match code_segment {
            solx_utils::CodeSegment::Deploy => {
                solx_utils::Dependencies::new(path.as_str(), Some(runtime_code_identifier))
            }
            solx_utils::CodeSegment::Runtime => {
                solx_utils::Dependencies::new(runtime_code_identifier.as_str(), None)
            }
        };

        Self {
            source,
            dependencies,
        }
    }
}
