//!
//! Tests for benchmark I/O: input path resolution, benchmark filtering, and
//! spreadsheet output.
//!

use tempfile::TempDir;

use crate::benchmark::Benchmark;
use crate::input::Input;
use crate::output::xlsx::Xlsx;
use crate::output::xlsx::sheet::Sheet;

#[test]
fn a_single_file_is_an_input_not_a_directory() {
    let dir = TempDir::new().expect("scratch directory");
    let file = dir.path().join("candidate.json");
    std::fs::write(file.as_path(), "{}").expect("file writing");
    assert_eq!(
        Input::resolve_paths(vec![file.clone()]).expect("resolution"),
        [file]
    );
}

#[test]
fn a_single_directory_expands_to_its_json_files() {
    let dir = TempDir::new().expect("scratch directory");
    std::fs::create_dir_all(dir.path().join("nested")).expect("nested directory creation");
    for name in ["a.json", "nested/b.json", "ignored.txt"] {
        std::fs::write(dir.path().join(name), "{}").expect("file writing");
    }
    let mut resolved = Input::resolve_paths(vec![dir.path().to_path_buf()]).expect("resolution");
    resolved.sort();
    assert_eq!(
        resolved,
        [dir.path().join("a.json"), dir.path().join("nested/b.json")]
    );
}

#[test]
fn no_inputs_is_an_error() {
    assert!(Input::resolve_paths(Vec::new()).is_err());
}

#[test]
fn remove_blacklisted_drops_only_listed_rows() {
    let mut benchmark = Benchmark::default();
    benchmark.insert_test(
        "solady",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "fallback()",
    );
    benchmark.insert_test(
        "solady",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "someFunction()",
    );
    benchmark.insert_test(
        "other-project",
        "src/accounts/ERC6551Proxy.sol:ERC6551Proxy",
        "fallback()",
    );

    benchmark.remove_blacklisted();

    assert_eq!(benchmark.tests.len(), 2);
    for test in benchmark.tests.values() {
        let selector = &test.metadata.selector;
        assert!(
            selector.project != "solady"
                || selector
                    .input
                    .as_ref()
                    .and_then(|input| input.runtime_name())
                    != Some("fallback()")
        );
    }
}

#[test]
fn every_sheet_indexes_its_own_worksheet() {
    // `sheet as usize` indexes the worksheets `new` filled in `ALL` order:
    // a variant added mid-enum but appended to `ALL` would silently write
    // every later sheet's data to the wrong tab.
    let mut xlsx = Xlsx::new().expect("workbook creation");
    assert_eq!(xlsx.worksheets.len(), Sheet::ALL.len());
    for (index, sheet) in Sheet::ALL.into_iter().enumerate() {
        assert_eq!(sheet as usize, index, "{sheet:?}");
        let (name, headers) = sheet.spec();
        assert_eq!(xlsx.sheet(sheet).headers, headers, "{name}");
    }
}
