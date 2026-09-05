# EEL Agent Rules (derived from AGENTS.md — dependency + compilation + audit chain)

Accept the rules in AGENTS.md (§ Dependency & Compilation Rules). Key enforcement points:
- Lockfile: `Cargo.lock` committed, `--locked` / `--frozen` builds.
- Updates: `cargo update -p` one crate, precise version, ≥30 days old.
- Compiler: `rust-toolchain.toml` pinned exact.
- Security: `cargo audit` + `cargo deny check` before commit.
- Services: all 20 compile clean (see AGENTS.md status table); `field-mobile-svc` errors pre-existing.
- Audit chain: `auditId` (UUID7), `proofHash`, `signature` (Ed25519/HSM) preserved in all docs.
- M0→M5 sequential (§11.0) maintained; M4 after M2/M3; M5 after M4.
