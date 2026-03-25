{
  description = "{{DESCRIPTION}}";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    {{POETRY2NIX_INPUT}}
    {{EXTRA_INPUTS}}
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, {{POETRY2NIX_ARG}}{{EXTRA_INPUT_ARGS}} }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      perSystem = { pkgs, system, ... }:
        let
          {{POETRY2NIX_LET}}
          {{PYTHON_PACKAGE}}
          {{EXTRA_OVERLAYS}}
        in
        {
          checks = {
            {{PYTHON_CHECKS}}
            {{EXTRA_CHECKS}}
          };

          packages.default = {{PYTHON_DEFAULT_PACKAGE}};

          devShells.default = pkgs.mkShell {
            packages = with pkgs; [
              {{PYTHON_DEV_PACKAGES}}
              {{EXTRA_PACKAGES}}
            ];

            inputsFrom = [ {{PYTHON_INPUTS_FROM}} ];

            {{SHELL_HOOK}}

            {{ENV_VARS}}
          };
        };
    };
}
