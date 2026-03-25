use clap::{Parser, Subcommand};
use miette::{IntoDiagnostic, Result};
use std::path::PathBuf;

use nixify::commands::check::run_check;
use nixify::commands::init::{resolve_config, run_init, InitOptions};
use nixify::commands::update::run_update;
use nixify::detect::detect_project;
use nixify::generate::generate;
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
        Commands::Init {
            path,
            dry_run,
            diff,
            style,
            force,
            no_check,
            config,
            lang: _,
        } => {
            let options = InitOptions {
                path: path.clone(),
                dry_run,
                diff,
                style,
                force,
                no_check,
                config_path: config,
            };

            let project_info = detect_project(&path).into_diagnostic()?;
            let flake_config = resolve_config(&options).into_diagnostic()?;
            let generated = generate(&project_info, &flake_config).into_diagnostic()?;

            run_init(&project_info, &generated, &options).into_diagnostic()?;

            Ok(())
        }
        Commands::Check { path } => run_check(&path).into_diagnostic(),
        Commands::Update { .. } => run_update().into_diagnostic(),
    }
}
