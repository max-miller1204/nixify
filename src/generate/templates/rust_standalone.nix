{
  description = "{{DESCRIPTION}}";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
    {{EXTRA_INPUTS}}
  };

  outputs = { self, nixpkgs, crane, flake-utils, {{EXTRA_INPUT_ARGS}} }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
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
      });
}
