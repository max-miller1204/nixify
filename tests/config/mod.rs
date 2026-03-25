use std::path::Path;

use nixify::config::{load_config, merge_config, parse_config, CliOverrides};
use nixify::types::FlakeStyle;

#[test]
fn test_load_full_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config/full.toml");
    let cfg = load_config(&path).unwrap();

    assert_eq!(cfg.style, FlakeStyle::FlakeParts);
    assert_eq!(cfg.extra_packages, vec!["ripgrep", "fd"]);
    assert_eq!(
        cfg.shell_hook.as_deref(),
        Some("echo 'Welcome to the dev environment!'")
    );
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
fn test_load_minimal_config() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/config/minimal.toml");
    let cfg = load_config(&path).unwrap();

    assert_eq!(cfg.style, FlakeStyle::Standalone);
    assert!(cfg.extra_packages.is_empty());
    assert!(cfg.shell_hook.is_none());
    assert!(cfg.env_vars.is_empty());
    assert!(cfg.extra_inputs.is_empty());
    assert!(cfg.extra_overlays.is_empty());
}

#[test]
fn test_parse_empty_config_gives_defaults() {
    let cfg = parse_config("", Path::new("empty.toml")).unwrap();
    assert_eq!(cfg.style, FlakeStyle::Standalone);
    assert!(cfg.extra_packages.is_empty());
}

#[test]
fn test_merge_cli_style_overrides_file() {
    let file_cfg = parse_config(
        "[flake]\nstyle = \"standalone\"",
        Path::new("test.toml"),
    )
    .unwrap();

    let overrides = CliOverrides {
        style: Some(FlakeStyle::FlakeParts),
        ..Default::default()
    };

    let merged = merge_config(file_cfg, overrides);
    assert_eq!(merged.style, FlakeStyle::FlakeParts);
}

#[test]
fn test_merge_cli_packages_override_file() {
    let file_cfg = parse_config(
        "[devshell]\nextra_packages = [\"ripgrep\"]",
        Path::new("test.toml"),
    )
    .unwrap();

    let overrides = CliOverrides {
        extra_packages: vec!["fd".into(), "bat".into()],
        ..Default::default()
    };

    let merged = merge_config(file_cfg, overrides);
    assert_eq!(merged.extra_packages, vec!["fd", "bat"]);
}

#[test]
fn test_invalid_toml_error() {
    let result = parse_config("[[[bad toml", Path::new("bad.toml"));
    assert!(result.is_err());
}

#[test]
fn test_invalid_style_error() {
    let result = parse_config(
        "[flake]\nstyle = \"not-a-style\"",
        Path::new("test.toml"),
    );
    assert!(result.is_err());
}
