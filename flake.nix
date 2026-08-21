{
  description = "eel monorepo development shells";

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

        rustToolchain = pkgs.rust-bin.stable."1.83.0".default.override {
          extensions = [ "rust-src" "rust-analyzer" "clippy" "rustfmt" "llvm-tools-preview" ];
        };

        # Shared shell hook helpers
        printVersions = tools: ''
          echo "--- eel dev shell ---"
          ${builtins.concatStringsSep "\n" (map (t: ''
            ${t} --version 2>/dev/null || echo "${t}: not versioned"
          '') tools)}
          echo "---------------------"
        '';

        rustShell = pkgs.mkShell {
          name = "eel-rust";
          packages = with pkgs; [
            rustToolchain
            cargo-nextest
            cargo-hakari
            cargo-audit
            cargo-deny
            cargo-fuzz
            sccache
            sqlx-cli
            buf
            just
            # spectral via npm, not in nixpkgs stable, use node wrapper
            nodejs_22
          ];
          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          shellHook = ''
            ${printVersions [ "rustc" "cargo" "cargo-nextest" "sqlx" "buf" "just" ]}
          '';
        };

        pythonShell = pkgs.mkShell {
          name = "eel-python";
          packages = with pkgs; [
            python312
            uv
            ruff
            pyright
            just
          ];
          shellHook = ''
            ${printVersions [ "python3" "uv" "ruff" "pyright" "just" ]}
          '';
        };

        elixirShell = pkgs.mkShell {
          name = "eel-elixir";
          packages = with pkgs; [
            erlang_27
            elixir_1_17
            rebar3
            just
          ];
          shellHook = ''
            ${printVersions [ "erl" "elixir" "mix" "rebar3" "just" ]}
          '';
        };

        dartShell = pkgs.mkShell {
          name = "eel-dart";
          packages = with pkgs; [
            flutter
            just
          ];
          shellHook = ''
            ${printVersions [ "flutter" "dart" "just" ]}
          '';
        };

        nodeShell = pkgs.mkShell {
          name = "eel-node";
          packages = with pkgs; [
            nodejs_22
            nodePackages.pnpm
            just
          ];
          shellHook = ''
            ${printVersions [ "node" "pnpm" "just" ]}
            # install spectral and openapi-generator-cli locally if not present
            if ! command -v spectral &>/dev/null; then
              echo "tip: pnpm add -g @stoplight/spectral-cli openapi-generator-cli"
            fi
          '';
        };

        defaultShell = pkgs.mkShell {
          name = "eel-default";
          packages = with pkgs; [
            rustToolchain
            cargo-nextest
            cargo-hakari
            cargo-audit
            cargo-deny
            cargo-fuzz
            sccache
            sqlx-cli
            buf
            python312
            uv
            ruff
            pyright
            erlang_27
            elixir_1_17
            rebar3
            flutter
            nodejs_22
            nodePackages.pnpm
            just
          ];
          RUSTC_WRAPPER = "${pkgs.sccache}/bin/sccache";
          shellHook = ''
            echo "--- eel default dev shell (all languages) ---"
            rustc --version
            cargo --version
            python3 --version
            uv --version
            elixir --version | head -1
            flutter --version | head -1
            node --version
            pnpm --version
            just --version
            echo "---------------------------------------------"
          '';
        };

      in {
        devShells = {
          rust    = rustShell;
          python  = pythonShell;
          elixir  = elixirShell;
          dart    = dartShell;
          node    = nodeShell;
          default = defaultShell;
        };
      }
    );
}
