# ADR 0006: Build & supply chain

Status: accepted
Date: 2026-08-21

## Context

`sw/expect.md:39` requires reproducible builds anchored in air-gapped Nix or Bazel. `sw/ARCHITECTURE.md` requires cosign admission, SLSA-grade attestations, and 30-day pre-election freeze. Every dependency must be mirrorable to an internal registry.

## Decision

- **Reproducible builds**: **Nix flakes** for every service, every image. Bit-identical rebuilds for the M14 `make reproduce` deliverable.
- **Language build tools**: `cargo` (Rust) + `uv` (Python) + `pnpm` (TS) + `mix` (Elixir) + `flutter` (Dart), orchestrated by root `just` recipes; each build produces a Nix-hashed artifact.
- **Monorepo orchestration**: Nix + `just`; per-language build graphs stay native (cargo workspaces, pnpm workspaces, uv workspaces, mix umbrella). No Bazel.
- **Forge**: **Gitea** or **Forgejo** (both MIT/GPL). Self-hosted, on-prem, mirrorable.
- **CI**: **Woodpecker CI** (Apache 2.0). Runners in K8s. GitLab CE rejected as heavier than needed.
- **Air-gapped dependency mirror**: **Sonatype Nexus OSS** mirroring PyPI, npm, cargo registry, Maven, apt, OCI, hex. All builds pull from Nexus only; no direct upstream fetch on the build path.
- **Container registry**: **Harbor** with cosign verification (see ADR-0004).
- **Artifact signing**: **cosign** with **Sigstore** for provenance. Keyless signing via SPIFFE workload identity for CI-built artifacts; keyed signing for release artifacts using HSM-anchored key.
- **Supply-chain attestations**: **SLSA L3**, provenance, isolated builds, reproducible, non-falsifiable. **in-toto** attestations across build stages.
- **Dependency review**: `cargo-vet` + `cargo-deny` + `cargo-audit` (Rust); `pip-audit` + `pip-licenses` (Python); `pnpm audit` + `npm-audit-resolver` (TS); `mix deps.audit` (Elixir). Trust base shared across teams.
- **Vulnerability scanning**: **Trivy** + **Grype** + **Syft** on every image + every dep manifest, gating deploy.
- **Renovate** for dependency PRs, running in a segregated environment (not on the production build path); PRs reviewed by humans, tested in staging, merged manually.
- **30-day pre-election freeze**: enforced by a CI gate that refuses non-critical merges within the freeze window; critical security fixes require dual sign-off + tabletop discussion.

## Consequences

Positive: bit-identical rebuilds satisfy M14 reproducibility bundle without extra tooling. Cosign + SLSA L3 gives auditors a verifiable chain from source to running container. Air-gapped mirror removes upstream availability as a build-path failure mode.

Negative: Nix has a steep learning curve; mitigated by a shared `flake.nix` template per language. Nexus mirror is a stateful component to keep in sync; automated nightly sync with lag alerts. Reproducibility discipline requires locking every input (compiler toolchain, base image, transitive deps), real ongoing cost but non-negotiable.

## Alternatives rejected

- Bazel: heavier than Nix for our shape; the incremental build wins are eaten by ecosystem friction across Rust/Python/Elixir/Dart.
- GitLab CE: capable one-stop shop but heavier than Gitea + Woodpecker; less operational familiarity.
- GitHub Actions: not usable in air-gapped path.
- JFrog Artifactory OSS: usable alternative to Nexus; Nexus picked for team familiarity.

## Related

ADR-0002 (build tools per language), ADR-0004 (Harbor + Kyverno enforce cosign), ADR-0005 (HSM-anchored release signing).
