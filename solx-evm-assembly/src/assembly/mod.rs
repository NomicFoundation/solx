//!
//! The `solc --asm-json` output.
//!

pub mod data;
pub mod instruction;

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use twox_hash::XxHash3_64;

use solx_codegen_evm::IContext;

use crate::ethereal_ir::EtherealIR;
use crate::extra_metadata::ExtraMetadata;

use self::data::Data;
use self::instruction::name::Name as InstructionName;
use self::instruction::Instruction;

///
/// The JSON assembly.
///
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Assembly {
    /// The metadata string.
    #[serde(rename = ".auxdata", default, skip_serializing_if = "Option::is_none")]
    pub auxdata: Option<String>,
    /// The deploy code instructions.
    #[serde(rename = ".code", default, skip_serializing_if = "Option::is_none")]
    pub code: Option<Vec<Instruction>>,
    /// The runtime code.
    #[serde(rename = ".data", default, skip_serializing_if = "Option::is_none")]
    pub data: Option<BTreeMap<String, Data>>,

    /// The full contract path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_path: Option<String>,
    /// The EVM legacy assembly extra metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_metadata: Option<ExtraMetadata>,
}

impl Assembly {
    /// The default serializing/deserializing buffer size.
    pub const DEFAULT_SERDE_BUFFER_SIZE: usize = 1048576;

    ///
    /// Sets the full contract path.
    ///
    pub fn set_full_path(&mut self, full_path: String) {
        self.full_path = Some(full_path);
    }

    ///
    /// Returns the full contract path if it is set, or `<undefined>` otherwise.
    ///
    /// # Panics
    /// If the `full_path` has not been set.
    ///
    pub fn full_path(&self) -> &str {
        self.full_path
            .as_deref()
            .unwrap_or_else(|| panic!("The full path of some contracts is unset"))
    }

    ///
    /// Returns a runtime code reference from the deploy code assembly.
    ///
    pub fn runtime_code(&self) -> anyhow::Result<&Assembly> {
        match self
            .data
            .as_ref()
            .and_then(|data| data.get("0"))
            .ok_or_else(|| anyhow::anyhow!("Runtime code data not found"))?
        {
            Data::Assembly(assembly) => Ok(assembly),
            Data::Hash(hash) => {
                anyhow::bail!("Expected runtime code, found hash `{hash}`");
            }
            Data::Path(path) => {
                anyhow::bail!("Expected runtime code, found path `{path}`");
            }
        }
    }

    ///
    /// Returns a runtime code mutable reference from the deploy code assembly.
    ///
    pub fn runtime_code_mut(&mut self) -> anyhow::Result<&mut Assembly> {
        match self
            .data
            .as_mut()
            .and_then(|data| data.get_mut("0"))
            .ok_or_else(|| anyhow::anyhow!("Runtime code data not found"))?
        {
            Data::Assembly(assembly) => Ok(assembly),
            Data::Hash(hash) => {
                anyhow::bail!("Expected runtime code, found hash `{hash}`");
            }
            Data::Path(path) => {
                anyhow::bail!("Expected runtime code, found path `{path}`");
            }
        }
    }

    ///
    /// Get the list of unlinked deployable libraries.
    ///
    pub fn get_unlinked_libraries(&self) -> BTreeSet<String> {
        let mut unlinked_libraries = BTreeSet::new();
        if let Some(code) = self.code.as_ref() {
            for instruction in code.iter() {
                if let InstructionName::PUSHLIB = instruction.name {
                    let library_path = instruction.value.to_owned().expect("Always exists");
                    unlinked_libraries.insert(library_path);
                }
            }
        }
        if let Some(data) = self.data.as_ref() {
            for (_, data) in data.iter() {
                unlinked_libraries.extend(data.get_unlinked_libraries());
            }
        }
        unlinked_libraries
    }

    ///
    /// Get the list of EVM dependencies.
    ///
    pub fn accumulate_evm_dependencies(&self, dependencies: &mut solx_codegen_evm::Dependencies) {
        if let Some(code) = self.code.as_ref() {
            for instruction in code.iter() {
                match instruction.name {
                    InstructionName::PUSH_DataOffset | InstructionName::PUSH_DataSize => {
                        let dependency = instruction.value.to_owned().expect("Always exists");
                        let is_runtime_code = dependencies.identifier
                            == dependency
                                .strip_suffix(
                                    format!(".{}", solx_utils::CodeSegment::Runtime).as_str(),
                                )
                                .unwrap_or(dependencies.identifier.as_str());
                        dependencies.push(dependency, is_runtime_code);
                    }
                    _ => {}
                }
            }
        }
    }

    ///
    /// Returns the `blake3` hash of the assembly representation.
    ///
    pub fn hash(&self) -> u64 {
        let mut preimage: Vec<u8> = Vec::with_capacity(Self::DEFAULT_SERDE_BUFFER_SIZE);
        ciborium::into_writer(&self, &mut preimage).expect("Always valid");
        XxHash3_64::oneshot(preimage.as_slice())
    }

    ///
    /// Replaces with dependency indexes with actual data.
    ///
    pub fn preprocess_dependencies(
        contracts: BTreeMap<String, BTreeMap<String, &mut Self>>,
    ) -> anyhow::Result<()> {
        let mut hash_path_mapping = BTreeMap::new();

        for (path, file) in contracts.iter() {
            for (name, deploy_code_assembly) in file.iter() {
                let deploy_code_path = format!("{path}:{name}");
                let deploy_code_hash = deploy_code_assembly.hash();

                let runtime_code_path =
                    format!("{path}:{name}.{}", solx_utils::CodeSegment::Runtime);
                let runtime_code_assembly = deploy_code_assembly.runtime_code()?;
                let runtime_code_hash = runtime_code_assembly.hash();

                hash_path_mapping.insert(deploy_code_hash, deploy_code_path);
                hash_path_mapping.insert(runtime_code_hash, runtime_code_path);
            }
        }

        let mut assemblies = BTreeMap::new();
        for (path, file) in contracts.into_iter() {
            for (name, assembly) in file.into_iter() {
                let full_path = format!("{path}:{name}");
                assemblies.insert(full_path, assembly);
            }
        }
        assemblies
            .into_par_iter()
            .map(|(full_path, assembly)| {
                Self::preprocess_dependency_level(
                    full_path.as_str(),
                    assembly,
                    &hash_path_mapping,
                )?;
                Ok(())
            })
            .collect::<anyhow::Result<()>>()?;

        Ok(())
    }

    ///
    /// Preprocesses an assembly JSON structure dependency data map.
    ///
    fn preprocess_dependency_level(
        full_path: &str,
        assembly: &mut Assembly,
        hash_path_mapping: &BTreeMap<u64, String>,
    ) -> anyhow::Result<()> {
        assembly.set_full_path(full_path.to_owned());

        let deploy_code_index_path_mapping =
            assembly.deploy_dependencies_pass(full_path, hash_path_mapping)?;
        if let Some(deploy_code_instructions) = assembly.code.as_deref_mut() {
            Instruction::replace_data_aliases(
                deploy_code_instructions,
                &deploy_code_index_path_mapping,
            )?;
        };

        let runtime_code_index_path_mapping =
            assembly.runtime_dependencies_pass(hash_path_mapping)?;
        if let Some(runtime_code_instructions) = assembly
            .data
            .as_mut()
            .and_then(|data_map| data_map.get_mut("0"))
            .and_then(|data| data.get_assembly_mut())
            .and_then(|assembly| assembly.code.as_deref_mut())
        {
            Instruction::replace_data_aliases(
                runtime_code_instructions,
                &runtime_code_index_path_mapping,
            )?;
        }

        Ok(())
    }

    ///
    /// Replaces the deploy code dependencies with full contract path and returns the list.
    ///
    fn deploy_dependencies_pass(
        &mut self,
        full_path: &str,
        hash_data_mapping: &BTreeMap<u64, String>,
    ) -> anyhow::Result<BTreeMap<String, String>> {
        let mut index_path_mapping = BTreeMap::new();
        let index = "0".repeat(solx_utils::BYTE_LENGTH_FIELD * 2);
        index_path_mapping.insert(
            index,
            format!("{full_path}.{}", solx_utils::CodeSegment::Runtime),
        );

        let dependencies = match self.data.as_mut() {
            Some(dependencies) => dependencies,
            None => return Ok(index_path_mapping),
        };
        for (index, data) in dependencies.iter_mut() {
            if index == "0" {
                continue;
            }

            let mut index_extended = "0".repeat(solx_utils::BYTE_LENGTH_FIELD * 2 - index.len());
            index_extended.push_str(index.as_str());

            *data = match data {
                Data::Assembly(assembly) => {
                    let hash = assembly.hash();
                    let full_path = hash_data_mapping.get(&hash).cloned().ok_or_else(|| {
                        anyhow::anyhow!("Contract path not found for hash `{hash}`")
                    })?;

                    index_path_mapping.insert(index_extended, full_path.clone());
                    Data::Path(full_path)
                }
                Data::Hash(hash) => {
                    index_path_mapping.insert(index_extended, hash.to_owned());
                    continue;
                }
                _ => continue,
            };
        }

        Ok(index_path_mapping)
    }

    ///
    /// Replaces the runtime code dependencies with full contract path and returns the list.
    ///
    fn runtime_dependencies_pass(
        &mut self,
        hash_data_mapping: &BTreeMap<u64, String>,
    ) -> anyhow::Result<BTreeMap<String, String>> {
        let mut index_path_mapping = BTreeMap::new();

        let dependencies = match self
            .data
            .as_mut()
            .and_then(|data| data.get_mut("0"))
            .and_then(|data| data.get_assembly_mut())
            .and_then(|assembly| assembly.data.as_mut())
        {
            Some(dependencies) => dependencies,
            None => return Ok(index_path_mapping),
        };
        for (index, data) in dependencies.iter_mut() {
            let mut index_extended = "0".repeat(solx_utils::BYTE_LENGTH_FIELD * 2 - index.len());
            index_extended.push_str(index.as_str());

            *data = match data {
                Data::Assembly(assembly) => {
                    let hash = Assembly::hash(assembly);
                    let full_path = hash_data_mapping.get(&hash).cloned().ok_or_else(|| {
                        anyhow::anyhow!("Contract path not found for hash `{hash}`")
                    })?;

                    index_path_mapping.insert(index_extended, full_path.clone());
                    Data::Path(full_path)
                }
                Data::Hash(hash) => {
                    index_path_mapping.insert(index_extended, hash.to_owned());
                    continue;
                }
                _ => continue,
            };
        }

        Ok(index_path_mapping)
    }
}

impl solx_codegen_evm::WriteLLVM for Assembly {
    fn declare(&mut self, _context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        Ok(())
    }

    fn into_llvm(self, context: &mut solx_codegen_evm::Context) -> anyhow::Result<()> {
        let full_path = self.full_path().to_owned();

        let (code_segment, blocks) = if let Ok(runtime_code) = self.runtime_code() {
            if let Some(debug_config) = context.debug_config() {
                debug_config.dump_evmla(full_path.as_str(), self.to_string().as_str())?;
            }

            let deploy_code_blocks = EtherealIR::get_blocks(
                context.evmla().expect("Always exists").version.to_owned(),
                solx_utils::CodeSegment::Deploy,
                self.code
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Deploy code instructions not found"))?,
            )?;

            let runtime_code_instructions = runtime_code
                .code
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Runtime code instructions not found"))?;
            let runtime_code_blocks = EtherealIR::get_blocks(
                context.evmla().expect("Always exists").version.to_owned(),
                solx_utils::CodeSegment::Runtime,
                runtime_code_instructions.as_slice(),
            )?;

            let mut blocks = deploy_code_blocks;
            blocks.extend(runtime_code_blocks);
            (solx_utils::CodeSegment::Deploy, blocks)
        } else {
            if let Some(debug_config) = context.debug_config() {
                debug_config.dump_evmla(
                    format!("{full_path}.{}", solx_utils::CodeSegment::Runtime).as_str(),
                    self.to_string().as_str(),
                )?;
            }

            let blocks = EtherealIR::get_blocks(
                context.evmla().expect("Always exists").version.to_owned(),
                solx_utils::CodeSegment::Runtime,
                self.code
                    .as_deref()
                    .ok_or_else(|| anyhow::anyhow!("Deploy code instructions not found"))?,
            )?;
            (solx_utils::CodeSegment::Runtime, blocks)
        };

        let ethereal_ir = EtherealIR::new(
            context.evmla().expect("Always exists").version.to_owned(),
            self.extra_metadata.unwrap_or_default(),
            Some(code_segment),
            blocks,
        )?;
        if let Some(debug_config) = context.debug_config() {
            let mut path = full_path.to_owned();
            if let solx_utils::CodeSegment::Runtime = code_segment {
                path.push_str(format!(".{code_segment}").as_str());
            }
            debug_config.dump_ethir(path.as_str(), ethereal_ir.to_string().as_str())?;
        }

        let mut entry = solx_codegen_evm::EntryFunction::new(ethereal_ir);
        entry.declare(context)?;
        entry.into_llvm(context)?;

        Ok(())
    }
}

impl std::fmt::Display for Assembly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(instructions) = self.code.as_ref() {
            for (index, instruction) in instructions.iter().enumerate() {
                match instruction.name {
                    InstructionName::Tag => writeln!(f, "{index:03} {instruction}")?,
                    _ => writeln!(f, "{index:03}     {instruction}")?,
                }
            }
        }

        writeln!(f)?;
        if let Some(data) = self.data.as_ref() {
            for data in data.values() {
                writeln!(f, "{data}")?;
            }
        }

        Ok(())
    }
}
