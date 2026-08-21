{
  description = "eel dart/flutter workspace flake — deterministic hello build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Build a standalone Dart CLI hello binary
        helloDart = pkgs.stdenv.mkDerivation {
          name = "eel-hello-dart";
          version = "0.1.0";
          src = ./src;
          nativeBuildInputs = [ pkgs.dart ];
          buildPhase = ''
            dart pub get --offline
            dart compile exe bin/main.dart -o eel-hello-dart
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp eel-hello-dart $out/bin/
          '';
        };
      in {
        packages.default = helloDart;
        devShells.default = pkgs.mkShell {
          packages = [ pkgs.flutter pkgs.dart ];
          shellHook = ''
            flutter --version | head -2
            dart --version
          '';
        };
      }
    );
}
