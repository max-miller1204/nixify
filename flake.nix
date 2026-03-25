{
  description = "nixify - Auto-detect project languages and generate working Nix flakes";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        # Include non-Rust files needed for build (templates) and tests (fixtures, snapshots)
        extraFilter = path: _type:
          (builtins.match ".*\\.nix$" path != null) ||
          (builtins.match ".*\\.snap$" path != null) ||
          (builtins.match ".*\\.toml$" path != null) ||
          (builtins.match ".*\\.py$" path != null) ||
          (builtins.match ".*\\.txt$" path != null) ||
          (builtins.match ".*\\.lock$" path != null);
        srcFilter = path: type:
          (extraFilter path type) || (craneLib.filterCargoSources path type);

        commonArgs = {
          src = pkgs.lib.cleanSourceWith {
            src = ./.;
            filter = srcFilter;
          };
          strictDeps = true;
          buildInputs = [ ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            pkgs.libiconv
          ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        nixify = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
        });
      in
      {
        checks = {
          inherit nixify;

          nixify-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          nixify-fmt = craneLib.cargoFmt {
            src = pkgs.lib.cleanSourceWith {
              src = ./.;
              filter = srcFilter;
            };
          };
        };

        packages = {
          default = nixify;
          inherit nixify;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [
            rust-analyzer
            cargo-insta
          ];
        };
      }
    );
}
