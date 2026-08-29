{
  description = "eel inventory-svc (custody-api, printing-api) — reproducible Nix image";

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

        # Reproducible binaries for inventory-svc
        custodyApi = pkgs.rustPlatform.buildRustPackage {
          pname = "custody-api";
          version = "0.1.0";
          src = ../../../.;
          cargoLock.lockFile = ../../../Cargo.lock;
          buildAndTestSubdir = "services/inventory-svc/crates/custody";
        };
        printingApi = pkgs.rustPlatform.buildRustPackage {
          pname = "printing-api";
          version = "0.1.0";
          src = ../../../.;
          cargoLock.lockFile = ../../../Cargo.lock;
          buildAndTestSubdir = "services/inventory-svc/crates/printing";
        };
      in {
        packages = {
          custody-api = custodyApi;
          printing-api = printingApi;
          default = custodyApi;
        };
        # Copy of ADR-0006 template — extend per-service (see reproducibility/nix-flakes/rust)
        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain pkgs.cargo-nextest ];
        };
      }
    );
}
