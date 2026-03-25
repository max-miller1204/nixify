# nixify

Auto-detect project languages and generate working Nix flakes.

Point `nixify` at any project directory and get a `flake.nix` with a dev shell, package output, and CI checks — no Nix knowledge required.

## Supported Languages

- **Rust** — Detects Cargo.toml (simple projects and workspaces). Uses [crane](https://github.com/ipetkov/crane) for builds.
- **Python** — Detects Poetry, pip/setuptools, uv, and pdm projects. Uses [poetry2nix](https://github.com/nix-community/poetry2nix) for Poetry projects.

## Usage

```sh
# Generate a flake.nix for the current directory
nixify init

# Generate for a specific path
nixify init ./my-project

# Preview without writing
nixify init --dry-run

# Use flake-parts style instead of standalone
nixify init --style flake-parts

# Show diff against existing flake.nix
nixify init --diff

# Run nix flake check on an existing flake
nixify check
```

## What It Generates

A complete `flake.nix` with:

- **Dev shell** — Language toolchain + common dev tools (e.g., rust-analyzer, clippy, rustfmt)
- **Package output** — `nix build` just works
- **CI checks** — Linting, formatting, and tests via `nix flake check`
- **Multi-system support** — x86_64-linux, aarch64-linux, x86_64-darwin, aarch64-darwin

## Configuration

Create a `.nixify.toml` to customize the generated flake:

```toml
[devshell]
extra_packages = ["ripgrep", "fd"]
shell_hook = "echo 'Welcome!'"
env_vars = { DATABASE_URL = "postgres://localhost/dev" }

[inputs]
extra = { my-overlay = "github:someone/overlay" }

[flake]
style = "flake-parts"

[overlays]
extra = ["my-overlay.overlays.default"]
```

CLI flags take precedence over config file values.

## Install

```sh
# Run directly
nix run github:yourusername/nixify -- init

# Or install into your profile
nix profile install github:yourusername/nixify
```

## Build from Source

```sh
nix build
# or
nix develop --command cargo build --release
```
