{
  description = "{{DESCRIPTION}}";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    {{POETRY2NIX_INPUT}}
    {{EXTRA_INPUTS}}
  };

  outputs = { self, nixpkgs, flake-utils, {{POETRY2NIX_ARG}}{{EXTRA_INPUT_ARGS}} }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
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
      });
}
