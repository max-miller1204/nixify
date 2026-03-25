use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::errors::{NixifyError, Result};
use crate::types::{FlakeConfig, FlakeStyle};

/// Raw TOML representation of .nixify.toml
#[derive(Debug, Deserialize, Default)]
struct RawConfig {
    #[serde(default)]
    devshell: DevshellSection,
    #[serde(default)]
    inputs: InputsSection,
    #[serde(default)]
    flake: FlakeSection,
    #[serde(default)]
    overlays: OverlaysSection,
}

#[derive(Debug, Deserialize, Default)]
struct DevshellSection {
    #[serde(default)]
    extra_packages: Vec<String>,
    shell_hook: Option<String>,
    #[serde(default)]
    env_vars: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct InputsSection {
    #[serde(default)]
    extra: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default)]
struct FlakeSection {
    style: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct OverlaysSection {
    #[serde(default)]
    extra: Vec<String>,
}

/// Load and parse a `.nixify.toml` config file from the given path.
pub fn load_config(path: &Path) -> Result<FlakeConfig> {
    let content = std::fs::read_to_string(path).map_err(|e| NixifyError::IoError {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;

    parse_config(&content, path)
}

/// Parse TOML content into a `FlakeConfig`.
pub fn parse_config(content: &str, path: &Path) -> Result<FlakeConfig> {
    let raw: RawConfig = toml::from_str(content).map_err(|e| NixifyError::ConfigParseError {
        path: path.display().to_string(),
        source: e,
    })?;

    let style = match raw.flake.style {
        Some(s) => s
            .parse::<FlakeStyle>()
            .map_err(|reason| NixifyError::TomlParseError {
                path: path.display().to_string(),
                reason,
            })?,
        None => FlakeStyle::default(),
    };

    Ok(FlakeConfig {
        style,
        extra_packages: raw.devshell.extra_packages,
        shell_hook: raw.devshell.shell_hook,
        env_vars: raw.devshell.env_vars,
        extra_inputs: raw.inputs.extra,
        extra_overlays: raw.overlays.extra,
    })
}

/// Load config from a path if it exists; otherwise return default.
pub fn load_config_or_default(path: &Path) -> Result<FlakeConfig> {
    if path.exists() {
        load_config(path)
    } else {
        Ok(FlakeConfig::default())
    }
}

/// CLI-provided overrides that take precedence over the config file.
#[derive(Debug, Default)]
pub struct CliOverrides {
    pub style: Option<FlakeStyle>,
    pub extra_packages: Vec<String>,
    pub shell_hook: Option<String>,
    pub env_vars: HashMap<String, String>,
}

/// Merge a file-based `FlakeConfig` with CLI overrides. CLI takes precedence.
pub fn merge_config(file_config: FlakeConfig, overrides: CliOverrides) -> FlakeConfig {
    FlakeConfig {
        style: overrides.style.unwrap_or(file_config.style),
        extra_packages: if overrides.extra_packages.is_empty() {
            file_config.extra_packages
        } else {
            overrides.extra_packages
        },
        shell_hook: overrides.shell_hook.or(file_config.shell_hook),
        env_vars: if overrides.env_vars.is_empty() {
            file_config.env_vars
        } else {
            overrides.env_vars
        },
        extra_inputs: file_config.extra_inputs,
        extra_overlays: file_config.extra_overlays,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
[devshell]
extra_packages = ["ripgrep", "fd"]
shell_hook = "echo 'Welcome!'"
env_vars = { DATABASE_URL = "postgres://localhost/dev" }

[inputs]
extra = { my-overlay = "github:someone/overlay" }

[flake]
style = "flake-parts"

[overlays]
extra = ["my-overlay.overlays.default"]
"#;
        let cfg = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(cfg.style, FlakeStyle::FlakeParts);
        assert_eq!(cfg.extra_packages, vec!["ripgrep", "fd"]);
        assert_eq!(cfg.shell_hook.as_deref(), Some("echo 'Welcome!'"));
        assert_eq!(
            cfg.env_vars.get("DATABASE_URL").unwrap(),
            "postgres://localhost/dev"
        );
        assert_eq!(
            cfg.extra_inputs.get("my-overlay").unwrap(),
            "github:someone/overlay"
        );
        assert_eq!(cfg.extra_overlays, vec!["my-overlay.overlays.default"]);
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = "";
        let cfg = parse_config(toml, Path::new("test.toml")).unwrap();
        assert_eq!(cfg.style, FlakeStyle::Standalone);
        assert!(cfg.extra_packages.is_empty());
        assert!(cfg.shell_hook.is_none());
        assert!(cfg.env_vars.is_empty());
        assert!(cfg.extra_inputs.is_empty());
        assert!(cfg.extra_overlays.is_empty());
    }

    #[test]
    fn test_merge_cli_overrides_style() {
        let file_cfg = FlakeConfig {
            style: FlakeStyle::Standalone,
            ..Default::default()
        };
        let overrides = CliOverrides {
            style: Some(FlakeStyle::FlakeParts),
            ..Default::default()
        };
        let merged = merge_config(file_cfg, overrides);
        assert_eq!(merged.style, FlakeStyle::FlakeParts);
    }

    #[test]
    fn test_merge_cli_overrides_packages() {
        let file_cfg = FlakeConfig {
            extra_packages: vec!["ripgrep".into()],
            ..Default::default()
        };
        let overrides = CliOverrides {
            extra_packages: vec!["fd".into()],
            ..Default::default()
        };
        let merged = merge_config(file_cfg, overrides);
        assert_eq!(merged.extra_packages, vec!["fd"]);
    }

    #[test]
    fn test_merge_preserves_file_when_no_overrides() {
        let file_cfg = FlakeConfig {
            style: FlakeStyle::FlakeParts,
            extra_packages: vec!["ripgrep".into()],
            shell_hook: Some("echo hi".into()),
            env_vars: HashMap::from([("FOO".into(), "bar".into())]),
            extra_inputs: HashMap::from([("x".into(), "github:x/y".into())]),
            extra_overlays: vec!["x.overlays.default".into()],
        };
        let overrides = CliOverrides::default();
        let merged = merge_config(file_cfg.clone(), overrides);
        assert_eq!(merged.style, file_cfg.style);
        assert_eq!(merged.extra_packages, file_cfg.extra_packages);
        assert_eq!(merged.shell_hook, file_cfg.shell_hook);
        assert_eq!(merged.env_vars, file_cfg.env_vars);
        assert_eq!(merged.extra_inputs, file_cfg.extra_inputs);
        assert_eq!(merged.extra_overlays, file_cfg.extra_overlays);
    }

    #[test]
    fn test_invalid_toml_returns_error() {
        let toml = "this is not valid toml [[[";
        let result = parse_config(toml, Path::new("bad.toml"));
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_style_returns_error() {
        let toml = r#"
[flake]
style = "invalid-style"
"#;
        let result = parse_config(toml, Path::new("test.toml"));
        assert!(result.is_err());
    }
}
