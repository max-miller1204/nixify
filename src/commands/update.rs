use colored::Colorize;

use crate::errors::Result;

/// Stub for the `update` subcommand. Not yet implemented.
pub fn run_update() -> Result<()> {
    println!(
        "{} `nixify update` is coming soon! For now, re-run `nixify init`.",
        "info:".cyan().bold()
    );
    Ok(())
}
