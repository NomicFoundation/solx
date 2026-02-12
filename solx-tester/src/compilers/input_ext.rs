//!
//! Input construction helpers for the test harness.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;

///
/// Creates the output selection required for testing.
///
/// Selects AST, bytecode, deployedBytecode, methodIdentifiers,
/// and either Yul or EVMLegacyAssembly depending on via_ir.
///
pub fn selection_required_for_testing(via_ir: bool) -> solx_standard_json::InputSelection {
    let mut selectors = BTreeSet::new();
    selectors.insert(solx_standard_json::InputSelector::AST);
    selectors.insert(solx_standard_json::InputSelector::Bytecode);
    selectors.insert(solx_standard_json::InputSelector::RuntimeBytecode);
    selectors.insert(solx_standard_json::InputSelector::MethodIdentifiers);
    selectors.insert(if via_ir {
        solx_standard_json::InputSelector::Yul
    } else {
        solx_standard_json::InputSelector::EVMLegacyAssembly
    });
    solx_standard_json::InputSelection::new(selectors)
}

///
/// Creates an Input for solc toolchain compilation.
///
pub fn new_input_for_solc(
    language: solx_standard_json::InputLanguage,
    sources: BTreeMap<String, String>,
    libraries: solx_utils::Libraries,
    remappings: Option<BTreeSet<String>>,
    evm_version: Option<solx_utils::EVMVersion>,
    via_ir: bool,
    mut output_selection: solx_standard_json::InputSelection,
    optimizer_enabled: bool,
    debug: Option<solx_standard_json::InputDebug>,
) -> solx_standard_json::Input {
    let sources = sources
        .into_iter()
        .map(|(path, content)| {
            (
                path,
                solx_standard_json::InputSource {
                    content: Some(content),
                    urls: None,
                },
            )
        })
        .collect();

    output_selection.set_selector(via_ir.into());

    solx_standard_json::Input {
        language,
        sources,
        settings: solx_standard_json::InputSettings {
            optimizer: solx_standard_json::InputOptimizer {
                enabled: Some(optimizer_enabled),
                mode: None,
                size_fallback: None,
            },
            libraries,
            remappings: remappings.unwrap_or_default(),
            evm_version,
            via_ir,
            output_selection,
            metadata: solx_standard_json::InputMetadata::default(),
            debug,
            llvm_options: Vec::new(),
        },
    }
}

///
/// Creates an Input from LLVM IR source code.
///
pub fn new_input_from_llvm_ir_sources(
    sources: BTreeMap<String, solx_standard_json::InputSource>,
    libraries: solx_utils::Libraries,
    optimizer: solx_standard_json::InputOptimizer,
    output_selection: &solx_standard_json::InputSelection,
    metadata: solx_standard_json::InputMetadata,
    llvm_options: Vec<String>,
) -> solx_standard_json::Input {
    solx_standard_json::Input {
        language: solx_standard_json::InputLanguage::LLVMIR,
        sources,
        settings: solx_standard_json::InputSettings::new(
            optimizer,
            libraries,
            BTreeSet::new(),
            None,
            false,
            output_selection.to_owned(),
            metadata,
            None,
            llvm_options,
        ),
    }
}
