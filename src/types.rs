use std::collections::HashMap;
use std::path::PathBuf;

/// Supported programming languages
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Rust => write!(f, "Rust"),
            Language::Python => write!(f, "Python"),
        }
    }
}

/// Build system detected for a language
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildSystem {
    // Rust
    Cargo,
    CargoWorkspace { members: Vec<String> },
    // Python
    Poetry,
    Pip,
    Uv,
    Pdm,
    SetupTools,
}

impl std::fmt::Display for BuildSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildSystem::Cargo => write!(f, "cargo"),
            BuildSystem::CargoWorkspace { members } => {
                write!(f, "cargo workspace ({} crates)", members.len())
            }
            BuildSystem::Poetry => write!(f, "poetry"),
            BuildSystem::Pip => write!(f, "pip"),
            BuildSystem::Uv => write!(f, "uv"),
            BuildSystem::Pdm => write!(f, "pdm"),
            BuildSystem::SetupTools => write!(f, "setuptools"),
        }
    }
}

/// Project type: binary, library, or both
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectKind {
    Binary,
    Library,
    Both,
}

/// Information about a single detected language in the project
#[derive(Debug, Clone)]
pub struct LanguageInfo {
    pub language: Language,
    pub build_system: BuildSystem,
    pub kind: ProjectKind,
    /// Extra metadata (e.g., python version, rust edition)
    pub metadata: HashMap<String, String>,
}

/// Complete project detection result
#[derive(Debug, Clone)]
pub struct ProjectInfo {
    pub path: PathBuf,
    pub languages: Vec<LanguageInfo>,
    /// Project name (derived from directory name or config files)
    pub name: String,
    /// Project description if found in config files
    pub description: Option<String>,
}

impl ProjectInfo {
    pub fn is_multi_language(&self) -> bool {
        self.languages.len() > 1
    }

    pub fn primary_language(&self) -> Option<&LanguageInfo> {
        self.languages.first()
    }
}

/// Flake output style
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FlakeStyle {
    #[default]
    Standalone,
    FlakeParts,
}

impl std::fmt::Display for FlakeStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlakeStyle::Standalone => write!(f, "standalone"),
            FlakeStyle::FlakeParts => write!(f, "flake-parts"),
        }
    }
}

impl std::str::FromStr for FlakeStyle {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "standalone" => Ok(FlakeStyle::Standalone),
            "flake-parts" | "flakeparts" => Ok(FlakeStyle::FlakeParts),
            _ => Err(format!(
                "unknown flake style: '{s}' (expected 'standalone' or 'flake-parts')"
            )),
        }
    }
}

/// Resolved configuration from CLI flags + .nixify.toml
#[derive(Debug, Clone, Default)]
pub struct FlakeConfig {
    pub style: FlakeStyle,
    pub extra_packages: Vec<String>,
    pub shell_hook: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub extra_inputs: HashMap<String, String>,
    pub extra_overlays: Vec<String>,
}

/// Result of flake generation
#[derive(Debug, Clone)]
pub struct GeneratedFlake {
    pub content: String,
    pub style: FlakeStyle,
    pub inputs_used: Vec<String>,
    pub devshell_packages: Vec<String>,
    pub checks: Vec<String>,
}
