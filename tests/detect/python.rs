use nixify::detect::detect_project;
use nixify::types::{BuildSystem, Language, ProjectKind};
use std::path::Path;

fn fixtures_path() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .leak()
}

#[test]
fn test_python_poetry() {
    let path = fixtures_path().join("python-poetry");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 1);
    let lang = &info.languages[0];
    assert_eq!(lang.language, Language::Python);
    assert_eq!(lang.build_system, BuildSystem::Poetry);
    assert_eq!(lang.kind, ProjectKind::Library);

    assert_eq!(info.name, "my-poetry-app");
    assert_eq!(info.description.as_deref(), Some("A Poetry project"));
}

#[test]
fn test_python_uv() {
    let path = fixtures_path().join("python-uv");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 1);
    let lang = &info.languages[0];
    assert_eq!(lang.language, Language::Python);
    assert_eq!(lang.build_system, BuildSystem::Uv);
    assert_eq!(lang.kind, ProjectKind::Library);
    assert_eq!(lang.metadata.get("requires_python").unwrap(), ">=3.11");

    assert_eq!(info.name, "my-uv-app");
}

#[test]
fn test_python_pip_setuptools() {
    let path = fixtures_path().join("python-pip");
    let info = detect_project(&path).unwrap();

    assert_eq!(info.languages.len(), 1);
    let lang = &info.languages[0];
    assert_eq!(lang.language, Language::Python);
    // setup.py present with requirements.txt -> SetupTools
    assert_eq!(lang.build_system, BuildSystem::SetupTools);
}
