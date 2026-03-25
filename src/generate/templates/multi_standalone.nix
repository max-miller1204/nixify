{
  description = "{{DESCRIPTION}}";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    {{POETRY2NIX_INPUT}}
    {{EXTRA_INPUTS}}
  };

  outputs = { self, nixpkgs, crane, flake-utils, {{POETRY2NIX_ARG}}{{EXTRA_INPUT_ARGS}} }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;
        {{POETRY2NIX_LET}}

        # Rust
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = [
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        crate = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });

        # Python
        {{PYTHON_PACKAGE}}
        {{EXTRA_OVERLAYS}}
      in
      {
        checks = {
          inherit crate;

          crate-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          crate-fmt = craneLib.cargoFmt {
            src = craneLib.cleanCargoSource ./.;
          };

          crate-test = craneLib.cargoNextest (commonArgs // {
            inherit cargoArtifacts;
            partitions = 1;
            partitionType = "count";
          });

          {{PYTHON_CHECKS}}
          {{EXTRA_CHECKS}}
        };

        packages = {
          default = crate;
          rust = crate;
          {{PYTHON_PACKAGE_OUTPUT}}
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            # Rust
            rustc
            cargo
            rust-analyzer
            clippy
            rustfmt
            cargo-watch

            # Python
            {{PYTHON_DEV_PACKAGES}}

            {{EXTRA_PACKAGES}}
          ];

          inputsFrom = [
            crate
            {{PYTHON_INPUTS_FROM}}
          ];

          {{SHELL_HOOK}}

          {{ENV_VARS}}
        };
      });
}
