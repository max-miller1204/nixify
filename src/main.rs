use clap::{Parser, Subcommand};
use miette::Result;
use std::path::PathBuf;

use nixify::types::FlakeStyle;

#[derive(Parser)]
#[command(
    name = "nixify",
    version,
    about = "Auto-detect project languages and generate working Nix flakes"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Detect project language and generate a flake.nix
    Init {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Print generated flake.nix to stdout instead of writing
        #[arg(long)]
        dry_run: bool,

        /// Show colored diff against existing flake.nix
        #[arg(long)]
        diff: bool,

        /// Flake output style
        #[arg(long, default_value = "standalone")]
        style: FlakeStyle,

        /// Overwrite existing flake.nix without backup
        #[arg(long)]
        force: bool,

        /// Skip the validation prompt
        #[arg(long)]
        no_check: bool,

        /// Path to .nixify.toml config file
        #[arg(long)]
        config: Option<PathBuf>,

        /// Override detected language (skip detection)
        #[arg(long)]
        lang: Option<String>,
    },

    /// Run `nix flake check` on an existing flake
    Check {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Update an existing generated flake (coming soon)
    Update {
        /// Path to the project directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { .. } => {
            // Will be wired up by Unit 3
            eprintln!("nixify init: not yet implemented");
            Ok(())
        }
        Commands::Check { .. } => {
            eprintln!("nixify check: not yet implemented");
            Ok(())
        }
        Commands::Update { .. } => {
            use colored::Colorize;
            println!(
                "{} `nixify update` is coming soon! For now, re-run `nixify init`.",
                "info:".cyan().bold()
            );
            Ok(())
        }
    }
}
