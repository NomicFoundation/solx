//!
//! Typed Slang view of a test's Solidity sources.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::AbicoderVersion;
use slang_solidity_v2::ast::ExperimentalFeature;
use slang_solidity_v2::ast::Pragma;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationBuilder;
use slang_solidity_v2::compilation::CompilationBuilderConfig;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::diagnostics::kinds::compilation::MissingFile;
use slang_solidity_v2::diagnostics::kinds::compilation::UnresolvedImport;
use slang_solidity_v2::utils::EvmTarget;
use slang_solidity_v2::utils::LanguageVersion;

use solx_standard_json::output::contract::Contract;
use solx_utils::ContractName;

///
/// Typed Slang parse of a test's sources.
///
/// The tester parses sources itself because the AST JSON emitted by the compiler is
/// serialize-only upstream and cannot be read back into typed nodes. Parsing at the
/// latest language version and EVM target mirrors the Slang frontend pipeline.
///
pub struct SlangAst {
    /// The parsed compilation unit.
    unit: CompilationUnit,
    /// The file identifiers in test source order, which the path-sorted unit does not retain.
    files: Vec<FileId>,
}

impl SlangAst {
    ///
    /// Parses the test sources.
    ///
    pub fn parse(sources: &[(String, String)]) -> Self {
        let files: Vec<FileId> = sources
            .iter()
            .map(|(path, _source_code)| FileId::from(path.as_str()))
            .collect();
        let mut builder = CompilationBuilder::create(
            LanguageVersion::LATEST,
            EvmTarget::LATEST,
            TestSources(
                sources
                    .iter()
                    .map(|(path, source_code)| (FileId::from(path.as_str()), source_code.clone()))
                    .collect(),
            ),
        );
        for file_id in files.iter() {
            builder.add_file(file_id.clone());
        }
        Self {
            unit: builder.build(),
            files,
        }
    }

    ///
    /// Whether the sources pin `pragma abicoder v1` without declaring v2 anywhere.
    ///
    /// Slang always encodes with v2 semantics, so a v1-pinned test is not reproducible
    /// under the Slang frontend.
    ///
    pub fn is_abi_encoder_v1_pinned(&self) -> bool {
        let mut v1_pinned = false;
        let mut v2_declared = false;
        for file in self.unit.files() {
            for member in file.ast().members().iter() {
                let SourceUnitMember::PragmaDirective(directive) = member else {
                    continue;
                };
                match directive.pragma() {
                    Pragma::AbicoderPragma(pragma) => match pragma.version() {
                        AbicoderVersion::AbicoderV1Keyword(_) => v1_pinned = true,
                        AbicoderVersion::AbicoderV2Keyword(_) => v2_declared = true,
                    },
                    Pragma::ExperimentalPragma(pragma) => {
                        if matches!(
                            pragma.feature(),
                            ExperimentalFeature::ABIEncoderV2Keyword(_)
                        ) {
                            v2_declared = true;
                        }
                    }
                    Pragma::VersionPragma(_) => {}
                }
            }
        }
        v1_pinned && !v2_declared
    }

    ///
    /// The full path of the last compiled deployable object, a contract, interface, or
    /// library, in test source order: the deploy target of a test that does not name one.
    ///
    pub fn last_deployable(
        &self,
        contracts: &BTreeMap<String, BTreeMap<String, Contract>>,
    ) -> Option<String> {
        for file_id in self.files.iter().rev() {
            let Some(compiled) = contracts.get(file_id.as_str()) else {
                continue;
            };
            let members: Vec<SourceUnitMember> = self
                .unit
                .file(file_id)
                .expect("every test source is added to the compilation unit")
                .ast()
                .members()
                .iter()
                .collect();
            for member in members.iter().rev() {
                let name = match member {
                    SourceUnitMember::ContractDefinition(definition) => definition.name().name(),
                    SourceUnitMember::InterfaceDefinition(definition) => definition.name().name(),
                    SourceUnitMember::LibraryDefinition(definition) => definition.name().name(),
                    _ => continue,
                };
                if compiled.contains_key(name.as_str()) {
                    return Some(ContractName::full_path(file_id.as_str(), name.as_str()));
                }
            }
        }
        None
    }
}

///
/// Serves source contents to the compilation builder.
///
/// Import resolution is an exact lookup among the test's own files: the tester's queries
/// are syntactic, so an unresolved import costs nothing beyond an unused diagnostic.
///
struct TestSources(BTreeMap<FileId, String>);

impl CompilationBuilderConfig for TestSources {
    fn read_file(&mut self, file_id: &FileId) -> Result<String, MissingFile> {
        self.0.get(file_id).cloned().ok_or_else(|| MissingFile {
            reason: format!("file not found {file_id}"),
        })
    }

    fn resolve_import(
        &mut self,
        source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        let candidate = FileId::from(import_path);
        if self.0.contains_key(&candidate) {
            Ok(candidate)
        } else {
            Err(UnresolvedImport {
                reason: format!("failed to resolve import {import_path} in {source_file_id}"),
            })
        }
    }
}
