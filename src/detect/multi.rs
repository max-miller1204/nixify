use crate::errors::Result;
use crate::types::LanguageInfo;
use std::path::Path;

use super::python::PythonDetector;
use super::rust::RustDetector;
use super::Detector;

/// Run all registered detectors and collect results.
pub fn detect_all(path: &Path) -> Result<Vec<LanguageInfo>> {
    let detectors: Vec<Box<dyn Detector>> = vec![Box::new(RustDetector), Box::new(PythonDetector)];

    let mut languages = Vec::new();
    for detector in &detectors {
        if let Some(info) = detector.detect(path)? {
            languages.push(info);
        }
    }

    Ok(languages)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BuildSystem, Language};

    #[test]
    fn test_empty_directory() {
        let dir = tempfile::tempdir().unwrap();
        let result = detect_all(dir.path()).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_multi_language() {
        let dir = tempfile::tempdir().unwrap();

        // Rust
        std::fs::write(
            dir.path().join("Cargo.toml"),
            "[package]\nname = \"multi\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(dir.path().join("src")).unwrap();
        std::fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

        // Python
        std::fs::write(
            dir.path().join("pyproject.toml"),
            "[project]\nname = \"multi\"\nversion = \"0.1.0\"\n",
        )
        .unwrap();
        std::fs::write(dir.path().join("uv.lock"), "").unwrap();

        let result = detect_all(dir.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].language, Language::Rust);
        assert_eq!(result[0].build_system, BuildSystem::Cargo);
        assert_eq!(result[1].language, Language::Python);
        assert_eq!(result[1].build_system, BuildSystem::Uv);
    }
}
