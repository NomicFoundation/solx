//!
//! Slang Solidity frontend implementation.
//!

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::PathBuf;

use slang_solidity::compilation::CompilationBuilder;
use slang_solidity::compilation::CompilationUnit;
use slang_solidity::utils::LanguageFacts;

use crate::SlangCompilationConfig;

///
/// The Slang frontend implementation.
///
#[derive(Debug)]
pub struct SlangFrontend {
    /// The Slang compiler version.
    version: solx_standard_json::Version,
}

impl Default for SlangFrontend {
    fn default() -> Self {
        let default = LanguageFacts::LATEST_VERSION;

        Self {
            version: solx_standard_json::Version::new(default.to_string(), default),
        }
    }
}

impl SlangFrontend {
    /// Builds a Slang compilation unit from the given source files.
    ///
    /// Uses the `CompilationBuilder` to parse all sources and resolve imports.
    ///
    /// # Errors
    ///
    /// Returns an error if the compilation builder fails to initialize or
    /// if import resolution fails.
    pub fn compile(
        &self,
        sources: BTreeMap<String, String>,
    ) -> anyhow::Result<CompilationUnit> {
        let keys: Vec<String> = sources.keys().cloned().collect();
        let configuration = SlangCompilationConfig::new(sources);
        let mut builder =
            CompilationBuilder::create(self.version.default.clone(), configuration)
                .map_err(|error| anyhow::anyhow!("slang compilation builder: {error}"))?;

        for path in &keys {
            builder.add_file(path)?;
        }

        Ok(builder.build())
    }

    /// Compiles Solidity source files from filesystem paths.
    ///
    /// Resolves sources, builds and returns a compilation unit with all files
    /// parsed and imports resolved.
    ///
    /// # Errors
    ///
    /// Returns an error if source resolution or compilation fails.
    pub fn compile_from_paths(
        &self,
        input_files: &[PathBuf],
        libraries: &[String],
        remappings: BTreeSet<String>,
    ) -> anyhow::Result<CompilationUnit> {
        let mut input = solx_standard_json::Input::try_from_solidity_paths(
            input_files,
            libraries,
            remappings,
            solx_standard_json::InputOptimizer::default(),
            None,
            false,
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        )?;

        input.resolve_sources()?;

        let mut sources = BTreeMap::new();
        for (path, source) in &input.sources {
            let content = source.content().ok_or_else(|| {
                anyhow::anyhow!("source content unavailable for '{path}'")
            })?;
            sources.insert(path.clone(), content.to_owned());
        }

        self.compile(sources)
    }
}

impl solx_core::Frontend for SlangFrontend {
    fn name(&self) -> &str {
        "Slang"
    }

    fn standard_json(
        &self,
        input_json: &mut solx_standard_json::Input,
        _use_import_callback: bool,
        _base_path: Option<&str>,
        _include_paths: &[String],
        _allow_paths: Option<String>,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut output = solx_standard_json::Output::new(&input_json.sources);

        if input_json.language != solx_standard_json::InputLanguage::Solidity {
            output
                .errors
                .push(solx_standard_json::OutputError::new_error(
                    "Slang frontend only supports Solidity sources.",
                ));
            return Ok(output);
        }

        if let Err(error) = input_json.resolve_sources() {
            output
                .errors
                .push(solx_standard_json::OutputError::new_error(error));
            return Ok(output);
        }

        let sources = crate::collect_sources(&input_json.sources, &mut output);
        let unit = self.compile(sources)?;
        crate::report_compilation_results(&unit, &input_json.sources, &mut output)?;

        Ok(output)
    }

    fn validate_yul_paths(
        &self,
        paths: &[PathBuf],
        libraries: solx_utils::Libraries,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut solc_input = solx_standard_json::Input::from_yul_paths(
            paths,
            libraries,
            solx_standard_json::InputOptimizer::default(),
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        );

        self.validate_yul_standard_json(&mut solc_input)
    }

    fn validate_yul_standard_json(
        &self,
        solc_input: &mut solx_standard_json::Input,
    ) -> anyhow::Result<solx_standard_json::Output> {
        let mut output = solx_standard_json::Output::new(&solc_input.sources);
        output
            .errors
            .push(solx_standard_json::OutputError::new_error(
                "Yul validation is not supported by the Slang frontend.",
            ));
        Ok(output)
    }

    fn version(&self) -> &solx_standard_json::Version {
        &self.version
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;

    use slang_solidity::utils::LanguageFacts;

    use solx_core::Frontend;

    use crate::SlangFrontend;

    fn build_solidity_input(path: &str, source: &str) -> solx_standard_json::Input {
        let mut sources = BTreeMap::new();
        sources.insert(
            path.to_owned(),
            solx_standard_json::InputSource {
                content: Some(source.to_owned()),
                urls: None,
            },
        );

        solx_standard_json::Input::try_from_solidity_sources(
            sources,
            solx_utils::Libraries::default(),
            BTreeSet::new(),
            solx_standard_json::InputOptimizer::default(),
            None,
            false,
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        )
        .expect("Always valid")
    }

    #[test]
    fn parses_valid_source_without_errors() {
        let frontend = SlangFrontend::default();
        let mut input = build_solidity_input("Test.sol", "contract Test {}");

        let output = frontend
            .standard_json(&mut input, false, None, &[], None)
            .expect("Always valid");

        assert!(output.errors.is_empty());
    }

    #[test]
    fn returns_parse_error_with_source_location() {
        let frontend = SlangFrontend::default();
        let mut input = build_solidity_input("Test.sol", "contract Test {");

        let output = frontend
            .standard_json(&mut input, false, None, &[], None)
            .expect("Always valid");

        assert!(!output.errors.is_empty());

        let error = output.errors.first().expect("Always exists");
        assert_eq!(error.severity, "error");
        assert!(error.message.contains("Expected"));

        let source_location = error.source_location.as_ref().expect("Always exists");
        assert_eq!(source_location.file, "Test.sol");
        assert!(source_location.start.is_some());
        assert!(source_location.end.is_some());
    }

    #[test]
    fn uses_latest_language_version() {
        let frontend = SlangFrontend::default();

        assert_eq!(frontend.version().default, LanguageFacts::LATEST_VERSION);
        assert_eq!(
            frontend.version().long,
            LanguageFacts::LATEST_VERSION.to_string()
        );
    }

    #[test]
    fn yul_validation_is_unsupported() {
        let frontend = SlangFrontend::default();
        let mut sources = BTreeMap::new();
        sources.insert(
            "Test.yul".to_owned(),
            solx_standard_json::InputSource {
                content: Some("{ let a := 1 }".to_owned()),
                urls: None,
            },
        );
        let mut input = solx_standard_json::Input::from_yul_sources(
            sources,
            solx_utils::Libraries::default(),
            solx_standard_json::InputOptimizer::default(),
            &solx_standard_json::InputSelection::default(),
            solx_standard_json::InputMetadata::default(),
            vec![],
        );

        let output = frontend
            .validate_yul_standard_json(&mut input)
            .expect("Always valid");

        assert_eq!(output.errors.len(), 1);
        assert!(
            output.errors[0]
                .message
                .contains("Yul validation is not supported")
        );
    }
}
