# tools/spicedb/

Host-side wrapper for the SpiceDB CLI, avoiding the naming clash with the unrelated `zed` code editor.

## Files

- `zed` (executable): resolves the SpiceDB `zed` CLI unambiguously. Never invokes the Zed code editor. Resolution order: `SPICEDB_ZED_BIN` env, Homebrew tap path, `~/.local/bin/spicedb-zed`, `~/.cargo/bin/spicedb-zed`, Nix flake output, PATH `zed` (verified to be the SpiceDB build). Fails with an install hint if none match.

## When to use the wrapper vs bare `zed`

| context | invoke as | rationale |
|---|---|---|
| justfile, .woodpecker/*, pre-commit, dev scripts on the host | `./tools/spicedb/zed <subcommand>` | host `zed` on PATH may be the code editor |
| Inside a container built from `authzed/zed:*` (e.g. `platform/spicedb/local/schema-seed-job.yaml`) | bare `zed` | image ships the SpiceDB CLI at `/usr/local/bin/zed`; no editor present |
| ADRs, docs, plans (prose only) | write "the SpiceDB `zed` CLI" the first time; then just `zed` in monospace | context makes it clear |

## Install the underlying CLI

- **macOS**: `brew install authzed/tap/zed`
- **Nix**: `nix shell nixpkgs#spicedb-zed` (or add to the repo `flake.nix`)
- **Linux/other**: download from https://github.com/authzed/zed/releases and place at `~/.local/bin/spicedb-zed`
- **Override**: `export SPICEDB_ZED_BIN=/path/to/spicedb/zed`
