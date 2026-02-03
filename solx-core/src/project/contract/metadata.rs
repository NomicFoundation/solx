//!
//! The contract metadata.
//!

///
/// The contract metadata.
///
/// Is used to append the metadata hash to the contract bytecode.
///
#[derive(Debug, serde::Serialize)]
pub struct Metadata<'a> {
    /// The `solc` version.
    pub solc_version: Option<semver::Version>,
    /// `solx` compiler version.
    pub solx_version: semver::Version,
    /// The LLVM compiler optimizer settings.
    pub optimizer_settings: solx_codegen_evm::OptimizerSettings,
    /// The LLVM extra arguments.
    pub llvm_options: &'a [String],
}

impl<'a> Metadata<'a> {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        solc_version: Option<&solx_standard_json::Version>,
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: &'a [String],
    ) -> Self {
        Self {
            solc_version: solc_version.map(|version| version.default.to_owned()),
            solx_version: crate::version().parse().expect("Always valid"),
            optimizer_settings,
            llvm_options,
        }
    }

    ///
    /// Inserts the metadata into the original `solc` object.
    ///
    pub fn insert_into(self, metadata_string: &str) -> String {
        let mut object: serde_json::Value =
            serde_json::from_str(metadata_string).expect("Always valid");
        object.as_object_mut().expect("Always valid").insert(
            env!("CARGO_PKG_NAME").to_owned(),
            serde_json::to_value(self).expect("Always valid"),
        );
        serde_json::to_string(&object).expect("Always valid")
    }
}
