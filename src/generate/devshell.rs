use crate::types::{BuildSystem, FlakeConfig, Language, ProjectInfo};

/// Collect devshell packages for the project.
pub fn collect_devshell_packages(info: &ProjectInfo, config: &FlakeConfig) -> Vec<String> {
    let mut packages = Vec::new();

    for lang in &info.languages {
        match lang.language {
            Language::Rust => {
                packages.extend_from_slice(&[
                    "rust-analyzer".to_string(),
                    "clippy".to_string(),
                    "rustfmt".to_string(),
                    "cargo-watch".to_string(),
                ]);
            }
            Language::Python => {
                packages.push("python3".to_string());
                match &lang.build_system {
                    BuildSystem::Poetry => packages.push("poetry".to_string()),
                    BuildSystem::Pip | BuildSystem::SetupTools => {
                        packages.push("python3Packages.pip".to_string());
                    }
                    BuildSystem::Uv => packages.push("uv".to_string()),
                    BuildSystem::Pdm => packages.push("pdm".to_string()),
                    _ => {}
                }
                packages.push("ruff".to_string());
                packages.push("mypy".to_string());
            }
        }
    }

    for extra in &config.extra_packages {
        if !packages.contains(extra) {
            packages.push(extra.clone());
        }
    }

    packages
}

/// Render extra packages for template injection.
pub fn render_extra_packages(config: &FlakeConfig) -> String {
    config.extra_packages.join("\n            ")
}

/// Render the shell hook for template injection.
pub fn render_shell_hook(config: &FlakeConfig) -> String {
    match &config.shell_hook {
        Some(hook) => format!("shellHook = ''\n            {hook}\n          '';"),
        None => String::new(),
    }
}

/// Render environment variables for template injection.
pub fn render_env_vars(config: &FlakeConfig) -> String {
    if config.env_vars.is_empty() {
        return String::new();
    }
    let mut entries: Vec<_> = config.env_vars.iter().collect();
    entries.sort_by_key(|(k, _)| (*k).clone());
    entries
        .iter()
        .map(|(k, v)| format!("{k} = \"{v}\";"))
        .collect::<Vec<_>>()
        .join("\n          ")
}

/// Render Python-specific dev packages for template injection.
pub fn render_python_dev_packages(info: &ProjectInfo) -> String {
    let mut packages = Vec::new();

    for lang in &info.languages {
        if lang.language == Language::Python {
            packages.push("python3".to_string());
            match &lang.build_system {
                BuildSystem::Poetry => packages.push("poetry".to_string()),
                BuildSystem::Pip | BuildSystem::SetupTools => {
                    packages.push("python3Packages.pip".to_string());
                }
                BuildSystem::Uv => packages.push("uv".to_string()),
                BuildSystem::Pdm => packages.push("pdm".to_string()),
                _ => {}
            }
            packages.push("ruff".to_string());
            packages.push("mypy".to_string());
        }
    }

    packages.join("\n            ")
}
