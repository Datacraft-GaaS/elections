{
  description = "eel rust workspace flake — deterministic hello build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rustToolchain = pkgs.rust-bin.stable."1.83.0".default;

        # Minimal hello binary used as reproducibility proof target
        helloRs = pkgs.stdenv.mkDerivation {
          name = "eel-hello-rs";
          version = "0.1.0";
          src = ./src;
          nativeBuildInputs = [ rustToolchain pkgs.cargo ];
          buildPhase = ''
            cargo build --release --manifest-path $src/Cargo.toml --target-dir $TMPDIR/target
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp $TMPDIR/target/release/eel-hello-rs $out/bin/
          '';
        };
      in {
        packages.default = helloRs;
        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain pkgs.cargo-nextest ];
          shellHook = ''
            rustc --version
            cargo --version
          '';
        };
      }
    );
}
