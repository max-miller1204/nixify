use crate::errors::Result;
use crate::types::{FlakeConfig, FlakeStyle, GeneratedFlake, Language, ProjectInfo};

use super::checks::{collect_checks, render_python_checks};
use super::devshell::{
    render_env_vars, render_extra_packages, render_python_dev_packages, render_shell_hook,
};
use super::inputs::{collect_inputs, render_extra_input_args, render_extra_inputs};
use super::packages::{
    render_poetry2nix_arg, render_poetry2nix_input, render_poetry2nix_let,
    render_python_default_package, render_python_inputs_from, render_python_package,
    render_python_package_output,
};

const RUST_TEMPLATE: &str = include_str!("templates/rust_flake_parts.nix");
const PYTHON_TEMPLATE: &str = include_str!("templates/python_flake_parts.nix");
const MULTI_TEMPLATE: &str = include_str!("templates/multi_flake_parts.nix");

pub fn generate_flake_parts(info: &ProjectInfo, config: &FlakeConfig) -> Result<GeneratedFlake> {
    let inputs = collect_inputs(info, config);
    let checks = collect_checks(info);
    let devshell_packages = super::devshell::collect_devshell_packages(info, config);

    let description = info
        .description
        .clone()
        .unwrap_or_else(|| format!("Nix flake for {}", info.name));

    let is_multi = info.is_multi_language();
    let has_rust = info.languages.iter().any(|l| l.language == Language::Rust);
    let has_python = info
        .languages
        .iter()
        .any(|l| l.language == Language::Python);

    let template = if is_multi {
        MULTI_TEMPLATE
    } else if has_rust {
        RUST_TEMPLATE
    } else if has_python {
        PYTHON_TEMPLATE
    } else {
        RUST_TEMPLATE // fallback
    };

    let content = apply_replacements(template, info, config, &description);

    Ok(GeneratedFlake {
        content: clean_output(&content),
        style: FlakeStyle::FlakeParts,
        inputs_used: inputs,
        devshell_packages,
        checks,
    })
}

fn apply_replacements(
    template: &str,
    info: &ProjectInfo,
    config: &FlakeConfig,
    description: &str,
) -> String {
    template
        .replace("{{DESCRIPTION}}", description)
        .replace("{{EXTRA_INPUTS}}", &render_extra_inputs(config))
        .replace("{{EXTRA_INPUT_ARGS}}", &render_extra_input_args(config))
        .replace("{{EXTRA_PACKAGES}}", &render_extra_packages(config))
        .replace("{{SHELL_HOOK}}", &render_shell_hook(config))
        .replace("{{ENV_VARS}}", &render_env_vars(config))
        .replace("{{EXTRA_OVERLAYS}}", &render_extra_overlays(config))
        .replace("{{EXTRA_CHECKS}}", "")
        .replace("{{POETRY2NIX_INPUT}}", &render_poetry2nix_input(info))
        .replace("{{POETRY2NIX_ARG}}", &render_poetry2nix_arg(info))
        .replace("{{POETRY2NIX_LET}}", &render_poetry2nix_let(info))
        .replace("{{PYTHON_PACKAGE}}", &render_python_package(info))
        .replace(
            "{{PYTHON_DEFAULT_PACKAGE}}",
            &render_python_default_package(info),
        )
        .replace(
            "{{PYTHON_PACKAGE_OUTPUT}}",
            &render_python_package_output(info),
        )
        .replace("{{PYTHON_DEV_PACKAGES}}", &render_python_dev_packages(info))
        .replace("{{PYTHON_CHECKS}}", &render_python_checks(info))
        .replace("{{PYTHON_INPUTS_FROM}}", &render_python_inputs_from(info))
}

fn render_extra_overlays(config: &FlakeConfig) -> String {
    if config.extra_overlays.is_empty() {
        return String::new();
    }
    config.extra_overlays.join("\n        ")
}

/// Remove empty lines that result from unused placeholders and trim trailing whitespace.
fn clean_output(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::new();
    let mut prev_empty = false;

    for line in lines {
        let trimmed = line.trim();
        let is_empty = trimmed.is_empty();

        if is_empty && prev_empty {
            continue;
        }

        result.push(line);
        prev_empty = is_empty;
    }

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}
