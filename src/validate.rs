use std::path::Path;
use std::process::Command;

use crate::errors::{NixifyError, Result};

/// Result of running `nix flake check`.
#[derive(Debug)]
pub struct CheckResult {
    pub success: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Run `nix flake check` on the given directory and capture the result.
pub fn run_flake_check(path: &Path) -> Result<CheckResult> {
    let nix = which_nix()?;

    let output = Command::new(&nix)
        .args(["flake", "check", "--no-build"])
        .arg(path)
        .output()
        .map_err(|e| NixifyError::IoError {
            path: nix.clone(),
            reason: e.to_string(),
        })?;

    Ok(CheckResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// Ensure the `nix` command is available on PATH.
fn which_nix() -> Result<String> {
    let output = Command::new("which")
        .arg("nix")
        .output()
        .map_err(|e| NixifyError::IoError {
            path: "which".into(),
            reason: e.to_string(),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(NixifyError::MissingCommand {
            command: "nix".into(),
        })
    }
}
