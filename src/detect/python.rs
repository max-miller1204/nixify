use crate::errors::{NixifyError, Result};
use crate::types::{BuildSystem, Language, LanguageInfo, ProjectKind};
use std::collections::HashMap;
use std::path::Path;

use super::Detector;

pub struct PythonDetector;

impl Detector for PythonDetector {
    fn detect(&self, path: &Path) -> Result<Option<LanguageInfo>> {
        let has_pyproject = path.join("pyproject.toml").exists();
        let has_setup_py = path.join("setup.py").exists();
        let has_setup_cfg = path.join("setup.cfg").exists();
        let has_requirements = path.join("requirements.txt").exists();
        let has_poetry_lock = path.join("poetry.lock").exists();
        let has_uv_lock = path.join("uv.lock").exists();
        let has_pdm_lock = path.join("pdm.lock").exists();

        if !has_pyproject
            && !has_setup_py
            && !has_setup_cfg
            && !has_requirements
            && !has_poetry_lock
            && !has_uv_lock
            && !has_pdm_lock
        {
            return Ok(None);
        }

        let build_system = detect_build_system(
            path,
            has_pyproject,
            has_poetry_lock,
            has_uv_lock,
            has_pdm_lock,
            has_setup_py,
        )?;
        let kind = detect_project_kind(path, has_pyproject)?;
        let metadata = extract_metadata(path, has_pyproject)?;

        Ok(Some(LanguageInfo {
            language: Language::Python,
            build_system,
            kind,
            metadata,
        }))
    }
}

fn detect_build_system(
    path: &Path,
    has_pyproject: bool,
    has_poetry_lock: bool,
    has_uv_lock: bool,
    has_pdm_lock: bool,
    has_setup_py: bool,
) -> Result<BuildSystem> {
    // Lock files are the strongest signal
    if has_poetry_lock {
        return Ok(BuildSystem::Poetry);
    }
    if has_uv_lock {
        return Ok(BuildSystem::Uv);
    }
    if has_pdm_lock {
        return Ok(BuildSystem::Pdm);
    }

    // Check pyproject.toml build-system
    if has_pyproject {
        let content = std::fs::read_to_string(path.join("pyproject.toml")).map_err(|e| {
            NixifyError::IoError {
                path: path.join("pyproject.toml").display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let doc: toml::Table =
            content
                .parse()
                .map_err(|e: toml::de::Error| NixifyError::TomlParseError {
                    path: path.join("pyproject.toml").display().to_string(),
                    reason: e.to_string(),
                })?;

        // Check build-system.requires for poetry
        if let Some(build_system) = doc.get("build-system") {
            if let Some(requires) = build_system.get("requires").and_then(|r| r.as_array()) {
                let requires_str: Vec<String> = requires
                    .iter()
                    .filter_map(|r| r.as_str().map(|s| s.to_lowercase()))
                    .collect();

                if requires_str.iter().any(|r| r.contains("poetry")) {
                    return Ok(BuildSystem::Poetry);
                }
                if requires_str.iter().any(|r| r.contains("pdm")) {
                    return Ok(BuildSystem::Pdm);
                }
                if requires_str.iter().any(|r| r.contains("setuptools")) {
                    return Ok(BuildSystem::SetupTools);
                }
            }
        }

        // Check for [tool.poetry] section
        if doc.get("tool").and_then(|t| t.get("poetry")).is_some() {
            return Ok(BuildSystem::Poetry);
        }

        // Check for [tool.uv] section
        if doc.get("tool").and_then(|t| t.get("uv")).is_some() {
            return Ok(BuildSystem::Uv);
        }

        // Check for [tool.pdm] section
        if doc.get("tool").and_then(|t| t.get("pdm")).is_some() {
            return Ok(BuildSystem::Pdm);
        }

        // Generic pyproject.toml with no clear build system
        return Ok(BuildSystem::Pip);
    }

    if has_setup_py {
        return Ok(BuildSystem::SetupTools);
    }

    // Fallback: requirements.txt or setup.cfg
    Ok(BuildSystem::Pip)
}

fn detect_project_kind(path: &Path, has_pyproject: bool) -> Result<ProjectKind> {
    if has_pyproject {
        let content = std::fs::read_to_string(path.join("pyproject.toml")).map_err(|e| {
            NixifyError::IoError {
                path: path.join("pyproject.toml").display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let doc: toml::Table =
            content
                .parse()
                .map_err(|e: toml::de::Error| NixifyError::TomlParseError {
                    path: path.join("pyproject.toml").display().to_string(),
                    reason: e.to_string(),
                })?;

        // Check for scripts (entry points = binary)
        let has_scripts = doc.get("project").and_then(|p| p.get("scripts")).is_some()
            || doc
                .get("tool")
                .and_then(|t| t.get("poetry"))
                .and_then(|p| p.get("scripts"))
                .is_some();

        if has_scripts {
            return Ok(ProjectKind::Both);
        }
    }

    // Python projects are libraries by default
    Ok(ProjectKind::Library)
}

fn extract_metadata(path: &Path, has_pyproject: bool) -> Result<HashMap<String, String>> {
    let mut metadata = HashMap::new();

    if has_pyproject {
        let content = std::fs::read_to_string(path.join("pyproject.toml")).map_err(|e| {
            NixifyError::IoError {
                path: path.join("pyproject.toml").display().to_string(),
                reason: e.to_string(),
            }
        })?;

        let doc: toml::Table =
            content
                .parse()
                .map_err(|e: toml::de::Error| NixifyError::TomlParseError {
                    path: path.join("pyproject.toml").display().to_string(),
                    reason: e.to_string(),
                })?;

        // Extract python version requirement
        if let Some(requires_python) = doc
            .get("project")
            .and_then(|p| p.get("requires-python"))
            .and_then(|r| r.as_str())
        {
            metadata.insert("requires_python".to_string(), requires_python.to_string());
        }

        // Extract project name
        let name = doc
            .get("project")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .or_else(|| {
                doc.get("tool")
                    .and_then(|t| t.get("poetry"))
                    .and_then(|p| p.get("name"))
                    .and_then(|n| n.as_str())
            });
        if let Some(n) = name {
            metadata.insert("name".to_string(), n.to_string());
        }

        // Extract version
        let version = doc
            .get("project")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .or_else(|| {
                doc.get("tool")
                    .and_then(|t| t.get("poetry"))
                    .and_then(|p| p.get("version"))
                    .and_then(|v| v.as_str())
            });
        if let Some(v) = version {
            metadata.insert("version".to_string(), v.to_string());
        }
    }

    Ok(metadata)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_python_markers() {
        let dir = tempfile::tempdir().unwrap();
        let detector = PythonDetector;
        let result = detector.detect(dir.path()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_requirements_txt_only() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("requirements.txt"), "flask>=2.0\n").unwrap();

        let detector = PythonDetector;
        let info = detector.detect(dir.path()).unwrap().unwrap();
        assert_eq!(info.language, Language::Python);
        assert_eq!(info.build_system, BuildSystem::Pip);
    }

    #[test]
    fn test_poetry_lock_detection() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[tool.poetry]\nname = \"test\"\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("poetry.lock"), "").unwrap();

        let detector = PythonDetector;
        let info = detector.detect(dir.path()).unwrap().unwrap();
        assert_eq!(info.build_system, BuildSystem::Poetry);
    }

    #[test]
    fn test_uv_lock_detection() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"test\"\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("uv.lock"), "").unwrap();

        let detector = PythonDetector;
        let info = detector.detect(dir.path()).unwrap().unwrap();
        assert_eq!(info.build_system, BuildSystem::Uv);
    }
}
