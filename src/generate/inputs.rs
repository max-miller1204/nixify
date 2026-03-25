use crate::types::{BuildSystem, FlakeConfig, FlakeStyle, Language, ProjectInfo};

/// Determine which flake inputs are needed based on the project and config.
pub fn collect_inputs(info: &ProjectInfo, config: &FlakeConfig) -> Vec<String> {
    let mut inputs = vec!["nixpkgs".to_string()];

    let has_rust = info.languages.iter().any(|l| l.language == Language::Rust);
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    if has_rust {
        inputs.push("crane".to_string());
    }

    match config.style {
        FlakeStyle::Standalone => inputs.push("flake-utils".to_string()),
        FlakeStyle::FlakeParts => inputs.push("flake-parts".to_string()),
    }

    let uses_poetry = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python && l.build_system == BuildSystem::Poetry);
    if has_python && uses_poetry {
        inputs.push("poetry2nix".to_string());
    }

    for name in config.extra_inputs.keys() {
        if !inputs.contains(name) {
            inputs.push(name.clone());
        }
    }

    inputs.sort();
    inputs
}

/// Render the extra inputs block for template injection.
pub fn render_extra_inputs(config: &FlakeConfig) -> String {
    config
        .extra_inputs
        .iter()
        .map(|(name, url)| format!("{name}.url = \"{url}\";"))
        .collect::<Vec<_>>()
        .join("\n    ")
}

/// Render the extra input argument names for the outputs function signature.
pub fn render_extra_input_args(config: &FlakeConfig) -> String {
    if config.extra_inputs.is_empty() {
        String::new()
    } else {
        let mut names: Vec<_> = config.extra_inputs.keys().cloned().collect();
        names.sort();
        names.join(", ") + ", "
    }
}
