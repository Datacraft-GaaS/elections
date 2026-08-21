{
  description = "eel python workspace flake, deterministic hello build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Deterministic Python wheel/sdist build
        helloPy = pkgs.python312Packages.buildPythonPackage {
          pname = "eel-hello-py";
          version = "0.1.0";
          src = ./src;
          format = "pyproject";
          nativeBuildInputs = [ pkgs.python312Packages.hatchling ];
          doCheck = true;
          checkPhase = ''
            python -m pytest tests/ -q
          '';
        };
      in {
        packages.default = helloPy;
        devShells.default = pkgs.mkShell {
          packages = [ pkgs.python312 pkgs.uv pkgs.ruff pkgs.pyright ];
          shellHook = ''
            python3 --version
            uv --version
          '';
        };
      }
    );
}
