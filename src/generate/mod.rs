mod checks;
mod devshell;
mod flake_parts;
mod inputs;
mod packages;
mod standalone;

use crate::errors::Result;
use crate::types::{FlakeConfig, FlakeStyle, GeneratedFlake, ProjectInfo};

/// Generate a `flake.nix` from the detected project info and configuration.
///
/// This is the main public API for the flake generator. It dispatches to either
/// the standalone or flake-parts generator based on `config.style`.
pub fn generate(info: &ProjectInfo, config: &FlakeConfig) -> Result<GeneratedFlake> {
    match config.style {
        FlakeStyle::Standalone => standalone::generate_standalone(info, config),
        FlakeStyle::FlakeParts => flake_parts::generate_flake_parts(info, config),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
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

    fn python_pip_project() -> ProjectInfo {
        ProjectInfo {
            path: PathBuf::from("/tmp/my-pip-app"),
            name: "my-pip-app".to_string(),
            description: None,
            languages: vec![LanguageInfo {
                language: Language::Python,
                build_system: BuildSystem::Pip,
                kind: ProjectKind::Binary,
                metadata: HashMap::new(),
            }],
        }
    }

    fn python_requirements_only_project() -> ProjectInfo {
        let mut metadata = HashMap::new();
        metadata.insert("has_packaging_metadata".to_string(), "false".to_string());

        ProjectInfo {
            path: PathBuf::from("/tmp/my-python-app"),
            name: "my-python-app".to_string(),
            description: Some("A sample Python application".to_string()),
            languages: vec![LanguageInfo {
                language: Language::Python,
                build_system: BuildSystem::Pip,
                kind: ProjectKind::Library,
                metadata,
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

    fn default_config() -> FlakeConfig {
        FlakeConfig::default()
    }

    fn flake_parts_config() -> FlakeConfig {
        FlakeConfig {
            style: FlakeStyle::FlakeParts,
            ..Default::default()
        }
    }

    fn config_with_extras() -> FlakeConfig {
        let mut env_vars = HashMap::new();
        env_vars.insert("RUST_LOG".to_string(), "debug".to_string());

        let mut extra_inputs = HashMap::new();
        extra_inputs.insert(
            "rust-overlay".to_string(),
            "github:oxalica/rust-overlay".to_string(),
        );

        FlakeConfig {
            style: FlakeStyle::Standalone,
            extra_packages: vec!["nodePackages.prettier".to_string(), "jq".to_string()],
            shell_hook: Some("echo 'Welcome to the dev shell!'".to_string()),
            env_vars,
            extra_inputs,
            extra_overlays: vec![],
        }
    }

    // === Rust standalone tests ===

    #[test]
    fn test_rust_standalone_generates() {
        let result = generate(&rust_project(), &default_config()).unwrap();
        assert_eq!(result.style, FlakeStyle::Standalone);
        assert!(result.content.contains("crane"));
        assert!(result.content.contains("flake-utils"));
        assert!(result.content.contains("A sample Rust application"));
        assert!(result.inputs_used.contains(&"crane".to_string()));
        assert!(result.checks.contains(&"crate-clippy".to_string()));
        insta::assert_snapshot!("rust_standalone", result.content);
    }

    #[test]
    fn test_rust_standalone_with_extras() {
        let result = generate(&rust_project(), &config_with_extras()).unwrap();
        assert!(result.content.contains("RUST_LOG"));
        assert!(result.content.contains("nodePackages.prettier"));
        assert!(result.content.contains("Welcome to the dev shell!"));
        assert!(result.content.contains("rust-overlay"));
        insta::assert_snapshot!("rust_standalone_extras", result.content);
    }

    // === Rust flake-parts tests ===

    #[test]
    fn test_rust_flake_parts_generates() {
        let result = generate(&rust_project(), &flake_parts_config()).unwrap();
        assert_eq!(result.style, FlakeStyle::FlakeParts);
        assert!(result.content.contains("flake-parts"));
        assert!(result.content.contains("mkFlake"));
        assert!(result.content.contains("perSystem"));
        insta::assert_snapshot!("rust_flake_parts", result.content);
    }

    // === Python standalone tests ===

    #[test]
    fn test_python_poetry_standalone() {
        let result = generate(&python_poetry_project(), &default_config()).unwrap();
        assert!(result.content.contains("poetry2nix"));
        assert!(result.content.contains("mkPoetryApplication"));
        assert!(result.inputs_used.contains(&"poetry2nix".to_string()));
        insta::assert_snapshot!("python_poetry_standalone", result.content);
    }

    #[test]
    fn test_python_pip_standalone() {
        let result = generate(&python_pip_project(), &default_config()).unwrap();
        assert!(result.content.contains("buildPythonApplication"));
        assert!(!result.content.contains("poetry2nix"));
        insta::assert_snapshot!("python_pip_standalone", result.content);
    }

    #[test]
    fn test_python_requirements_only_standalone() {
        let result = generate(&python_requirements_only_project(), &default_config()).unwrap();
        assert!(!result.content.contains("buildPythonApplication"));
        assert!(!result.content.contains("pythonApp ="));
        assert!(result.content.contains("packages.default = null;"));
        assert!(result.content.contains("inputsFrom = [  ];"));
        insta::assert_snapshot!("python_requirements_only_standalone", result.content);
    }

    // === Python flake-parts tests ===

    #[test]
    fn test_python_poetry_flake_parts() {
        let result = generate(&python_poetry_project(), &flake_parts_config()).unwrap();
        assert!(result.content.contains("flake-parts"));
        assert!(result.content.contains("poetry2nix"));
        insta::assert_snapshot!("python_poetry_flake_parts", result.content);
    }

    // === Multi-language tests ===

    #[test]
    fn test_multi_standalone() {
        let result = generate(&multi_project(), &default_config()).unwrap();
        assert!(result.content.contains("crane"));
        assert!(result.content.contains("poetry2nix"));
        assert!(result.content.contains("crate-clippy"));
        assert!(result.content.contains("python-lint"));
        insta::assert_snapshot!("multi_standalone", result.content);
    }

    #[test]
    fn test_multi_flake_parts() {
        let result = generate(&multi_project(), &flake_parts_config()).unwrap();
        assert!(result.content.contains("flake-parts"));
        assert!(result.content.contains("crane"));
        assert!(result.content.contains("poetry2nix"));
        insta::assert_snapshot!("multi_flake_parts", result.content);
    }

    // === Input collection tests ===

    #[test]
    fn test_inputs_rust_standalone() {
        let inputs = super::inputs::collect_inputs(&rust_project(), &default_config());
        assert!(inputs.contains(&"nixpkgs".to_string()));
        assert!(inputs.contains(&"crane".to_string()));
        assert!(inputs.contains(&"flake-utils".to_string()));
    }

    #[test]
    fn test_inputs_python_poetry() {
        let inputs = super::inputs::collect_inputs(&python_poetry_project(), &default_config());
        assert!(inputs.contains(&"poetry2nix".to_string()));
    }

    #[test]
    fn test_inputs_extra() {
        let inputs = super::inputs::collect_inputs(&rust_project(), &config_with_extras());
        assert!(inputs.contains(&"rust-overlay".to_string()));
    }

    // === Check collection tests ===

    #[test]
    fn test_checks_rust() {
        let checks = super::checks::collect_checks(&rust_project());
        assert!(checks.contains(&"crate-clippy".to_string()));
        assert!(checks.contains(&"crate-fmt".to_string()));
        assert!(checks.contains(&"crate-test".to_string()));
    }

    #[test]
    fn test_checks_python() {
        let checks = super::checks::collect_checks(&python_poetry_project());
        assert!(checks.contains(&"python-lint".to_string()));
        assert!(checks.contains(&"python-typecheck".to_string()));
    }

    #[test]
    fn test_checks_multi() {
        let checks = super::checks::collect_checks(&multi_project());
        assert!(checks.contains(&"crate-clippy".to_string()));
        assert!(checks.contains(&"python-lint".to_string()));
    }

    // === Devshell package tests ===

    #[test]
    fn test_devshell_packages_rust() {
        let pkgs = super::devshell::collect_devshell_packages(&rust_project(), &default_config());
        assert!(pkgs.contains(&"rust-analyzer".to_string()));
        assert!(pkgs.contains(&"clippy".to_string()));
    }

    #[test]
    fn test_devshell_packages_python_poetry() {
        let pkgs =
            super::devshell::collect_devshell_packages(&python_poetry_project(), &default_config());
        assert!(pkgs.contains(&"python3".to_string()));
        assert!(pkgs.contains(&"poetry".to_string()));
    }

    #[test]
    fn test_devshell_extra_packages() {
        let config = config_with_extras();
        let pkgs = super::devshell::collect_devshell_packages(&rust_project(), &config);
        assert!(pkgs.contains(&"nodePackages.prettier".to_string()));
        assert!(pkgs.contains(&"jq".to_string()));
    }

    // === GeneratedFlake field tests ===

    #[test]
    fn test_generated_flake_fields() {
        let result = generate(&rust_project(), &default_config()).unwrap();
        assert!(!result.inputs_used.is_empty());
        assert!(!result.devshell_packages.is_empty());
        assert!(!result.checks.is_empty());
    }

    #[test]
    fn test_no_description_uses_default() {
        let mut project = rust_project();
        project.description = None;
        let result = generate(&project, &default_config()).unwrap();
        assert!(result.content.contains("Nix flake for my-rust-app"));
    }
}
