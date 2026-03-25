# Project: nixify

## Goal
A Rust CLI tool that auto-detects project languages and build systems, then generates a working `flake.nix` with dev shell, package outputs, and CI checks — making Nix accessible to beginners who don't want to write Nix by hand.

## Target Companies
- **Flox** — directly in their mission of simplifying Nix for developers
- **Determinate Systems** — aligns with their Nix onboarding/DX tooling focus
- **Obsidian Systems** — demonstrates Nix + Rust + Haskell ecosystem knowledge
- **All others** — shows deep Nix fluency and ability to build developer tools

## Tech Stack
- **Language:** Rust
- **Build:** Nix flake (using crane for the Rust build itself)
- **CLI framework:** clap (derive macros)
- **Error handling:** miette for rich diagnostic error reports
- **Config parsing:** toml crate for .nixify.toml
- **Template engine:** None — templates embedded via `include_str!`, string interpolation for composition
- **Testing:** Unit tests + snapshot tests (insta) + integration tests (nix flake check)
- **Dependencies:** clap, miette, toml, serde, colored, insta (dev)

## Architecture

### Core Pipeline
```
detect → plan → generate → validate (optional) → report
```

1. **Detector** — Scans project directory for marker files + heuristics, produces a `ProjectInfo` struct describing what was found (languages, build systems, frameworks)
2. **Planner** — Takes `ProjectInfo` + user config/flags, decides which flake template strategy to use, resolves conflicts in multi-language projects
3. **Generator** — Produces the final `flake.nix` string from embedded templates, composing sections for inputs, outputs, devShell, packages, and checks
4. **Validator** — Optionally runs `nix flake check` on the generated output
5. **Reporter** — Prints rich colored summary: what was detected, what was generated, next steps, and beginner tips

### CLI Subcommands
- `nixify init [path]` — Main command. Detect + generate flake.nix (default path: `.`)
- `nixify check [path]` — Run `nix flake check` on existing flake
- `nixify update [path]` — Stubbed for v2 ("coming soon" message)

### CLI Flags (on `init`)
- `--dry-run` — Print generated flake.nix to stdout instead of writing
- `--diff` — Show colored diff against existing flake.nix
- `--style <standalone|flake-parts>` — Flake output style (default: standalone)
- `--force` — Overwrite existing flake.nix without backup
- `--no-check` — Skip the "would you like to validate?" prompt
- `--config <path>` — Path to .nixify.toml (default: `.nixify.toml`)
- `--lang <lang>` — Override detected language (skip detection)

### Config File (.nixify.toml)
```toml
[devshell]
extra_packages = ["ripgrep", "fd"]
shell_hook = "echo 'Welcome to the dev environment!'"
env_vars = { DATABASE_URL = "postgres://localhost/dev" }

[inputs]
extra = { my-overlay = "github:someone/overlay" }

[flake]
style = "flake-parts"  # or "standalone"

[overlays]
extra = ["my-overlay.overlays.default"]
```

Precedence: CLI flags > .nixify.toml > auto-detected defaults.

### Language Detection

**Rust:**
- Marker: `Cargo.toml`
- Heuristics: workspace detection (members field), binary vs library (src/main.rs vs src/lib.rs)
- Nix helper: crane
- DevShell: rustc, cargo, rust-analyzer, clippy, rustfmt

**Python:**
- Markers: `pyproject.toml`, `setup.py`, `setup.cfg`, `requirements.txt`, `uv.lock`, `pdm.lock`, `poetry.lock`
- Heuristics: detect poetry vs pip vs uv vs pdm from lock files and pyproject.toml build-system
- Nix helper: poetry2nix (for poetry), nixpkgs python builders (for others)
- DevShell: python3, pip, detected tooling (poetry/uv/pdm)

### Generated Flake Structure

**Standalone style:**
```nix
{
  description = "...";
  inputs = { ... };
  outputs = { self, nixpkgs, ... }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
    in {
      packages = forAllSystems (system: { ... });
      devShells = forAllSystems (system: { ... });
      checks = forAllSystems (system: { ... });
    };
}
```

**flake-parts style:**
```nix
{
  description = "...";
  inputs = { nixpkgs = ...; flake-parts = ...; ... };
  outputs = inputs: inputs.flake-parts.lib.mkFlake { inherit inputs; } {
    systems = [ ... ];
    perSystem = { pkgs, ... }: { ... };
  };
}
```

### Existing Flake Handling
- If `flake.nix` exists: copy to `flake.nix.bak`, then overwrite
- If `flake.nix.bak` already exists: use `flake.nix.bak.1`, `.bak.2`, etc.
- Print warning about backup location

### Post-Generation Report
Rich colored output:
```
✔ Detected: Rust (cargo workspace, 3 crates)
✔ Generated: flake.nix (standalone style)
✔ Backed up: flake.nix → flake.nix.bak

📦 What was created:
  • devShell with: rustc, cargo, rust-analyzer, clippy, rustfmt
  • Package: default (built with crane)
  • Check: cargo clippy, cargo test

🚀 Next steps:
  $ nix develop    # Enter the dev environment
  $ nix build      # Build the project
  $ nix flake check # Run all checks

💡 Tip: The devShell gives you all tools without installing them globally.
    Add extra packages in .nixify.toml under [devshell].extra_packages

Would you like to run `nix flake check` now? [y/N]
```

## Foundation Work (sequential)

### 1. Project scaffold
- Initialize Rust project with `flake.nix` that builds nixify itself using crane
- Set up directory structure, Cargo.toml with all dependencies
- Create `src/main.rs` with clap CLI skeleton (init, check, update subcommands)
- Create core types module with shared data structures (`ProjectInfo`, `FlakeConfig`, `Language`, `BuildSystem`, etc.)
- Create error types module using miette

**Files:**
- `flake.nix`
- `Cargo.toml`
- `src/main.rs`
- `src/lib.rs`
- `src/types.rs`
- `src/errors.rs`

## Work Units (parallel)

### Unit 1: Language Detection Engine
**Files owned:**
- `src/detect/mod.rs`
- `src/detect/rust.rs`
- `src/detect/python.rs`
- `src/detect/multi.rs`
- `tests/detect/mod.rs`
- `tests/detect/rust.rs`
- `tests/detect/python.rs`
- `tests/fixtures/rust-simple/Cargo.toml`
- `tests/fixtures/rust-simple/src/main.rs`
- `tests/fixtures/rust-workspace/Cargo.toml`
- `tests/fixtures/python-poetry/pyproject.toml`
- `tests/fixtures/python-poetry/poetry.lock`
- `tests/fixtures/python-uv/pyproject.toml`
- `tests/fixtures/python-uv/uv.lock`
- `tests/fixtures/python-pip/requirements.txt`
- `tests/fixtures/python-pip/setup.py`
- `tests/fixtures/multi-rust-python/Cargo.toml`
- `tests/fixtures/multi-rust-python/pyproject.toml`

**Description:** Implements the detection pipeline. Scans a directory for marker files, applies heuristics to determine language, build system, and framework details. Handles multi-language detection by composing results. Each language detector is a separate module implementing a `Detector` trait.

**Acceptance criteria:**
- Correctly detects Rust projects (simple binary, library, workspace)
- Correctly detects Python projects (poetry, pip/setuptools, uv, pdm)
- Handles multi-language repos by returning all detected languages
- Returns structured `ProjectInfo` with all relevant metadata
- Unit tests pass for all fixture projects

### Unit 2: Flake Generator
**Files owned:**
- `src/generate/mod.rs`
- `src/generate/inputs.rs`
- `src/generate/devshell.rs`
- `src/generate/packages.rs`
- `src/generate/checks.rs`
- `src/generate/standalone.rs`
- `src/generate/flake_parts.rs`
- `src/generate/templates/rust_standalone.nix`
- `src/generate/templates/rust_flake_parts.nix`
- `src/generate/templates/python_standalone.nix`
- `src/generate/templates/python_flake_parts.nix`
- `src/generate/templates/multi_standalone.nix`
- `src/generate/templates/multi_flake_parts.nix`
- `tests/generate/mod.rs`
- `tests/generate/snapshots/` (all snapshot files)

**Description:** Takes a `ProjectInfo` + `FlakeConfig` and produces a valid `flake.nix` string. Embedded templates are composed by section (inputs, devShell, packages, checks). Supports both standalone and flake-parts output styles. Multi-language flakes compose sections from each language.

**Acceptance criteria:**
- Generates valid standalone flake.nix for Rust projects (simple + workspace)
- Generates valid standalone flake.nix for Python projects (poetry + pip)
- Generates valid flake-parts style for both languages
- Generates composed multi-language flake
- All snapshot tests pass (generated output matches expected)
- Generated flakes include proper inputs (crane, poetry2nix, etc.)

### Unit 3: Config, Validation & CLI Integration
**Files owned:**
- `src/config.rs`
- `src/validate.rs`
- `src/report.rs`
- `src/commands/mod.rs`
- `src/commands/init.rs`
- `src/commands/check.rs`
- `src/commands/update.rs`
- `tests/config/mod.rs`
- `tests/fixtures/config/full.toml`
- `tests/fixtures/config/minimal.toml`

**Description:** Implements .nixify.toml parsing and merging with CLI flags. Implements the validation step (running `nix flake check`). Implements the rich reporter that prints detection summary, generation results, next steps, and beginner tips. Wires up the init/check/update subcommand handlers that orchestrate detect → plan → generate → validate → report.

**Acceptance criteria:**
- Parses .nixify.toml with all supported fields (extra_packages, shell_hook, env_vars, extra inputs, overlays, style)
- CLI flags override config file values
- `nixify init` orchestrates full pipeline: detect → generate → backup → write → report
- `nixify check` runs nix flake check and reports results
- `nixify update` prints "coming soon" stub
- `--dry-run` prints to stdout without writing
- `--diff` shows colored diff against existing flake
- Rich reporter shows colored summary with next steps and tips
- Backup logic works (flake.nix → flake.nix.bak, handles existing backups)
- miette errors display properly for common failure modes

## Verification

### Automated
1. `cargo test` — all unit + snapshot tests pass
2. `nix build` — nixify itself builds via the project's own flake.nix
3. `nix flake check` — all checks pass on nixify's own flake

### Manual end-to-end
1. Create a fresh Rust project (`cargo init /tmp/test-rust`) → run `nixify init` → verify flake.nix is valid → `nix build` and `nix develop` work
2. Create a fresh Poetry project → run `nixify init` → verify flake.nix works
3. Create a multi-language project → run `nixify init` → verify composed flake
4. Test `--dry-run`, `--diff`, `--style=flake-parts` flags
5. Test `.nixify.toml` config overrides
6. Test existing flake.nix backup behavior
7. Test error cases: empty directory, unrecognized project, invalid config
