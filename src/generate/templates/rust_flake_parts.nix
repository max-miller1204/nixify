{
  description = "{{DESCRIPTION}}";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-parts.url = "github:hercules-ci/flake-parts";
    {{EXTRA_INPUTS}}
  };

  outputs = inputs@{ self, nixpkgs, crane, flake-parts, {{EXTRA_INPUT_ARGS}} }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { pkgs, system, ... }:
        let
          craneLib = crane.mkLib pkgs;

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
            {{EXTRA_CHECKS}}
          };

          packages.default = crate;

          devShells.default = craneLib.devShell {
            checks = self.checks.${system};

            packages = with pkgs; [
              rust-analyzer
              clippy
              rustfmt
              cargo-watch
              {{EXTRA_PACKAGES}}
            ];

            {{SHELL_HOOK}}

            {{ENV_VARS}}
          };
        };
    };
}
