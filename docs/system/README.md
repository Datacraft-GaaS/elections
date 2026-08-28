# System Reference (`docs/system/`)

Complete, implementation-referenced documentation of the eel election platform: architecture, functions, capabilities, operation, safeguards, and security. Every behavioural claim is cited to the file (and where useful, line) that implements it. Where older prose docs and the code disagree, these volumes follow the code and flag the discrepancy.

## Volumes

| File | Covers |
|---|---|
| [architecture.md](architecture.md) | System shape: topology, runtime tiers, normative rules with ADR + code anchors, data infrastructure, event flow, cross-cutting mechanisms |
| [anti-rigging.md](anti-rigging.md) | Election-integrity controls: threat-by-threat map of dedup, signing, tally integrity, transparency log, audits, custody and kit trust — each graded IMPLEMENTED/PARTIAL/STUB with code citations |
| [anti-rigging-opportunities.md](anti-rigging-opportunities.md) | Ten further integrity controls + honourable mentions — all implemented and test-verified: threat, implementation citations, deployment prerequisites |
| [anti-rigging-next.md](anti-rigging-next.md) | Wave-2 controls — all implemented, offline-first: commit–reveal forms, register anchoring, canary voters, audit-chain ledger, recording continuity, threshold encryption, integrity statements, coverage certificates, adaptive audits, signed verifiers + 4 honourable mentions |
| [security.md](security.md) | Identity & authn (SPIFFE/Keycloak), authorization (SpiceDB/tiers), cryptography & HSM, data protection & PII, edge defences, integrity/provenance, enumeration & timing defences, compliance hooks, threat model, known weaknesses |
| [operations.md](operations.md) | Build systems, test strategy, verification suites, deployment topology, configuration conventions, observability ops, degradation modes, disaster recovery, CI gates |
| [packages/shared-packages.md](packages/shared-packages.md) | All ~30 shared libraries under `packages/`: public APIs, consumers, security behaviours, per-language comparison |
| [tools-reference.md](tools-reference.md) | Everything under `tools/`: completeness scanner, HSM ceremony tooling, IAM seeding, ASVS/WCAG audits, PII-in-events checker, DR failover, synth, and more |

## Service references (`services/`)

One page per service in `services/`, each following the same eight-section template: Overview, Ownership & Boundaries, Repository Layout, Capabilities & Functions (every route/gRPC method/LiveView/worker enumerated from the routers), Data Model (every migration table, every topic), Operation (build/run/test/env vars), Safeguards & Security (per-safeguard citations plus tests), Observability.

| Service | Doc | Runtime |
|---|---|---|
| abis-svc | [abis.md](services/abis.md) | Rust + Python vendor adapters |
| ai-platform-svc | [ai-platform.md](services/ai-platform.md) | Rust + Python ML |
| biometric-edge-svc | [biometric-edge.md](services/biometric-edge.md) | Rust |
| boundary-svc | [boundary.md](services/boundary.md) | Rust (PostGIS boundaries) |
| bvi-svc | [bvi.md](services/bvi.md) | Rust + Astro portal + LiveView console |
| bvr-svc | *(see docs/architecture/services/bvr.md; excluded from workspace build)* | Rust |
| crms-svc | [crms.md](services/crms.md) | Rust |
| ecc-svc | [ecc.md](services/ecc.md) | Rust + Phoenix LiveView |
| field-mobile-svc | [field-mobile.md](services/field-mobile.md) | Python + Flutter |
| governance-svc | [governance.md](services/governance.md) | Rust + Django |
| hello-svc | [hello.md](services/hello.md) | template service |
| inventory-svc | [inventory.md](services/inventory.md) | Rust + Django |
| portal-svc | [portal.md](services/portal.md) | Rust |
| queue-svc | [queue.md](services/queue.md) | Rust + LiveView |
| rts-svc | [rts.md](services/rts.md) | Rust + Python kits/ICR |
| support-admin-svc | [support-admin.md](services/support-admin.md) | Rust |
| training-svc | [training.md](services/training.md) | Rust |
| transparency-svc | [transparency.md](services/transparency.md) | Rust + Django admin + Astro |
| vprs-svc | [vprs.md](services/vprs.md) | Rust |

## Method

These volumes were produced by reading each router, migration set, store trait, and worker entrypoint in the repository and citing them directly. They are designed to be re-verifiable: follow any citation to check a claim.

## Known gaps between prose docs and code

Findings flagged during documentation (each verified in code; detailed per page):

- Several READMEs describe endpoints/ports/layouts that differ from the shipped code (notably `services/rts-svc`, `services/biometric-edge-svc`, `services/hello-svc`, `services/transparency-svc`).
- Some TRACEABILITY.yaml files reference implementation paths that do not exist (notably `services/bvi-svc`); `services/ecc-svc`, `services/vprs-svc`, and `services/field-mobile-svc` have none.
- A number of capabilities documented in CAPABILITIES.md files are aspirational rather than implemented (e.g. vprs IPRS gRPC integration, field-mobile Kafka write-behind).
- Placeholder/stub statuses that remain by design or pending work packages: transparency-log STH signatures pending WP-2.2 (`services/transparency-svc/crates/tlog-sequencer/src/sth_producer.rs`), ABIS auth middleware unwired, several admin handlers returning fixed JSON (ecc, support-admin, portal snapshot routes).
- Security weaknesses tracked for remediation are listed in [security.md §Known weaknesses](security.md).

## Relationship to other docs

- `docs/architecture/` — earlier system-wide design prose and decision log (kept for context; superseded where it conflicts with code).
- `docs/adr/` — numbered architecture decisions referenced throughout this set.
- `docs/runbooks/`, `docs/guides/` — operational how-tos; `operations.md` indexes them.
