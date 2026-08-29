{
  description = "eel bvr-svc (bvr-api, credential-api, verifier-api, channel-gateway) — reproducible Nix image";

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

        mkSvc = pname: subdir: pkgs.rustPlatform.buildRustPackage {
          inherit pname;
          version = "0.1.0";
          src = ../../../.;
          cargoLock.lockFile = ../../../Cargo.lock;
          buildAndTestSubdir = subdir;
        };
      in {
        packages = {
          bvr-api = mkSvc "bvr-api" "services/bvr-svc/crates/bvr-api";
          credential-api = mkSvc "credential-api" "services/bvr-svc/crates/credential";
          verifier-api = mkSvc "verifier-api" "services/bvr-svc/crates/verifier-api";
          channel-gateway = mkSvc "channel-gateway" "services/bvr-svc/crates/channel-gateway";
          default = mkSvc "bvr-api" "services/bvr-svc/crates/bvr-api";
        };
        devShells.default = pkgs.mkShell {
          packages = [ rustToolchain pkgs.cargo-nextest ];
        };
      }
    );
}
