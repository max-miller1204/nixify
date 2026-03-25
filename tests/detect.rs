#[path = "detect/python.rs"]
mod python;
#[path = "detect/rust.rs"]
mod rust;

use nixify::detect::detect_project;
use nixify::types::{BuildSystem, Language};
use std::path::Path;

fn fixtures_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .leak()
}

#[test]
fn test_multi_rust_python() {
    let path = fixtures_path().join("multi-rust-python");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 2);
    assert!(info.is_multi_language());

    let rust_lang = info
        .languages
        .iter()
        .find(|l| l.language == Language::Rust)
        .expect("should detect Rust");
    assert_eq!(rust_lang.build_system, BuildSystem::Cargo);

    let python_lang = info
        .languages
        .iter()
        .find(|l| l.language == Language::Python)
        .expect("should detect Python");
    assert_eq!(python_lang.build_system, BuildSystem::Uv);

    assert_eq!(info.name, "multi-project");
    assert_eq!(
        info.description.as_deref(),
        Some("A multi-language project")
    );
}

#[test]
fn test_empty_directory_fails() {
    let dir = tempfile::tempdir().unwrap();
    let result = detect_project(dir.path());
    assert!(result.is_err());
}
