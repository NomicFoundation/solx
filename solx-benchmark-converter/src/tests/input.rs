//!
//! Tests for benchmark input path resolution.
//!

use crate::input::Input;

#[test]
fn a_single_file_is_an_input_not_a_directory() {
    let dir = tempfile::TempDir::new().expect("scratch directory");
    let file = dir.path().join("candidate.json");
    std::fs::write(file.as_path(), "{}").expect("file writing");
    assert_eq!(
        Input::resolve_paths(vec![file.clone()]).expect("resolution"),
        [file]
    );
}

#[test]
fn a_single_directory_expands_to_its_json_files() {
    let dir = tempfile::TempDir::new().expect("scratch directory");
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
