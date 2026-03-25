use std::path::Path;

use colored::Colorize;

use crate::errors::Result;
use crate::validate::run_flake_check;

/// Run `nix flake check` on the given project directory and print the result.
pub fn run_check(path: &Path) -> Result<()> {
    println!(
        "{} Running {} in {}",
        "info:".cyan().bold(),
        "nix flake check".cyan(),
        path.display()
    );

    let result = run_flake_check(path)?;

    if result.success {
        println!("{} All checks passed!", "ok:".green().bold());
        if !result.stdout.is_empty() {
            println!("{}", result.stdout);
        }
    } else {
        println!("{} Flake check failed.", "error:".red().bold());
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
    }

    Ok(())
}
