use crate::errors::{NixifyError, Result};
use crate::types::{BuildSystem, Language, LanguageInfo, ProjectKind};
use std::collections::HashMap;
use std::path::Path;

use super::Detector;

pub struct RustDetector;

impl Detector for RustDetector {
    fn detect(&self, path: &Path) -> Result<Option<LanguageInfo>> {
        let cargo_path = path.join("Cargo.toml");
        if !cargo_path.exists() {
            return Ok(None);
        }

        let content = std::fs::read_to_string(&cargo_path).map_err(|e| NixifyError::IoError {
            path: cargo_path.display().to_string(),
            reason: e.to_string(),
        })?;

        let doc: toml::Table =
            content
                .parse()
                .map_err(|e: toml::de::Error| NixifyError::TomlParseError {
                    path: cargo_path.display().to_string(),
                    reason: e.to_string(),
                })?;

        let build_system = detect_build_system(&doc);
        let kind = detect_project_kind(path, &doc);
        let metadata = extract_metadata(&doc);

        Ok(Some(LanguageInfo {
            language: Language::Rust,
            build_system,
            kind,
            metadata,
        }))
    }
}

fn detect_build_system(doc: &toml::Table) -> BuildSystem {
    if let Some(workspace) = doc.get("workspace") {
        if let Some(members) = workspace.get("members").and_then(|m| m.as_array()) {
            let member_strings: Vec<String> = members
                .iter()
                .filter_map(|m| m.as_str().map(String::from))
                .collect();
            if !member_strings.is_empty() {
                return BuildSystem::CargoWorkspace {
                    members: member_strings,
                };
            }
        }
    }
    BuildSystem::Cargo
}

fn detect_project_kind(path: &Path, doc: &toml::Table) -> ProjectKind {
    // Check for explicit targets in Cargo.toml
    let has_lib_section = doc.get("lib").is_some();
    let has_bin_section = doc.get("bin").is_some();

    if has_lib_section && has_bin_section {
        return ProjectKind::Both;
    }

    // Check filesystem markers
    let has_main = path.join("src/main.rs").exists();
    let has_lib = path.join("src/lib.rs").exists();

    match (has_main || has_bin_section, has_lib || has_lib_section) {
        (true, true) => ProjectKind::Both,
        (true, false) => ProjectKind::Binary,
        (false, true) => ProjectKind::Library,
        // Workspace roots often have neither
        (false, false) => {
            if doc.get("workspace").is_some() {
                ProjectKind::Library
            } else {
                ProjectKind::Binary
            }
        }
    }
}

fn extract_metadata(doc: &toml::Table) -> HashMap<String, String> {
    let mut metadata = HashMap::new();

    if let Some(package) = doc.get("package") {
        if let Some(edition) = package.get("edition").and_then(|e| e.as_str()) {
            metadata.insert("edition".to_string(), edition.to_string());
        }
        if let Some(version) = package.get("version").and_then(|v| v.as_str()) {
            metadata.insert("version".to_string(), version.to_string());
        }
        if let Some(name) = package.get("name").and_then(|n| n.as_str()) {
            metadata.insert("name".to_string(), name.to_string());
        }
    }

    metadata
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cargo_toml() {
        let dir = tempfile::tempdir().unwrap();
        let detector = RustDetector;
        let result = detector.detect(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_simple_binary() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[package]
name = "hello"
version = "0.1.0"
edition = "2021"
"#,
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        let detector = RustDetector;
        let info = detector.detect(dir.path()).unwrap().unwrap();
        assert_eq!(info.language, Language::Rust);
        assert_eq!(info.build_system, BuildSystem::Cargo);
        assert_eq!(info.kind, ProjectKind::Binary);
        assert_eq!(info.metadata.get("edition").unwrap(), "2021");
    }

    #[test]
    fn test_workspace() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("Cargo.toml"),
            r#"
[workspace]
members = ["crate-a", "crate-b"]
"#,
        )
        .unwrap();

        let detector = RustDetector;
        let info = detector.detect(dir.path()).unwrap().unwrap();
        assert_eq!(
            info.build_system,
            BuildSystem::CargoWorkspace {
                members: vec!["crate-a".to_string(), "crate-b".to_string()]
            }
        );
    }
}
