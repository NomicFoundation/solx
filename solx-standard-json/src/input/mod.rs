//!
//! The `solc --standard-json` input.
//!

pub mod language;
pub mod settings;
pub mod source;

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use crate::input::settings::metadata::Metadata as InputSettingsMetadata;
use crate::input::settings::optimizer::Optimizer as InputSettingsOptimizer;
use crate::input::settings::selection::Selection as InputSettingsSelection;

use self::language::Language;
use self::settings::Settings;
use self::source::Source;

///
/// The `solc --standard-json` input.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Input {
    /// The input language.
    pub language: Language,
    /// The input source code files hashmap.
    pub sources: BTreeMap<String, Source>,
    /// The compiler settings.
    pub settings: Settings,
}

impl Input {
    ///
    /// A shortcut constructor.
    ///
    /// If the `path` is `None`, the input is read from the stdin.
    ///
    pub fn try_from(path: Option<&Path>) -> anyhow::Result<Self> {
        let input_json = match path {
            Some(path) if path.to_string_lossy() == Source::STDIN_INPUT_IDENTIFIER => {
                std::io::read_to_string(std::io::stdin())
                    .map_err(|error| anyhow::anyhow!("Standard JSON reading from stdin: {error}"))
            }
            Some(path) => std::fs::read_to_string(path)
                .map_err(|error| anyhow::anyhow!("Standard JSON file {path:?} reading: {error}")),
            None => std::io::read_to_string(std::io::stdin())
                .map_err(|error| anyhow::anyhow!("Standard JSON reading from stdin: {error}")),
        }?;
        if let Ok(output_path) = std::env::var(crate::STANDARD_JSON_DEBUG_ENV) {
            std::fs::write(output_path.as_str(), input_json.as_str()).map_err(|error| {
                anyhow::anyhow!("Standard JSON input debug file `{output_path}` writing: {error}")
            })?;
        }
        solx_utils::deserialize_from_str::<Self>(input_json.as_str())
            .map_err(|error| anyhow::anyhow!("Standard JSON parsing: {error}"))
    }

    ///
    /// A shortcut constructor from paths to Solidity source files.
    ///
    pub fn try_from_solidity_paths(
        paths: &[PathBuf],
        libraries: &[String],
        remappings: BTreeSet<String>,
        optimizer: InputSettingsOptimizer,
        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> anyhow::Result<Self> {
        let mut paths: BTreeSet<PathBuf> = paths.iter().cloned().collect();
        let libraries = solx_utils::Libraries::try_from(libraries)?;
        for library_file in libraries.as_inner().keys() {
            paths.insert(PathBuf::from(library_file));
        }

        let sources = paths
            .into_par_iter()
            .map(|path| {
                let source = Source::try_from_path(path.as_path())?;
                let path = if path.to_string_lossy() == Source::STDIN_INPUT_IDENTIFIER {
                    Source::STDIN_OUTPUT_IDENTIFIER.to_owned()
                } else {
                    path.to_string_lossy().to_string()
                };
                Ok((path, source))
            })
            .collect::<anyhow::Result<BTreeMap<String, Source>>>()?;

        Self::try_from_solidity_sources(
            sources,
            libraries,
            remappings,
            optimizer,
            evm_version,
            via_ir,
            output_selection,
            metadata,
            llvm_options,
        )
    }

    ///
    /// A shortcut constructor from Solidity source code.
    ///
    pub fn try_from_solidity_sources(
        sources: BTreeMap<String, Source>,
        libraries: solx_utils::Libraries,
        remappings: BTreeSet<String>,
        optimizer: InputSettingsOptimizer,
        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            language: Language::Solidity,
            sources,
            settings: Settings::new(
                optimizer,
                libraries,
                remappings,
                evm_version,
                via_ir,
                output_selection.to_owned(),
                metadata,
                llvm_options,
            ),
        })
    }

    ///
    /// A shortcut constructor from paths to Yul source files.
    ///
    pub fn from_yul_paths(
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
        optimizer: InputSettingsOptimizer,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> Self {
        let sources = paths
            .iter()
            .map(|path| {
                (
                    path.to_string_lossy().to_string(),
                    Source::from(path.as_path()),
                )
            })
            .collect();

        Self::from_yul_sources(
            sources,
            libraries,
            optimizer,
            output_selection,
            metadata,
            llvm_options,
        )
    }

    ///
    /// A shortcut constructor from Yul source code.
    ///
    pub fn from_yul_sources(
        sources: BTreeMap<String, Source>,
        libraries: solx_utils::Libraries,
        optimizer: InputSettingsOptimizer,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> Self {
        Self {
            language: Language::Yul,
            sources,
            settings: Settings::new(
                optimizer,
                libraries,
                BTreeSet::new(),
                None,
                false,
                output_selection.to_owned(),
                metadata,
                llvm_options,
            ),
        }
    }

    ///
    /// A shortcut constructor from paths to LLVM IR source files.
    ///
    pub fn from_llvm_ir_paths(
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
        optimizer: InputSettingsOptimizer,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> Self {
        let sources = paths
            .iter()
            .map(|path| {
                (
                    path.to_string_lossy().to_string(),
                    Source::from(path.as_path()),
                )
            })
            .collect();

        Self::from_llvm_ir_sources(
            sources,
            libraries,
            optimizer,
            output_selection,
            metadata,
            llvm_options,
        )
    }

    ///
    /// A shortcut constructor from LLVM IR source code.
    ///
    pub fn from_llvm_ir_sources(
        sources: BTreeMap<String, Source>,
        libraries: solx_utils::Libraries,
        optimizer: InputSettingsOptimizer,
        output_selection: &InputSettingsSelection,
        metadata: InputSettingsMetadata,
        llvm_options: Vec<String>,
    ) -> Self {
        Self {
            language: Language::LLVMIR,
            sources,
            settings: Settings::new(
                optimizer,
                libraries,
                BTreeSet::new(),
                None,
                false,
                output_selection.to_owned(),
                metadata,
                llvm_options,
            ),
        }
    }

    ///
    /// Resolves the sources by applying library links and remappings.
    ///
    pub fn resolve_sources(&mut self) -> anyhow::Result<()> {
        for source in self.sources.values_mut() {
            source.try_resolve()?
        }
        Ok(())
    }
}
