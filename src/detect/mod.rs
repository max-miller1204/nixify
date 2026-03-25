mod multi;
mod python;
mod rust;

use crate::errors::{NixifyError, Result};
use crate::types::{LanguageInfo, ProjectInfo};
use std::path::Path;

/// Trait for language-specific detectors.
pub trait Detector {
    /// Returns `Some(LanguageInfo)` if this detector recognizes the project at `path`.
    fn detect(&self, path: &Path) -> Result<Option<LanguageInfo>>;
}

/// Run all detectors against a project directory and return a `ProjectInfo`.
pub fn detect_project(path: &Path) -> Result<ProjectInfo> {
    let canonical = path.canonicalize().map_err(|e| NixifyError::IoError {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;

    let languages = multi::detect_all(&canonical)?;

    if languages.is_empty() {
        return Err(NixifyError::NoLanguageDetected {
            path: canonical.display().to_string(),
        });
    }

    let (name, description) = extract_project_metadata(&canonical, &languages);

    Ok(ProjectInfo {
        path: canonical,
        languages,
        name,
        description,
    })
}

/// Derive a project name and optional description from detected languages.
fn extract_project_metadata(path: &Path, languages: &[LanguageInfo]) -> (String, Option<String>) {
    // Try Cargo.toml first for name/description
    for lang in languages {
        if lang.language == crate::types::Language::Rust {
            if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
                if let Ok(doc) = content.parse::<toml::Table>() {
                    let name = doc
                        .get("package")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from);
                    let desc = doc
                        .get("package")
                        .and_then(|p| p.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    if let Some(n) = name {
                        return (n, desc);
                    }
                }
            }
        }
    }

    // Try pyproject.toml
    for lang in languages {
        if lang.language == crate::types::Language::Python {
            if let Ok(content) = std::fs::read_to_string(path.join("pyproject.toml")) {
                if let Ok(doc) = content.parse::<toml::Table>() {
                    let name = doc
                        .get("project")
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from);
                    let desc = doc
                        .get("project")
                        .and_then(|p| p.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    if let Some(n) = name {
                        return (n, desc);
                    }

                    // Poetry uses [tool.poetry] section
                    let name = doc
                        .get("tool")
                        .and_then(|t| t.get("poetry"))
                        .and_then(|p| p.get("name"))
                        .and_then(|n| n.as_str())
                        .map(String::from);
                    let desc = doc
                        .get("tool")
                        .and_then(|t| t.get("poetry"))
                        .and_then(|p| p.get("description"))
                        .and_then(|d| d.as_str())
                        .map(String::from);
                    if let Some(n) = name {
                        return (n, desc);
                    }
                }
            }
        }
    }

    // Fallback to directory name
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string());
    (name, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_nonexistent_path() {
        let result = detect_project(Path::new("/tmp/nonexistent-nixify-test-dir"));
        assert!(result.is_err());
    }
}
