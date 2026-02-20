//!
//! The contract EVM legacy assembly source code.
//!

///
/// The contract EVM legacy assembly source code.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct EVMLegacyAssembly {
    /// The EVM legacy assembly source code.
    pub assembly: solx_evm_assembly::Assembly,
    /// Dependencies of the EVM assembly object.
    pub dependencies: solx_codegen_evm::Dependencies,
    /// Runtime code object that is only set in deploy code.
    pub runtime_code: Option<Box<Self>>,
}

impl EVMLegacyAssembly {
    ///
    /// Transforms the `solc` standard JSON output contract into an EVM legacy assembly object.
    ///
    pub fn from_contract(
        full_path: &str,
        mut assembly: solx_evm_assembly::Assembly,
        extra_metadata: Option<solx_evm_assembly::ExtraMetadata>,
    ) -> anyhow::Result<Self> {
        assembly.extra_metadata = extra_metadata.clone();
        if let Ok(runtime_code) = assembly.runtime_code_mut() {
            runtime_code.extra_metadata = extra_metadata;
        }

        let runtime_code_assembly = assembly
            .runtime_code()
            .map_err(|error| {
                anyhow::anyhow!(
                    "EVM legacy assembly is missing runtime code for `{full_path}`: {error}"
                )
            })?
            .to_owned();
        let runtime_code_identifier = format!("{full_path}.{}", solx_utils::CodeSegment::Runtime);
        let mut runtime_code_dependencies =
            solx_codegen_evm::Dependencies::new(runtime_code_identifier.as_str());
        runtime_code_assembly.accumulate_evm_dependencies(&mut runtime_code_dependencies);
        let runtime_code = Some(Box::new(Self {
            assembly: runtime_code_assembly,
            dependencies: runtime_code_dependencies,
            runtime_code: None,
        }));

        let mut deploy_code_dependencies = solx_codegen_evm::Dependencies::new(full_path);
        assembly.accumulate_evm_dependencies(&mut deploy_code_dependencies);

        Ok(Self {
            assembly,
            dependencies: deploy_code_dependencies,
            runtime_code,
        })
    }
}
