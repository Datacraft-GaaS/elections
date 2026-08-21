# M14 reproducibility-bundle

| field | value |
|---|---|
| M-number | M14 |
| Home service | transparency-svc (ADR-0020) |
| Owning work package | WP-4.6 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M14-reproducibility-bundle.md |

## Purpose

Assemble and publish the statutory post-election reproducibility package: a self-contained, independently verifiable archive of all signed forms, participation tokens, Merkle proofs, source code, and build manifests, enabling any technically capable citizen, journalist, or party to independently re-derive and verify every published result figure.

## Surfaces and languages

| surface | technology | path under services/transparency-svc/ |
|---|---|---|
| Bundle CLI (offline use) | Rust (clap) | crates/bundle-cli/ |
| Bundle web portal | Astro on packages/eel-web-astro | astro/reproducibility-portal/ |
| Bundle API | Rust (axum + utoipa) | crates/transparency-api/ |
| Bundle assembly batch | Rust | crates/bundle-batch/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Bundle | bundle_id, election_id, components_manifest, merkle_root, assembled_at, sig, size_bytes | D | T1-audit |
| BundleComponent | comp_id, bundle_id, type (forms/tokens/src/build), ref, hash, size_bytes | D | T1-audit |
| ParticipationToken | token_sha256, station_code, issued_at, inclusion_proof_ref | D | T1-audit |
| BugBounty | bounty_id, reporter_hash, finding_summary, status, reward_level | C | T2 |

## APIs

OpenAPI file: packages/openapi-contracts/transparency/bundle.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /transparency/v1/bundles | 1 signed-static | public | n/a | List available bundles for an election |
| GET /transparency/v1/bundles/{id} | 1 signed-static | public | n/a | Bundle manifest + component list |
| GET /transparency/v1/bundles/{id}/download | 4 bulk | public | n/a | Signed bundle archive download |
| GET /transparency/v1/tokens/{sha}/proof | 2 live-per-user | public | n/a | Participation token inclusion proof |
| POST /transparency/v1/bug-bounty | 2 live-per-user | public | yes | Submit bug bounty / reproducibility finding |
| GET /transparency/v1/bug-bounty/{id} | 3 accredited | JWT+SpiceDB | n/a | Triage + status (IEBC internal) |
| POST /transparency/v1/bundles/assemble | 3 accredited | JWT+SpiceDB | yes | Trigger bundle assembly (post-election) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.transparency.bundle_assembled.v1 | emits | eel.transparency.bundle.v1 | operational | yes (HSM) | exactly-once |
| eel.transparency.bundle_published.v1 | emits | eel.transparency.bundle.v1 | operational | yes | exactly-once |
| eel.transparency.bug_bounty_submitted.v1 | emits | eel.transparency.bounty.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form_ingested.v1 | consumes | eel.rts.form.v1 | event-source | yes | exactly-once |
| eel.rts.merkle_root_anchored.v1 | consumes | eel.rts.merkle.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| RTS back-end M10 | internal | integrations/rts-backend/ | yes | form + token pull; retry |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on bundle sign |
| Transparency log (P05) | internal | integrations/tlog/ | yes | Merkle anchoring queued |
| KBC SFTP | external | integrations/kbc-sftp/ | yes | statutory broadcast mirror |
| Gazette (Gov Printer) | external | integrations/gazette/ | yes | notice publication |
| MinIO (platform) | internal | platform/minio/ | yes | large archive blob store |

## ML and AI

none

## Dependencies

- Modules: M08 (participation token source), M10 (form + Merkle source), M13 (reads token proof endpoint), M21 (RLA reads from bundle)
- Platform: P05 (audit), P06 (HSM), MinIO (blob storage)
- Packages: eel-crypto-rs, eel-web-astro

## Feature checklist

1. FR-M14-001: assemble post-election bundle: signed forms (34A/B/C), participation tokens, Merkle proofs, source code snapshot, build manifests, SBOM.
2. FR-M14-002: deterministic bundle assembly: same inputs always produce same archive (cross-arch reproducible build).
3. FR-M14-003: HSM-signed bundle manifest with component hashes; verifiable offline.
4. FR-M14-004: public download: bundle archive (tar.zst) with detached signature.
5. FR-M14-005: CLI verifier: citizen can independently verify bundle integrity offline.
6. FR-M14-006: participation token inclusion proof API: any citizen can verify their token in the Merkle tree.
7. FR-M14-007: statutory mirroring: bundle pushed to KBC SFTP + Government Gazette on publish.
8. FR-M14-008: bug bounty intake: any citizen can report discrepancies; triage workflow for IEBC.
9. FR-M14-009: partial bundles: constituency-level download (not just national).
10. FR-M14-010: bundle versioning: if correction issued post-publication, new version with delta manifest; old version immutable.
11. FR-M14-011: SBOM embedded in bundle; every build artifact traceable.
12. FR-M14-012: air-gapped verification guide: printed + digital instructions for offline verification.
13. FR-M14-013: cross-architecture determinism: bundle built on x86_64 and aarch64 produces same hash.
14. Bundle assembly <= 4 h for national election; constituency bundle <= 30 min.
15. Bundle download p95 <= 10 s (CDN-cached).
16. Token inclusion proof p95 <= 500 ms.
17. Bundle signature verifiable with standard OpenPGP / minisign tooling.
18. Statutory mirror push within 1 h of bundle publication.

## Acceptance criteria

AC-M14-01: bundle reproducible on clean machine by independent party (reproducibility audit).
AC-M14-02: participation token inclusion proof verified by reference client (integration test).
AC-M14-03: statutory mirror pushed to KBC + Gazette within 1 h (integration test).
AC-M14-04: CLI verifier passes on known-good and known-bad bundles (regression test).
AC-M14-05: cross-arch build produces identical hash (CI build test).
AC-M14-06: bug bounty intake + triage workflow functional (UAT).
AC-M14-07: 99.9% availability for token proof endpoint during petition window (SLO measurement).

## Applicable edge cases

**data**
EC-DAT-01: duplicate bundle assemble trigger; idempotent; same bundle_id.
EC-DAT-07: MinIO large object upload; multipart; resume on error.

**security**
EC-SEC-01: bundle signature mismatch; CLI rejects.
EC-SEC-03: HSM unavailable; bundle assembly paused; alert.
EC-SEC-12: CDN cache poisoning on bundle manifest; key includes sig version.
EC-SEC-16: secrets in build artifacts; SBOM scanner blocks on secret patterns.

**legal**
EC-TIM-08: petition window; bundle published before petition deadline T+7.
EC-LEG-04: correction bundle: new version with delta; old version immutable; chain of custody clear.

**operations**
EC-OPS-03: KBC SFTP unavailable; retry queue; alert after 30 min.
EC-OPS-04: Gazette unavailable; queued; escalated if > 1 h.

**scale**
EC-SCL-06: national bundle > 50 GB; MinIO multipart; streaming download.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | bundle manifest builder, hash chain, token proof serialiser, bug-bounty FSM | nextest | M14-UNIT- |
| property | same inputs -> same hash; all components in manifest accounted for | proptest | M14-PROP- |
| contract | M10 form pull (consumer); M13 token proof (provider) | Pact | M14-PACT- |
| integration | M10 double; SoftHSM sign; MinIO; KBC SFTP double; Gazette double | nextest + testcontainers | M14-INT- |
| e2e | assemble bundle -> CLI verify -> public download -> bug bounty submission | cargo test + Playwright | M14-E2E- |
| build | cross-arch reproducibility: build on x86_64 and aarch64; compare hashes | CI (GitHub Actions matrix) | M14-BUILD- |
| load | national bundle assembly under load; download at CDN edge | k6 | M14-LOAD- |
| security | SBOM secret scan; signature bypass; OWASP ZAP; Semgrep | Semgrep + OWASP | M14-SEC- |

## Runbooks and dashboards

- docs/runbooks/transparency-svc-bundle.md (create): bundle assembly failure; HSM fail; KBC SFTP timeout; correction bundle procedure.
- platform/observability/transparency-svc/grafana-bundle.json: bundle assembly progress, token proof latency, download rate, bug bounty queue depth.
- platform/observability/transparency-svc/slo-alerts.yaml: 99.9% burn rate; assembly > 4 h; KBC mirror > 1 h.

## Open questions

- Cross-arch determinism: Rust reproducible builds are well-supported but the Python ICR model artifacts (M10) need verification that they produce identical byte output on x86_64 and aarch64.
- KBC SFTP access: KBC has historically required manual coordination for SFTP credential rotation; formal SLA needed before G4.
