//!
//! The contract metadata.
//!

///
/// The contract metadata the compiler adds to the frontend's: its version and the code
/// generation settings.
///
/// The result is output and hashed into the contract bytecode.
///
#[derive(Debug)]
pub struct Metadata<'a> {
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
        optimizer_settings: solx_codegen_evm::OptimizerSettings,
        llvm_options: &'a [String],
    ) -> Self {
        Self {
            optimizer_settings,
            llvm_options,
        }
    }

    ///
    /// Inserts the metadata into the frontend's metadata object, sorting every key.
    ///
    pub fn insert_into(self, metadata_string: &str) -> String {
        let mut object: serde_json::Value =
            serde_json::from_str(metadata_string).expect("Always valid");
        let object_map = object.as_object_mut().expect("Always valid");
        object_map.insert(
            "compiler".to_owned(),
            serde_json::json!({
                "name": crate::r#const::DEFAULT_EXECUTABLE_NAME,
                "version": crate::Compiler::version(),
            }),
        );
        let settings = object_map
            .entry("settings")
            .or_insert_with(|| serde_json::json!({}))
            .as_object_mut()
            .expect("Always valid");
        settings.insert(
            "optimizer".to_owned(),
            serde_json::json!({
                "mode": self.optimizer_settings.middle_end_as_char().to_string(),
                "sizeFallback": self.optimizer_settings.is_fallback_to_size_enabled(),
            }),
        );
        if !self.llvm_options.is_empty() {
            settings.insert("llvmOptions".to_owned(), self.llvm_options.into());
        }
        object.sort_all_objects();
        serde_json::to_string(&object).expect("Always valid")
    }
}
