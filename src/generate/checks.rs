use crate::types::{Language, ProjectInfo};

/// Collect the names of checks that will be generated.
pub fn collect_checks(info: &ProjectInfo) -> Vec<String> {
    let mut checks = Vec::new();

    let has_rust = info.languages.iter().any(|l| l.language == Language::Rust);
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if has_rust {
        checks.extend_from_slice(&[
            "crate".to_string(),
            "crate-clippy".to_string(),
            "crate-fmt".to_string(),
            "crate-test".to_string(),
        ]);
    }

    if has_python {
        checks.extend_from_slice(&["python-lint".to_string(), "python-typecheck".to_string()]);
    }

    checks
}

/// Render Python-specific checks for template injection.
pub fn render_python_checks(info: &ProjectInfo) -> String {
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if !has_python {
        return String::new();
    }

    r#"python-lint = pkgs.runCommand "python-lint" {
            nativeBuildInputs = [ pkgs.ruff ];
          } ''
            cd ${./.}
            ruff check .
            touch $out
          '';

          python-typecheck = pkgs.runCommand "python-typecheck" {
            nativeBuildInputs = [ pkgs.mypy ];
          } ''
            cd ${./.}
            mypy .
            touch $out
          '';"#
        .to_string()
}
