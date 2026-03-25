use crate::types::{BuildSystem, Language, ProjectInfo};

/// Render the Python package definition for the `let` block.
pub fn render_python_package(info: &ProjectInfo) -> String {
    let python_lang = info
        .languages
        .iter()
        .find(|l| l.language == Language::Python);

    let Some(lang) = python_lang else {
        return String::new();
    };

    match &lang.build_system {
        BuildSystem::Poetry => r#"pythonApp = poetry2nix.mkPoetryApplication {
          projectDir = ./.;
        };"#
        .to_string(),
        _ => {
            format!(
                r#"pythonApp = pkgs.python3Packages.buildPythonApplication {{
          pname = "{name}";
          version = "0.1.0";
          src = ./.;
          format = "setuptools";
        }};"#,
                name = info.name
            )
        }
    }
}

/// Render the Python default package reference.
pub fn render_python_default_package(info: &ProjectInfo) -> String {
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if has_python {
        "pythonApp".to_string()
    } else {
        "null".to_string()
    }
}

/// Render the Python package output for multi-language flakes.
pub fn render_python_package_output(info: &ProjectInfo) -> String {
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if has_python {
        "python = pythonApp;".to_string()
    } else {
        String::new()
    }
}

/// Render poetry2nix input line if needed.
pub fn render_poetry2nix_input(info: &ProjectInfo) -> String {
    let uses_poetry = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python && l.build_system == BuildSystem::Poetry);

    if uses_poetry {
        "poetry2nix.url = \"github:nix-community/poetry2nix\";".to_string()
    } else {
        String::new()
    }
}

/// Render poetry2nix argument in outputs function.
pub fn render_poetry2nix_arg(info: &ProjectInfo) -> String {
    let uses_poetry = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python && l.build_system == BuildSystem::Poetry);

    if uses_poetry {
        "poetry2nix, ".to_string()
    } else {
        String::new()
    }
}

/// Render poetry2nix let binding.
pub fn render_poetry2nix_let(info: &ProjectInfo) -> String {
    let uses_poetry = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python && l.build_system == BuildSystem::Poetry);

    if uses_poetry {
        "poetry2nix = poetry2nix.lib.mkPoetry2Nix { inherit pkgs; };".to_string()
    } else {
        String::new()
    }
}

/// Render the inputsFrom for Python in devShell.
pub fn render_python_inputs_from(info: &ProjectInfo) -> String {
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if has_python {
        "pythonApp".to_string()
    } else {
        String::new()
    }
}
