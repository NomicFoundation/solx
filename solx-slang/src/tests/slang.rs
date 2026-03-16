//! Tests for [`crate::SlangFrontend`].

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
