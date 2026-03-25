use nixify::generate::generate;
use nixify::types::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn rust_project() -> ProjectInfo {
    ProjectInfo {
        path: PathBuf::from("/tmp/my-rust-app"),
        name: "my-rust-app".to_string(),
        description: Some("A sample Rust application".to_string()),
        languages: vec![LanguageInfo {
            language: Language::Rust,
            build_system: BuildSystem::Cargo,
            kind: ProjectKind::Binary,
            metadata: HashMap::new(),
        }],
    }
}

fn python_poetry_project() -> ProjectInfo {
    ProjectInfo {
        path: PathBuf::from("/tmp/my-python-app"),
        name: "my-python-app".to_string(),
        description: Some("A sample Python application".to_string()),
        languages: vec![LanguageInfo {
            language: Language::Python,
            build_system: BuildSystem::Poetry,
            kind: ProjectKind::Binary,
            metadata: HashMap::new(),
        }],
    }
}

fn multi_project() -> ProjectInfo {
    ProjectInfo {
        path: PathBuf::from("/tmp/my-multi-app"),
        name: "my-multi-app".to_string(),
        description: Some("A multi-language project".to_string()),
        languages: vec![
            LanguageInfo {
                language: Language::Rust,
                build_system: BuildSystem::Cargo,
                kind: ProjectKind::Binary,
                metadata: HashMap::new(),
            },
            LanguageInfo {
                language: Language::Python,
                build_system: BuildSystem::Poetry,
                kind: ProjectKind::Library,
                metadata: HashMap::new(),
            },
        ],
    }
}

#[test]
fn test_rust_standalone_integration() {
    let config = FlakeConfig::default();
    let result = generate(&rust_project(), &config).unwrap();
    assert_eq!(result.style, FlakeStyle::Standalone);
    assert!(result.content.contains("crane"));
    assert!(result.content.contains("buildPackage"));
}

#[test]
fn test_rust_flake_parts_integration() {
    let config = FlakeConfig {
        style: FlakeStyle::FlakeParts,
        ..Default::default()
    };
    let result = generate(&rust_project(), &config).unwrap();
    assert_eq!(result.style, FlakeStyle::FlakeParts);
    assert!(result.content.contains("mkFlake"));
}

#[test]
fn test_python_poetry_integration() {
    let config = FlakeConfig::default();
    let result = generate(&python_poetry_project(), &config).unwrap();
    assert!(result.content.contains("poetry2nix"));
    assert!(result.content.contains("mkPoetryApplication"));
}

#[test]
fn test_multi_language_integration() {
    let config = FlakeConfig::default();
    let result = generate(&multi_project(), &config).unwrap();
    assert!(result.content.contains("crane"));
    assert!(result.content.contains("poetry2nix"));
    assert!(result.content.contains("python-lint"));
    assert!(result.content.contains("crate-clippy"));
}

#[test]
fn test_extra_config_injection() {
    let mut env_vars = HashMap::new();
    env_vars.insert("MY_VAR".to_string(), "hello".to_string());

    let config = FlakeConfig {
        style: FlakeStyle::Standalone,
        extra_packages: vec!["htop".to_string()],
        shell_hook: Some("echo hello".to_string()),
        env_vars,
        extra_inputs: HashMap::new(),
        extra_overlays: vec![],
    };

    let result = generate(&rust_project(), &config).unwrap();
    assert!(result.content.contains("htop"));
    assert!(result.content.contains("echo hello"));
    assert!(result.content.contains("MY_VAR"));
}
