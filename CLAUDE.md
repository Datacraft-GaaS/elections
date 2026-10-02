# CLAUDE.md

Accept the rules in AGENTS.md — dependency, compilation, and audit chain. Key
enforcement points:

- Lockfile: `Cargo.lock` committed; `--locked` / `--frozen` builds.
- Updates: `cargo update -p` one crate, precise version, ≥30 days old.
- Compiler: `rust-toolchain.toml` pinned to an exact version.
- Security: `cargo audit` + `cargo deny check` before commit.
- Audit chain: `auditId` (UUID7), `proofHash`, `signature` preserved in all docs.
- Naming: `eel` prefix only, in every identifier, package, and event name.