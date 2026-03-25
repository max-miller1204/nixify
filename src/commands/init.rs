use std::path::{Path, PathBuf};

use colored::Colorize;
use similar::{ChangeTag, TextDiff};

use crate::config::{load_config_or_default, merge_config, CliOverrides};
use crate::errors::{NixifyError, Result};
use crate::report::print_report;
use crate::types::{FlakeConfig, FlakeStyle, GeneratedFlake, ProjectInfo};

/// Options for the `init` subcommand.
#[derive(Debug)]
pub struct InitOptions {
    pub path: PathBuf,
    pub dry_run: bool,
    pub diff: bool,
    pub style: FlakeStyle,
    pub force: bool,
    pub no_check: bool,
    pub config_path: Option<PathBuf>,
}

/// Resolve the final `FlakeConfig` by loading the config file and merging CLI overrides.
pub fn resolve_config(options: &InitOptions) -> Result<FlakeConfig> {
    let config_path = options
        .config_path
        .clone()
        .unwrap_or_else(|| options.path.join(".nixify.toml"));

    let file_config = load_config_or_default(&config_path)?;

    let overrides = CliOverrides {
        style: Some(options.style),
        ..Default::default()
    };

    Ok(merge_config(file_config, overrides))
}

/// Run the full init pipeline given already-computed project info and generated flake.
///
/// The detect and generate steps are performed by other units; this function
/// accepts their results as parameters so that `main.rs` can wire everything together.
pub fn run_init(
    project_info: &ProjectInfo,
    generated: &GeneratedFlake,
    options: &InitOptions,
) -> Result<()> {
    let flake_path = options.path.join("flake.nix");

    // --diff: show colored diff against existing flake.nix
    if options.diff {
        show_diff(&flake_path, &generated.content)?;
        return Ok(());
    }

    // --dry-run: print to stdout without writing
    if options.dry_run {
        println!("{}", generated.content);
        return Ok(());
    }

    // Write the flake
    write_flake(&flake_path, &generated.content, options.force)?;

    // Print the report
    print_report(project_info, generated);

    Ok(())
}

/// Write flake.nix to disk, creating backups of existing files.
///
/// If `force` is false and `flake.nix` already exists, backup the existing file:
/// - `flake.nix` -> `flake.nix.bak`
/// - If `flake.nix.bak` exists -> `flake.nix.bak.1`, `.bak.2`, etc.
pub fn write_flake(path: &Path, content: &str, force: bool) -> Result<()> {
    if path.exists() && !force {
        backup_file(path)?;
    }

    std::fs::write(path, content).map_err(|e| NixifyError::IoError {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;

    println!(
        "{} Wrote {}",
        "ok:".green().bold(),
        path.display().to_string().cyan()
    );

    Ok(())
}

/// Create a backup of an existing file with incrementing `.bak`, `.bak.1`, `.bak.2` suffixes.
fn backup_file(path: &Path) -> Result<()> {
    let bak = path.with_extension("nix.bak");

    let target = if !bak.exists() {
        bak
    } else {
        let mut n = 1u32;
        loop {
            let candidate = path.with_extension(format!("nix.bak.{n}"));
            if !candidate.exists() {
                break candidate;
            }
            n += 1;
        }
    };

    std::fs::copy(path, &target).map_err(|e| NixifyError::IoError {
        path: path.display().to_string(),
        reason: e.to_string(),
    })?;

    println!(
        "{} Backed up existing flake to {}",
        "info:".cyan().bold(),
        target.display().to_string().cyan()
    );

    Ok(())
}

/// Show a colored diff between the existing flake.nix and the new content.
fn show_diff(flake_path: &Path, new_content: &str) -> Result<()> {
    let old_content = if flake_path.exists() {
        std::fs::read_to_string(flake_path).map_err(|e| NixifyError::IoError {
            path: flake_path.display().to_string(),
            reason: e.to_string(),
        })?
    } else {
        String::new()
    };

    let diff = TextDiff::from_lines(old_content.as_str(), new_content);
    let mut has_changes = false;

    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Delete => {
                has_changes = true;
                print!("{}", format!("-{change}").red());
            }
            ChangeTag::Insert => {
                has_changes = true;
                print!("{}", format!("+{change}").green());
            }
            ChangeTag::Equal => {
                print!(" {change}");
            }
        }
    }

    if !has_changes {
        println!("{}", "No changes.".yellow());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_write_flake_creates_file() {
        let dir = TempDir::new().unwrap();
        let flake = dir.path().join("flake.nix");
        write_flake(&flake, "{ }", false).unwrap();
        assert_eq!(std::fs::read_to_string(&flake).unwrap(), "{ }");
    }

    #[test]
    fn test_write_flake_creates_backup() {
        let dir = TempDir::new().unwrap();
        let flake = dir.path().join("flake.nix");
        std::fs::write(&flake, "old content").unwrap();

        write_flake(&flake, "new content", false).unwrap();

        let bak = dir.path().join("flake.nix.bak");
        assert!(bak.exists());
        assert_eq!(std::fs::read_to_string(&bak).unwrap(), "old content");
        assert_eq!(std::fs::read_to_string(&flake).unwrap(), "new content");
    }

    #[test]
    fn test_write_flake_increments_backup() {
        let dir = TempDir::new().unwrap();
        let flake = dir.path().join("flake.nix");
        let bak = dir.path().join("flake.nix.bak");

        // Create initial file and first backup
        std::fs::write(&flake, "v1").unwrap();
        std::fs::write(&bak, "v0").unwrap();

        write_flake(&flake, "v2", false).unwrap();

        let bak1 = dir.path().join("flake.nix.bak.1");
        assert!(bak1.exists());
        assert_eq!(std::fs::read_to_string(&bak1).unwrap(), "v1");
        assert_eq!(std::fs::read_to_string(&flake).unwrap(), "v2");
    }

    #[test]
    fn test_write_flake_force_no_backup() {
        let dir = TempDir::new().unwrap();
        let flake = dir.path().join("flake.nix");
        std::fs::write(&flake, "old").unwrap();

        write_flake(&flake, "new", true).unwrap();

        let bak = dir.path().join("flake.nix.bak");
        assert!(!bak.exists());
        assert_eq!(std::fs::read_to_string(&flake).unwrap(), "new");
    }
}
