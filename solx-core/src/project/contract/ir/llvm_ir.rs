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
    pub dependencies: solx_codegen_evm::Dependencies,
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
                let mut dependencies = solx_codegen_evm::Dependencies::new(path.as_str());
                dependencies.push(runtime_code_identifier.to_owned(), true);
                dependencies
            }
            solx_utils::CodeSegment::Runtime => {
                solx_codegen_evm::Dependencies::new(runtime_code_identifier.as_str())
            }
        };

        Self {
            source,
            dependencies,
        }
    }
}
