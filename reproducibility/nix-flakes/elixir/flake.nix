{
  description = "eel elixir workspace flake, deterministic hello build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        helloEx = pkgs.beamPackages.mixRelease {
          pname = "eel-hello-ex";
          version = "0.1.0";
          src = ./src;
          mixFodDeps = pkgs.beamPackages.fetchMixDeps {
            pname = "eel-hello-ex-deps";
            src = ./src;
            version = "0.1.0";
            sha256 = pkgs.lib.fakeSha256;
          };
        };
      in {
        packages.default = helloEx;
        devShells.default = pkgs.mkShell {
          packages = [ pkgs.erlang_27 pkgs.elixir_1_17 pkgs.rebar3 ];
          shellHook = ''
            elixir --version
            mix --version
          '';
        };
      }
    );
}
