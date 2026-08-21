{
  description = "eel node workspace flake, deterministic hello build";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

        # Bundle a minimal Node CLI script as a deterministic derivation
        helloNode = pkgs.stdenv.mkDerivation {
          name = "eel-hello-node";
          version = "0.1.0";
          src = ./src;
          nativeBuildInputs = [ pkgs.nodejs_22 pkgs.nodePackages.pnpm ];
          buildPhase = ''
            export HOME=$TMPDIR
            pnpm install --frozen-lockfile
            node --input-type=module <<<"import('./index.mjs').then(m=>m.build())"
          '';
          installPhase = ''
            mkdir -p $out/bin
            cp -r dist $out/
            cat > $out/bin/eel-hello-node <<'EOF'
            #!/usr/bin/env node
            import('../dist/index.js').then(m => m.main());
            EOF
            chmod +x $out/bin/eel-hello-node
          '';
        };
      in {
        packages.default = helloNode;
        devShells.default = pkgs.mkShell {
          packages = [ pkgs.nodejs_22 pkgs.nodePackages.pnpm ];
          shellHook = ''
            node --version
            pnpm --version
          '';
        };
      }
    );
}
