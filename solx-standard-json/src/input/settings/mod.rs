//!
//! The `solc --standard-json` input settings.
//!

pub mod debug;
pub mod metadata;
pub mod optimizer;
pub mod selection;

use std::collections::BTreeSet;

use self::debug::Debug;
use self::metadata::Metadata;
use self::optimizer::Optimizer;
use self::selection::Selection;

///
/// The `solc --standard-json` input settings.
///
#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// The optimizer settings.
    #[serde(default)]
    pub optimizer: Optimizer,

    /// The linker library addresses.
    #[serde(default, skip_serializing_if = "solx_utils::Libraries::is_empty")]
    pub libraries: solx_utils::Libraries,
    /// The sorted list of remappings.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub remappings: BTreeSet<String>,

    /// The target EVM version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<solx_utils::EVMVersion>,
    /// Whether to compile Solidity via IR.
    #[serde(
        default,
        rename = "viaIR",
        skip_serializing_if = "Settings::is_via_ir_default"
    )]
    pub via_ir: bool,

    /// The output selection filters.
    #[serde(default, skip_serializing_if = "Selection::is_empty")]
    pub output_selection: Selection,
    /// The metadata settings.
    #[serde(default)]
    pub metadata: Metadata,

    /// The debug settings (for solc).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug: Option<Debug>,

    /// The extra LLVM options.
    #[serde(default, skip_serializing)]
    pub llvm_options: Vec<String>,
}

impl Settings {
    ///
    /// A shortcut constructor for solx.
    ///
    /// Takes solx-specific parameters: LLVM optimizer mode, size fallback, metadata, llvm_options.
    ///
    pub fn new_for_solx(
        optimizer_mode: char,
        optimizer_size_fallback: bool,

        libraries: solx_utils::Libraries,
        remappings: BTreeSet<String>,

        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,

        mut output_selection: Selection,
        metadata: Metadata,
        llvm_options: Vec<String>,
    ) -> Self {
        output_selection.set_selector(via_ir.into());

        Self {
            optimizer: Optimizer::new(optimizer_mode, optimizer_size_fallback),

            libraries,
            remappings,

            evm_version,
            via_ir,

            output_selection,
            metadata,

            debug: None,
            llvm_options,
        }
    }

    ///
    /// A shortcut constructor for solc.
    ///
    /// Takes solc-specific parameters: optimizer enabled flag, debug settings.
    ///
    pub fn new_for_solc(
        optimizer_enabled: bool,

        libraries: solx_utils::Libraries,
        remappings: BTreeSet<String>,

        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,

        mut output_selection: Selection,
        debug: Option<Debug>,
    ) -> Self {
        output_selection.set_selector(via_ir.into());

        Self {
            optimizer: Optimizer::new_solc(optimizer_enabled),

            libraries,
            remappings,

            evm_version,
            via_ir,

            output_selection,
            metadata: Metadata::default(),

            debug,
            llvm_options: Vec::new(),
        }
    }

    ///
    /// A generic constructor with all options.
    ///
    pub fn new(
        optimizer: Optimizer,

        libraries: solx_utils::Libraries,
        remappings: BTreeSet<String>,

        evm_version: Option<solx_utils::EVMVersion>,
        via_ir: bool,

        mut output_selection: Selection,
        metadata: Metadata,

        debug: Option<Debug>,
        llvm_options: Vec<String>,
    ) -> Self {
        output_selection.set_selector(via_ir.into());

        Self {
            optimizer,

            libraries,
            remappings,

            evm_version,
            via_ir,

            output_selection,
            metadata,

            debug,
            llvm_options,
        }
    }

    ///
    /// Whether the via IR flag is the default.
    ///
    fn is_via_ir_default(via_ir: &bool) -> bool {
        !via_ir
    }
}
