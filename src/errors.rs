use miette::Diagnostic;
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
pub enum NixifyError {
    #[error("No supported language detected in '{path}'")]
    #[diagnostic(
        code(nixify::no_language),
        help("nixify supports Rust (Cargo.toml) and Python (pyproject.toml, setup.py, requirements.txt) projects.\nMake sure you're running nixify in the project root directory.")
    )]
    NoLanguageDetected { path: String },

    #[error("Failed to read '{path}': {reason}")]
    #[diagnostic(code(nixify::io_error))]
    IoError { path: String, reason: String },

    #[error("Failed to parse config file '{path}'")]
    #[diagnostic(
        code(nixify::config_parse),
        help("Check that your .nixify.toml is valid TOML.\nSee https://github.com/user/nixify for config examples.")
    )]
    ConfigParseError {
        path: String,
        #[source]
        source: toml::de::Error,
    },

    #[error("Failed to parse '{path}' as TOML")]
    #[diagnostic(code(nixify::toml_parse))]
    TomlParseError { path: String, reason: String },

    #[error("flake.nix already exists at '{path}'")]
    #[diagnostic(
        code(nixify::flake_exists),
        help("Use --force to overwrite, or nixify will create a backup at flake.nix.bak")
    )]
    FlakeAlreadyExists { path: String },

    #[error("`nix flake check` failed")]
    #[diagnostic(
        code(nixify::check_failed),
        help(
            "The generated flake.nix may have issues. Run `nix flake check` manually for details."
        )
    )]
    CheckFailed { stderr: String },

    #[error("'{command}' not found")]
    #[diagnostic(
        code(nixify::missing_command),
        help("Make sure Nix is installed. Visit https://nixos.org/download for installation instructions.")
    )]
    MissingCommand { command: String },

    #[error("{message}")]
    #[diagnostic(code(nixify::generate))]
    GenerationError { message: String },
}

impl From<std::io::Error> for NixifyError {
    fn from(err: std::io::Error) -> Self {
        NixifyError::IoError {
            path: String::new(),
            reason: err.to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, NixifyError>;
