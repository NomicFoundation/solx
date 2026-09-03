//!
//! Typed Slang view of a test's Solidity sources.
//!

use std::collections::BTreeMap;

use slang_solidity_v2::ast::AbicoderVersion;
use slang_solidity_v2::ast::Pragma;
use slang_solidity_v2::ast::SourceUnitMember;
use slang_solidity_v2::compilation::CompilationUnit;
use slang_solidity_v2::compilation::Configuration;
use slang_solidity_v2::compilation::FileId;
use slang_solidity_v2::compilation::ImportResolver;
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
        let unit = CompilationUnit::create(Configuration {
            language_version: LanguageVersion::LATEST,
            evm_target: EvmTarget::LATEST,
            sources: sources
                .iter()
                .map(|(path, source_code)| (FileId::from(path.as_str()), source_code.as_str())),
            resolver: VerbatimImportResolver,
        });
        Self { unit, files }
    }

    ///
    /// Whether any source pins `pragma abicoder v1`.
    ///
    /// Slang always encodes with v2 semantics, so a v1-pinned test is not reproducible
    /// under the Slang frontend.
    ///
    pub fn is_abi_encoder_v1_pinned(&self) -> bool {
        for file in self.unit.files() {
            for member in file.ast().members().iter() {
                let SourceUnitMember::PragmaDirective(directive) = member else {
                    continue;
                };
                if let Pragma::AbicoderPragma(pragma) = directive.pragma()
                    && matches!(pragma.version(), AbicoderVersion::V1)
                {
                    return true;
                }
            }
        }
        false
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
                let identifier = match member {
                    SourceUnitMember::ContractDefinition(definition) => definition.name(),
                    SourceUnitMember::InterfaceDefinition(definition) => definition.name(),
                    SourceUnitMember::LibraryDefinition(definition) => definition.name(),
                    _ => continue,
                };
                let name = identifier.name();
                if compiled.contains_key(name) {
                    return Some(ContractName::full_path(file_id.as_str(), name));
                }
            }
        }
        None
    }
}

///
/// Resolves an import path to the test file of that exact name.
///
/// Slang reports an import of a file outside the test's own sources; the tester's queries
/// are syntactic, so it costs nothing beyond an unused diagnostic.
///
struct VerbatimImportResolver;

impl ImportResolver for VerbatimImportResolver {
    fn resolve_import(
        &mut self,
        _source_file_id: &FileId,
        import_path: &str,
    ) -> Result<FileId, UnresolvedImport> {
        Ok(import_path.into())
    }
}
