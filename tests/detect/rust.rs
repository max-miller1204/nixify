use nixify::detect::detect_project;
use nixify::types::{BuildSystem, Language, ProjectKind};
use std::path::Path;

fn fixtures_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .leak()
}

#[test]
fn test_rust_simple_binary() {
    let path = fixtures_path().join("rust-simple");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 1);
    let lang = &info.languages[0];
    assert_eq!(lang.language, Language::Rust);
    assert_eq!(lang.build_system, BuildSystem::Cargo);
    assert_eq!(lang.kind, ProjectKind::Binary);
    assert_eq!(lang.metadata.get("edition").unwrap(), "2021");

    assert_eq!(info.name, "hello-rust");
    assert_eq!(info.description.as_deref(), Some("A simple Rust binary"));
}

#[test]
fn test_rust_workspace() {
    let path = fixtures_path().join("rust-workspace");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 1);
    let lang = &info.languages[0];
    assert_eq!(lang.language, Language::Rust);
    assert_eq!(
        lang.build_system,
        BuildSystem::CargoWorkspace {
            members: vec!["app".to_string(), "lib-core".to_string()]
        }
    );
    // Workspace root with no src/ defaults to Library
    assert_eq!(lang.kind, ProjectKind::Library);
}
