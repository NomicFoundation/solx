//!
//! Solidity contract build.
//!

pub mod object;

use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use normpath::PathExt;

use self::object::Object;

///
/// Solidity contract build.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Contract {
    /// Contract name.
    pub name: solx_utils::ContractName,
    /// Deploy code object compilation result.
    pub deploy_object_result: Option<crate::Result<Object>>,
    /// Runtime code object.
    pub runtime_object_result: Option<crate::Result<Object>>,
    /// Combined `solc` and `solx` metadata.
    pub metadata: Option<String>,
    /// solc ABI.
    pub abi: Option<serde_json::Value>,
    /// solc method identifiers.
    pub method_identifiers: Option<BTreeMap<String, String>>,
    /// solc user documentation.
    pub userdoc: Option<serde_json::Value>,
    /// solc developer documentation.
    pub devdoc: Option<serde_json::Value>,
    /// solc storage layout.
    pub storage_layout: Option<serde_json::Value>,
    /// solc transient storage layout.
    pub transient_storage_layout: Option<serde_json::Value>,
    /// solc EVM legacy assembly.
    pub legacy_assembly: Option<solx_evm_assembly::Assembly>,
    /// solc Yul IR.
    pub yul: Option<String>,
}

impl Contract {
    ///
    /// A shortcut constructor.
    ///
    pub fn new(
        name: solx_utils::ContractName,
        deploy_object_result: Option<crate::Result<Object>>,
        runtime_object_result: Option<crate::Result<Object>>,
        metadata: Option<String>,
        abi: Option<serde_json::Value>,
        method_identifiers: Option<BTreeMap<String, String>>,
        userdoc: Option<serde_json::Value>,
        devdoc: Option<serde_json::Value>,
        storage_layout: Option<serde_json::Value>,
        transient_storage_layout: Option<serde_json::Value>,
        legacy_assembly: Option<solx_evm_assembly::Assembly>,
        yul: Option<String>,
    ) -> Self {
        Self {
            name,
            deploy_object_result,
            runtime_object_result,
            metadata,
            abi,
            method_identifiers,
            userdoc,
            devdoc,
            storage_layout,
            transient_storage_layout,
            legacy_assembly,
            yul,
        }
    }

    ///
    /// Writes the contract text assembly and bytecode to terminal.
    ///
    pub fn write_to_terminal(
        mut self,
        output_selection: &solx_standard_json::InputSelection,
    ) -> anyhow::Result<()> {
        writeln!(
            std::io::stdout(),
            "\n======= {} =======",
            self.name.full_path
        )?;

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::EVMLegacyAssembly,
        ) {
            let legacy_assembly = self.legacy_assembly.take().expect("Always exists");
            writeln!(std::io::stdout(), "EVM assembly:\n{legacy_assembly}")?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeLLVMAssembly,
            )
        {
            let deploy_assembly = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .assembly
                .take()
                .expect("Always exists");
            writeln!(
                std::io::stdout(),
                "Deploy LLVM EVM assembly:\n{deploy_assembly}"
            )?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
            )
        {
            let runtime_assembly = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .assembly
                .take()
                .expect("Always exists");
            writeln!(
                std::io::stdout(),
                "Runtime LLVM EVM assembly:\n{runtime_assembly}"
            )?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeObject,
            )
        {
            let bytecode_hex = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .bytecode_hex
                .take()
                .expect("Always exists");
            writeln!(std::io::stdout(), "Binary:\n{bytecode_hex}")?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeObject,
            )
        {
            let bytecode_hex = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .bytecode_hex
                .take()
                .expect("Always exists");
            writeln!(
                std::io::stdout(),
                "Binary of the runtime part:\n{bytecode_hex}"
            )?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeDebugInfo,
            )
        {
            let debug_info = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .debug_info
                .take()
                .map(hex::encode)
                .expect("Always exists");
            writeln!(std::io::stdout(), "Debug info:\n{debug_info}")?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
            )
        {
            let debug_info = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .debug_info
                .take()
                .map(hex::encode)
                .expect("Always valid");
            writeln!(
                std::io::stdout(),
                "Debug info of the runtime part:\n{debug_info}"
            )?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::Yul,
        ) {
            let yul = self.yul.take().expect("Always exists");
            writeln!(std::io::stdout(), "IR:\n{yul}")?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeEVMLA,
            )
            && let Some(evmla) = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .evmla
                .take()
        {
            writeln!(std::io::stdout(), "Deploy EVM legacy assembly:\n{evmla}")?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeEVMLA,
            )
            && let Some(evmla) = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .evmla
                .take()
        {
            writeln!(std::io::stdout(), "Runtime EVM legacy assembly:\n{evmla}")?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeEthIR,
            )
            && let Some(ethir) = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .ethir
                .take()
        {
            writeln!(std::io::stdout(), "Deploy Ethereal IR:\n{ethir}")?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeEthIR,
            )
            && let Some(ethir) = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .ethir
                .take()
        {
            writeln!(std::io::stdout(), "Runtime Ethereal IR:\n{ethir}")?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
            )
            && let Some(llvm_ir) = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .llvm_ir_unoptimized
                .take()
        {
            writeln!(
                std::io::stdout(),
                "Deploy LLVM IR (unoptimized):\n{llvm_ir}"
            )?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
            )
            && let Some(llvm_ir) = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .llvm_ir_unoptimized
                .take()
        {
            writeln!(
                std::io::stdout(),
                "Runtime LLVM IR (unoptimized):\n{llvm_ir}"
            )?;
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeLLVMIR,
            )
            && let Some(llvm_ir) = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .llvm_ir
                .take()
        {
            writeln!(std::io::stdout(), "Deploy LLVM IR:\n{llvm_ir}")?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
            )
            && let Some(llvm_ir) = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .llvm_ir
                .take()
        {
            writeln!(std::io::stdout(), "Runtime LLVM IR:\n{llvm_ir}")?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::MethodIdentifiers,
        ) {
            writeln!(std::io::stdout(), "Function signatures:")?;
            for (signature, identifier) in
                self.method_identifiers.expect("Always exists").into_iter()
            {
                writeln!(std::io::stdout(), "{identifier}: {signature}")?;
            }
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::Metadata,
        ) {
            writeln!(
                std::io::stdout(),
                "Metadata:\n{}",
                self.metadata.expect("Always exists")
            )?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::ABI,
        ) {
            writeln!(
                std::io::stdout(),
                "Contract JSON ABI:\n{}",
                self.abi.expect("Always exists")
            )?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::StorageLayout,
        ) {
            writeln!(
                std::io::stdout(),
                "Contract Storage Layout:\n{}",
                self.storage_layout.expect("Always exists")
            )?;
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::TransientStorageLayout,
        ) {
            writeln!(
                std::io::stdout(),
                "Contract Transient Storage Layout:\n{}",
                self.transient_storage_layout.expect("Always exists")
            )?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::DeveloperDocumentation,
        ) {
            writeln!(
                std::io::stdout(),
                "Developer Documentation:\n{}",
                self.devdoc.expect("Always exists")
            )?;
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::UserDocumentation,
        ) {
            writeln!(
                std::io::stdout(),
                "User Documentation:\n{}",
                self.userdoc.expect("Always exists")
            )?;
        }
        if let (Some(deploy_object_result), Some(runtime_object_result)) =
            (self.deploy_object_result, self.runtime_object_result)
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::Benchmarks,
            )
        {
            writeln!(std::io::stdout(), "Benchmarks:")?;
            for (name, value) in deploy_object_result
                .expect("Always exists")
                .benchmarks
                .into_iter()
            {
                writeln!(std::io::stdout(), "    {name}: {value}ms")?;
            }
            for (name, value) in runtime_object_result
                .expect("Always exists")
                .benchmarks
                .into_iter()
            {
                writeln!(std::io::stdout(), "    {name}: {value}ms")?;
            }
        }

        Ok(())
    }

    ///
    /// Writes the contract text assembly and bytecode to files.
    ///
    pub fn write_to_directory(
        mut self,
        output_directory: &Path,
        output_selection: &solx_standard_json::InputSelection,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        let contract_path = PathBuf::from(self.name.path.as_str());
        let contract_name = contract_path
            .file_name()
            .expect("Always exists")
            .to_str()
            .expect("Always valid");
        let contract_path = contract_path.normalize()?;
        let contract_path = if contract_path.starts_with(std::env::current_dir()?) {
            contract_path
                .as_path()
                .strip_prefix(std::env::current_dir()?)?
        } else {
            contract_path.as_path()
        }
        .to_string_lossy()
        .replace(['\\', '/', '.'], "_");

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeObject,
            )
        {
            let output_name = format!(
                "{contract_path}_{}.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_EVM_BINARY
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let bytecode_hex = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .bytecode_hex
                .take()
                .expect("Always exists");
            Self::write_to_file(output_path.as_path(), bytecode_hex, overwrite)?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeObject,
            )
        {
            let output_name = format!(
                "{contract_path}_{}.{}-{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_EVM_BINARY,
                solx_utils::CodeSegment::Runtime,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let bytecode_hex = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .bytecode_hex
                .take()
                .expect("Always exists");
            Self::write_to_file(output_path.as_path(), bytecode_hex, overwrite)?;
        }

        if let (Some(deploy_object_result), Some(runtime_object_result)) = (
            self.deploy_object_result.as_mut(),
            self.runtime_object_result.as_mut(),
        ) && output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::BytecodeLLVMAssembly,
        ) {
            for (object, code_segment) in [
                deploy_object_result.as_mut(),
                runtime_object_result.as_mut(),
            ]
            .iter_mut()
            .zip([
                solx_utils::CodeSegment::Deploy,
                solx_utils::CodeSegment::Runtime,
            ]) {
                let output_name = format!(
                    "{contract_path}_{}_llvm.{}{}",
                    self.name.name.as_deref().unwrap_or(contract_name),
                    solx_utils::EXTENSION_EVM_ASSEMBLY,
                    match code_segment {
                        solx_utils::CodeSegment::Deploy => "".to_owned(),
                        solx_utils::CodeSegment::Runtime => format!("-{code_segment}"),
                    },
                );
                let mut output_path = output_directory.to_owned();
                output_path.push(output_name.as_str());

                let assembly = object
                    .as_mut()
                    .expect("Always exists")
                    .assembly
                    .take()
                    .expect("Always exists");
                Self::write_to_file(output_path.as_path(), assembly, overwrite)?;
            }
        }

        if let Some(deploy_object_result) = self.deploy_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::BytecodeDebugInfo,
            )
        {
            let output_name = format!(
                "{contract_path}_{}.dbg.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_EVM_BINARY
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let debug_info = deploy_object_result
                .as_mut()
                .expect("Always exists")
                .debug_info
                .take()
                .expect("Always exists");
            Self::write_to_file(output_path.as_path(), debug_info, overwrite)?;
        }
        if let Some(runtime_object_result) = self.runtime_object_result.as_mut()
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
            )
        {
            let output_name = format!(
                "{contract_path}_{}.dbg.{}-{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_EVM_BINARY,
                solx_utils::CodeSegment::Runtime,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let debug_info = runtime_object_result
                .as_mut()
                .expect("Always exists")
                .debug_info
                .take()
                .expect("Always exists");
            Self::write_to_file(output_path.as_path(), debug_info, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::Metadata,
        ) {
            let output_name = format!(
                "{contract_path}_{}_meta.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_JSON,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let metadata = self.metadata.take().expect("Always exists");
            Self::write_to_file(output_path.as_path(), metadata, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::ABI,
        ) {
            let output_name = format!(
                "{contract_path}_{}.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_SOLIDITY_ABI,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let abi = self.abi.take().expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), abi, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::MethodIdentifiers,
        ) {
            let output_name = format!(
                "{contract_path}_{}.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_SOLIDITY_SIGNATURES,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let mut output = "Function signatures:\n".to_owned();
            for (signature, identifier) in
                self.method_identifiers.expect("Always exists").into_iter()
            {
                output.push_str(format!("{identifier}: {signature}\n").as_str());
            }
            Self::write_to_file(output_path.as_path(), output, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::StorageLayout,
        ) {
            let output_name = format!(
                "{contract_path}_{}_storage.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_JSON,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let storage_layout = self.storage_layout.expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), storage_layout, overwrite)?;
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::TransientStorageLayout,
        ) {
            let output_name = format!(
                "{contract_path}_{}_transient_storage.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_JSON,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let transient_storage_layout = self
                .transient_storage_layout
                .expect("Always exists")
                .to_string();

            Self::write_to_file(output_path.as_path(), transient_storage_layout, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::DeveloperDocumentation,
        ) {
            let output_name = format!(
                "{contract_path}_{}.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_SOLIDITY_DOCDEV,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let devdoc = self.devdoc.expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), devdoc, overwrite)?;
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::UserDocumentation,
        ) {
            let output_name = format!(
                "{contract_path}_{}.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_SOLIDITY_DOCUSER,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let userdoc = self.userdoc.expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), userdoc, overwrite)?;
        }

        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::EVMLegacyAssembly,
        ) {
            let output_name = format!(
                "{contract_path}_{}_evm.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_JSON,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let legacy_assembly = self.legacy_assembly.expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), legacy_assembly, overwrite)?;
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::Yul,
        ) {
            let output_name = format!(
                "{contract_path}_{}_opt.{}",
                self.name.name.as_deref().unwrap_or(contract_name),
                solx_utils::EXTENSION_YUL,
            );
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let yul = self.yul.expect("Always exists").to_string();
            Self::write_to_file(output_path.as_path(), yul, overwrite)?;
        }
        if let (Some(deploy_object_result), Some(runtime_object_result)) =
            (self.deploy_object_result, self.runtime_object_result)
            && output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::Benchmarks,
            )
        {
            let output_name = format!("{contract_path}_benchmarks.txt",);
            let mut output_path = output_directory.to_owned();
            output_path.push(output_name.as_str());

            let mut output = String::with_capacity(4096);
            output.push_str("Benchmarks:\n");
            for (name, value) in deploy_object_result
                .as_ref()
                .expect("Always exists")
                .benchmarks
                .iter()
            {
                output.push_str(format!("{name}: {value}ms\n").as_str());
            }
            for (name, value) in runtime_object_result
                .as_ref()
                .expect("Always exists")
                .benchmarks
                .iter()
            {
                output.push_str(format!("{name}: {value}ms\n").as_str());
            }
            Self::write_to_file(output_path.as_path(), output, overwrite)?;
        }

        Ok(())
    }

    ///
    /// Writes the contract text assembly and bytecode to the standard JSON.
    ///
    pub fn write_to_standard_json(
        mut self,
        standard_json_contract: &mut solx_standard_json::OutputContract,
        output_selection: &solx_standard_json::InputSelection,
        is_bytecode_linked: bool,
    ) {
        if let Some(value) = self.metadata.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::Metadata,
            )
        }) {
            standard_json_contract.metadata = Some(value);
        }
        if let Some(value) = self.abi.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::ABI,
            )
        }) {
            standard_json_contract.abi = Some(value);
        }
        if let Some(value) = self.userdoc.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::UserDocumentation,
            )
        }) {
            standard_json_contract.userdoc = Some(value);
        }
        if let Some(value) = self.devdoc.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::DeveloperDocumentation,
            )
        }) {
            standard_json_contract.devdoc = Some(value);
        }
        if let Some(value) = self.storage_layout.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::StorageLayout,
            )
        }) {
            standard_json_contract.storage_layout = Some(value);
        }
        if let Some(value) = self.transient_storage_layout.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::TransientStorageLayout,
            )
        }) {
            standard_json_contract.transient_storage_layout = Some(value);
        }
        if let Some(value) = self.yul.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::Yul,
            )
        }) {
            standard_json_contract.ir = Some(value);
        }

        let evm = standard_json_contract
            .evm
            .get_or_insert_with(solx_standard_json::OutputContractEVM::default);
        if let Some(value) = self.method_identifiers.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::MethodIdentifiers,
            )
        }) {
            evm.method_identifiers = Some(value);
        }
        if let Some(value) = self.legacy_assembly.take().filter(|_| {
            output_selection.check_selection(
                self.name.path.as_str(),
                self.name.name.as_deref(),
                solx_standard_json::InputSelector::EVMLegacyAssembly,
            )
        }) {
            evm.legacy_assembly = Some(value);
        }
        if output_selection.check_selection(
            self.name.path.as_str(),
            self.name.name.as_deref(),
            solx_standard_json::InputSelector::GasEstimates,
        ) {
            evm.gas_estimates = Some(serde_json::json!({}));
        }

        evm.bytecode = Some(Self::build_bytecode_output(
            &mut self.deploy_object_result,
            self.name.path.as_str(),
            self.name.name.as_deref(),
            output_selection,
            is_bytecode_linked,
            solx_utils::CodeSegment::Deploy,
        ));

        evm.deployed_bytecode = Some(Self::build_bytecode_output(
            &mut self.runtime_object_result,
            self.name.path.as_str(),
            self.name.name.as_deref(),
            output_selection,
            is_bytecode_linked,
            solx_utils::CodeSegment::Runtime,
        ));
    }

    ///
    /// Builds the bytecode output for a single code segment.
    ///
    fn build_bytecode_output(
        object_result: &mut Option<crate::Result<Object>>,
        path: &str,
        name: Option<&str>,
        output_selection: &solx_standard_json::InputSelection,
        is_bytecode_linked: bool,
        code_segment: solx_utils::CodeSegment,
    ) -> solx_standard_json::OutputContractEVMBytecode {
        let (
            selector_object,
            selector_evmla,
            selector_ethir,
            selector_llvm_ir_unoptimized,
            selector_llvm_ir,
            selector_llvm_assembly,
            selector_debug_info,
            selector_link_references,
            selector_opcodes,
            selector_source_map,
            selector_function_debug_data,
            selector_generated_sources,
            selector_immutable_references,
        ) = match code_segment {
            solx_utils::CodeSegment::Deploy => (
                solx_standard_json::InputSelector::BytecodeObject,
                solx_standard_json::InputSelector::BytecodeEVMLA,
                solx_standard_json::InputSelector::BytecodeEthIR,
                solx_standard_json::InputSelector::BytecodeLLVMIRUnoptimized,
                solx_standard_json::InputSelector::BytecodeLLVMIR,
                solx_standard_json::InputSelector::BytecodeLLVMAssembly,
                solx_standard_json::InputSelector::BytecodeDebugInfo,
                solx_standard_json::InputSelector::BytecodeLinkReferences,
                solx_standard_json::InputSelector::BytecodeOpcodes,
                solx_standard_json::InputSelector::BytecodeSourceMap,
                solx_standard_json::InputSelector::BytecodeFunctionDebugData,
                solx_standard_json::InputSelector::BytecodeGeneratedSources,
                None,
            ),
            solx_utils::CodeSegment::Runtime => (
                solx_standard_json::InputSelector::RuntimeBytecodeObject,
                solx_standard_json::InputSelector::RuntimeBytecodeEVMLA,
                solx_standard_json::InputSelector::RuntimeBytecodeEthIR,
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMIRUnoptimized,
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMIR,
                solx_standard_json::InputSelector::RuntimeBytecodeLLVMAssembly,
                solx_standard_json::InputSelector::RuntimeBytecodeDebugInfo,
                solx_standard_json::InputSelector::RuntimeBytecodeLinkReferences,
                solx_standard_json::InputSelector::RuntimeBytecodeOpcodes,
                solx_standard_json::InputSelector::RuntimeBytecodeSourceMap,
                solx_standard_json::InputSelector::RuntimeBytecodeFunctionDebugData,
                solx_standard_json::InputSelector::RuntimeBytecodeGeneratedSources,
                Some(solx_standard_json::InputSelector::RuntimeBytecodeImmutableReferences),
            ),
        };

        solx_standard_json::OutputContractEVMBytecode::new(
            // object
            if is_bytecode_linked {
                object_result
                    .as_mut()
                    .and_then(|result| result.as_mut().ok())
                    .map(|object| {
                        object.bytecode_hex.take().filter(|_| {
                            output_selection.check_selection(path, name, selector_object)
                        })
                    })
                    .unwrap_or(Some(String::new()))
            } else {
                None
            },
            // evmla
            object_result.as_mut().and_then(|result| {
                result
                    .as_mut()
                    .ok()?
                    .evmla
                    .take()
                    .filter(|_| output_selection.check_selection(path, name, selector_evmla))
            }),
            // ethir
            object_result.as_mut().and_then(|result| {
                result
                    .as_mut()
                    .ok()?
                    .ethir
                    .take()
                    .filter(|_| output_selection.check_selection(path, name, selector_ethir))
            }),
            // llvm_ir_unoptimized
            object_result.as_mut().and_then(|result| {
                result
                    .as_mut()
                    .ok()?
                    .llvm_ir_unoptimized
                    .take()
                    .filter(|_| {
                        output_selection.check_selection(path, name, selector_llvm_ir_unoptimized)
                    })
            }),
            // llvm_ir
            object_result.as_mut().and_then(|result| {
                result
                    .as_mut()
                    .ok()?
                    .llvm_ir
                    .take()
                    .filter(|_| output_selection.check_selection(path, name, selector_llvm_ir))
            }),
            // llvm_assembly
            object_result.as_mut().and_then(|result| {
                result.as_mut().ok()?.assembly.take().filter(|_| {
                    output_selection.check_selection(path, name, selector_llvm_assembly)
                })
            }),
            // debug_info
            object_result
                .as_mut()
                .and_then(|result| result.as_mut().ok())
                .map(|object| {
                    object.debug_info.take().map(hex::encode).filter(|_| {
                        output_selection.check_selection(path, name, selector_debug_info)
                    })
                })
                .unwrap_or(Some(String::new())),
            // unlinked_symbols (link_references)
            if is_bytecode_linked
                && output_selection.check_selection(path, name, selector_link_references)
            {
                Some(
                    object_result
                        .as_ref()
                        .and_then(|result| result.as_ref().ok())
                        .map(|object| object.unlinked_symbols.to_owned())
                        .unwrap_or_default(),
                )
            } else {
                None
            },
            // benchmarks
            if output_selection.check_selection(
                path,
                name,
                solx_standard_json::InputSelector::Benchmarks,
            ) {
                object_result
                    .as_mut()
                    .and_then(|result| result.as_mut().ok())
                    .map(|object| object.benchmarks.drain(..).collect())
                    .unwrap_or_default()
            } else {
                vec![]
            },
            // opcodes
            if output_selection.check_selection(path, name, selector_opcodes) {
                Some(String::new())
            } else {
                None
            },
            // source_map
            if output_selection.check_selection(path, name, selector_source_map) {
                Some(String::new())
            } else {
                None
            },
            // function_debug_data
            if output_selection.check_selection(path, name, selector_function_debug_data) {
                Some(BTreeMap::new())
            } else {
                None
            },
            // generated_sources
            if output_selection.check_selection(path, name, selector_generated_sources) {
                Some(Vec::new())
            } else {
                None
            },
            // immutable_references
            selector_immutable_references.and_then(|selector| {
                if output_selection.check_selection(path, name, selector) {
                    Some(serde_json::json!({}))
                } else {
                    None
                }
            }),
        )
    }

    ///
    /// Writes data to the file, checking the `overwrite` flag.
    ///
    pub fn write_to_file<C: AsRef<[u8]>>(
        output_path: &Path,
        data: C,
        overwrite: bool,
    ) -> anyhow::Result<()> {
        if output_path.exists() && !overwrite {
            anyhow::bail!(
                "Refusing to overwrite an existing file {output_path:?} (use --overwrite to force)."
            );
        } else {
            std::fs::write(output_path, data)
                .map_err(|error| anyhow::anyhow!("File {output_path:?} writing: {error}"))?;
        }
        Ok(())
    }

    ///
    /// Returns references to objects.
    ///
    pub fn objects_ref(&self) -> Vec<&Object> {
        let mut objects = Vec::with_capacity(2);
        if let Some(deploy_object) = self.deploy_object_result.as_ref() {
            objects.push(deploy_object.as_ref().expect("Always exists"));
        }
        if let Some(runtime_object) = self.runtime_object_result.as_ref() {
            objects.push(runtime_object.as_ref().expect("Always exists"));
        }
        objects
    }

    ///
    /// Returns mutable references to objects.
    ///
    pub fn objects_mut(&mut self) -> Vec<&mut Object> {
        let mut objects = Vec::with_capacity(2);
        if let Some(deploy_object) = self.deploy_object_result.as_mut() {
            objects.push(deploy_object.as_mut().expect("Always exists"));
        }
        if let Some(runtime_object) = self.runtime_object_result.as_mut() {
            objects.push(runtime_object.as_mut().expect("Always exists"));
        }
        objects
    }

    ///
    /// Returns a mutable reference to the specified object.
    ///
    pub fn object_mut_by_code_segment(
        &mut self,
        code_segment: solx_utils::CodeSegment,
    ) -> Option<&mut Object> {
        match code_segment {
            solx_utils::CodeSegment::Deploy => Some(
                self.deploy_object_result
                    .as_mut()?
                    .as_mut()
                    .expect("Always exists"),
            ),
            solx_utils::CodeSegment::Runtime => Some(
                self.runtime_object_result
                    .as_mut()?
                    .as_mut()
                    .expect("Always exists"),
            ),
        }
    }
}
