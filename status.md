# eel status report

Generated 2026-08-22 from repository inspection. Evidence is cited per claim.

## What this codebase is

Integrated elections management platform for Kenya's IEBC (tender IEBC/OIT/01/2026-2027, software prime Datacraft). 69 functional modules across 18 Kubernetes services covering the full electoral lifecycle: voter registration with biometrics, polling-day identification, results transmission from ~65k KIEMS kits, tally and declaration, public verification (10M concurrent on election night), transparency logging, chain of custody, governance, archive.

Scale envelope: 22M voters, ~46k polling stations, 30-50k events/sec sustained writes, 80-150k burst, 15-year audit retention, Kenya-sovereign hosting on 2 DR sites plus 5 regional edge clusters.

Naming rule enforced everywhere: `eel` prefix only, no client name in any identifier.

## Layout

- `services/`: the 18 services (bvr, abis, rts, bvi, biometric-edge, portal, crms, transparency, ecc, boundary, queue, inventory, governance, ai-platform, field-mobile, training, support-admin, vprs). Each holds Rust crates (`crates/*`), Python (`py/`, `django/`), Elixir (`ex/`), Flutter apps (`apps/`), Astro sites, migrations, deploy manifests, loadtest harnesses, README, TRACEABILITY.yaml, data_dictionary.yaml. Plus `hello-svc`, the reference/template service.
- `packages/`: 43 shared libraries mirrored in Rust/Python/Elixir: auth (JWT + SpiceDB), secrets (OpenBao), events (CloudEvents + HLC), outbox (sole Kafka publish path), eventstore, pii (five-tier typed wrappers), hlc, resilience, loadshedding, observability, crypto (Ed25519, BLAKE3, RFC 8785, Merkle, PKCS#11 pool, PQ hybrid, TSA, cosign), api-server (opinionated axum builder), verifier (RFC 6962 proofs as Dart FFI + WASM), kit-sdk-flutter, design-system, openapi-contracts, event-schemas, authz-schema, i18n (11 locales + pseudo), api-clients (generated on demand).
- `platform/`: cluster infra configs (cilium, kafka, postgres, keycloak, hsm-signer, argocd, observability stack).
- `integrations/`: adapters for external systems (NRB/IPRS, EACC, HSM PKCS#11, gazette, satellite, power-bi).
- `tools/`: synth, kit-simulator, completeness scanner, softhsm, spectral, asvs-check, pii-in-events, dr-failover.
- `verification/`: rehearsal, DR drill, chaos runner, 5x load, security harness, a11y-i18n sweep, completeness audit orchestrator.
- `docs/plans/modules/*.md`: full spec per module (69 pages). `docs/architecture/`: overview, trust model, ADR decision log. `docs/runbooks/`: one per service. Root TRACEABILITY.yaml maps requirements to tests.

## Architectural invariants

1. Services own their data; no cross-schema joins or shared writes. Cross-service state moves only via transactional outbox onto Kafka (Karapace-validated CloudEvents signed with HLC timestamps).
2. Only four bounded contexts are event-sourced (rts tally, custody); everything else is CRUD state plus an immutable audit shadow.
3. Every consensus-relevant artifact is signed and provable against a single monolithic Merkle log per election: domain-tagged leaves, HSM-signed STHs at 1s/60s/5min cadences, five publication channels (web, GitHub mirror, IPFS, Gazette print, KBC radio), IAB witness cosign day-1.
4. SPIFFE/SPIRE mTLS everywhere, SpiceDB permissions checked on every write path, OpenBao dynamic DB creds, PII tiers T0-T3 enforced by typed wrappers at field level.
5. Availability tiers: T1 election-critical 99.99% with sync DR, T2 cycle-critical 99.9%, T3 routine 99.5%.

## Stack and build

Rust nightly-first (T1 services, crypto), Python 3.12 (ML/ONNX sidecars, Django admin), Elixir 1.17 (LiveView dashboards), Dart/Flutter (kit firmware and mobile), TypeScript/Astro (portals). Postgres 17, Kafka/KRaft + Karapace, Valkey, MinIO, OpenSearch, Keycloak, SpiceDB, OpenBao, Cilium mesh, ArgoCD GitOps, Nix flake dev environments.

Root Cargo workspace covers `packages/eel-*-rs`, `verifier-rs`, `tools/*`, and service crates; several crates are excluded with inline reasons (some stale, see gap list). Service-level nested workspaces exist (e.g. rts-svc); the justfile iterates both. `just up` boots a kind cluster with the full data and identity plane; `just ci` = lint + test + contracts.

Testing layers: unit (nextest/pytest/ExUnit/flutter_test/vitest), property (proptest/hypothesis), contract (Pact, Karapace), fuzz (cargo-fuzz), formal (TLA+ on tally logic, kani on Merkle, creusot on HLC), load (k6/Gatling), chaos (Chaos Mesh), mutation testing, WCAG 2.2 AA sweeps.

## Current state

Build waves G0 through G6 delivered their harnesses and are dry-run green. G7 (ABIS world-class expansion, ~80 abis-svc crates plus reports subsystem) is ongoing. Gate reports live in `docs/audit/gate-G*.md`; gate-G6 was re-baselined 2026-08-22 to state honest verified coverage.

Recent commits show hardening work replacing stub/passthrough implementations with real ones and binding tests to requirement IDs (`eel-trace-rs` proc-macro now exists and compiles).

Uncommitted work on main (in flight): modified Flutter app `services/bvr-svc/apps/cvr-field-officer/` including new `test/integration/`, two brand-new tool crates `tools/softhsm/` and `tools/spectral/` (untracked), Cargo.lock churn.

---

## Gap analysis: 20 incomplete or missing items

Method: ran `eel-completeness scan` (389 non-allowlisted findings), `eel-completeness trace`, read gate reports, Cargo.toml excludes, and inspected each flagged site. Items are ordered roughly by risk.

### Test and verification debt

1. **Requirement-to-test coverage is 25 of 4,444 rows.** `eel-completeness trace` against `docs/plans/TRACEABILITY.yaml` reports zero passing tests for essentially every module: FR-M01 through FR-M69 and most EC-* edge-case rows are unmapped (confirmed in `docs/audit/gate-G6.md` re-baseline section, which puts the true gap count at 4,419). The platform's central quality gate (plan section 9.3: any row with zero mapped tests fails) currently fails wholesale. Closing this needs tests written or existing tests tagged with `#[trace(...)]` across all services.

2. **Integration tests are skipped, not passing.** `services/rts-svc/tests/integration/mod.rs:4` and `services/rts-svc/crates/rts-integration-tests/tests/integration/mod.rs:4` carry module-level `#[ignore]`; `services/biometric-edge-svc/tests/integration/kit_simulator.rs:14,46` has two ignored tests; `services/abis-svc/tests/common/mod.rs:74` another. The election-night critical path's integration suites do not run.

3. **All Wave-6 verification was dry-run only.** Per `docs/audit/gate-G6.md`: election-day rehearsal at scale, live DR failover drill, chaos-during-load, measured 5x load run, 24h fuzz plus DAST, and live accessibility/screen-reader sweeps were never executed because the pre-prod infrastructure does not exist. Harnesses assert 140/140 dry conditions only.

4. **The completeness audit pipeline runs on a fixture.** Step 9.x orchestration passed on a mini-audit fixture; live ingestion of the full multi-language JUnit matrix (step 9.7 SLA measurement, 9.4 contract closure against real Pact broker) is not wired.

### Stubs in load-bearing code

5. **Cosign bundle verification does not verify signatures.** `packages/eel-crypto-rs/src/cosign.rs:56-81`: `verify_bundle` parses JSON and base64-decodes the signature, then returns Ok without any cryptographic check (TODO pending sigstore 0.10 API stabilization). Any caller trusting `verify_bundle` today accepts forged bundles.

6. **RTS tally never anchors to the transparency log.** `services/rts-svc/crates/rts-tally/src/tlog_client.rs:13-17`: `append_leaf` logs the endpoint and returns `Ok(())`. The gRPC call to transparency-svc AppendLeaf is unwired, so snapshot anchoring on this path is a silent no-op success.

7. **Voter card blacklist is a no-op write path.** `services/bvr-svc/crates/voter-card/src/blacklist.rs:15-20`: `add_entry` logs and returns Ok; no DB write, no Kafka publish, so blacklisted cards would remain valid everywhere downstream.

8. **Transparency-log admin and witness paths have TODO stubs.** `services/transparency-svc/crates/tlog-admin/src/export.rs:6`, `tlog-admin/src/witness.rs:10`, `tlog-witness/src/iab_client.rs:39`. The IAB witness cosign flow (a day-1 trust requirement) is partially implemented.

9. **Python components ship bare `pass` bodies in real logic.** Certificate revocation list handling in `services/training-svc/django/accreditation/accreditation_app/crl.py:23,35`; FaceForensics loader `services/ai-platform-svc/py/adversarial-training/src/eel_adversarial_training/faceforensics.py:15`; GNN model `.../cross-platform-cib/src/eel_cross_platform_cib/gnn.py:129`; RFID adapters `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/cipherlab_rs36.py:205` and `llrp_common.py:140`; KBC SFTP deliverer `services/abis-svc/py/report-deliverers/src/eel_report_deliverers/kbc_sftp.py:26`; ad-hoc report authoring view `services/abis-svc/django/abis-adhoc-editor/adhoc_editor/apps/authoring/views.py:92`.

10. **Kit and mobile apps carry unfinished feature code.** 12 TODOs across the uncommitted `services/bvr-svc/apps/cvr-field-officer/` screens and services (capture, review, history, offline_queue, attestation); RFID service stubs in `services/inventory-svc/apps/custody-mobile/lib/src/services/rfid_reader_service.dart:147,150`; mode switch and signing TODOs in `services/rts-svc/apps/rts-kit/lib/src/rts_workflow/mode_switch.dart:55,90` and `sign.dart:92`; `services/biometric-edge-svc/apps/bvi-kit/lib/src/mode_switch.dart:68,100`; party-agent auth provider `services/field-mobile-svc/apps/party-agent/lib/src/features/auth/auth_provider.dart:32`.

### Build and workspace integrity

11. **The inventory custody domain cannot compile anywhere.** `services/inventory-svc/crates/custody/Cargo.toml:19` depends on `uuid6 = { workspace = true }`, a crate that does not exist on crates.io; the root workspace carries it as a phantom reference and `Cargo.lock` has no resolved entry. The crate is excluded from the root workspace, so the event-sourced chain-of-custody domain (M20) has no CI signal at all.

12. **vprs-api is excluded for missing sqlx offline cache.** No `.sqlx` directory exists under `services/vprs-svc/crates/vprs-api/`, so `sqlx::query!` macros fail without a live DATABASE_URL; the whole vprs API surface is outside the root build.

13. **mobile-bff is excluded for an unresolvable dependency.** `axum-extra` is referenced by `services/field-mobile-svc/crates/mobile-bff` but absent from root `[workspace.dependencies]`, so the field-mobile BFF (M13/M29-M34 backend) does not build in CI.

14. **Stale exclusions hide buildable crates from root CI.** `rts-device-api` and `rts-ro-api` cite missing ciborium/zstd/eel-api-server-rs/eel-outbox-rs, all of which now exist (and the rts-svc nested workspace builds them); `edge-api` cites kit-attest/reqwest, both now in workspace deps; `tools/dr-failover` cites a missing Cargo.toml that now exists with its own lockfile. Consequence: these crates get no `cargo nextest`/clippy/coverage from the root toolchain. New untracked tool crates `tools/softhsm/` and `tools/spectral/` are likewise excluded.

### Contract-first surfaces without implementations

15. **287 OpenAPI paths have no bound handler.** Scan kind `openapi_unbound_path`: paths in `packages/openapi-contracts/**` lack `x-handler-bound` annotations. Heaviest: field-mobile-bff (18 paths), bvi (10), support-special-voter (9), custody (9), boundary-transparency (8), support-intent (8), abis (7), crms-nomination (7), training-lms (7), portal-public (7), and about 30 more files. Gate-G6 states bindings are added only where real handlers exist, so these represent specified-but-unimplemented endpoints.

16. **Elixir oversight dashboards are partially wired.** `services/ecc-svc/ex/ecc_live/lib/ecc_live/live/dashboard_live.ex:9`, `alerts_live.ex:9,16`, `kits_live.ex:9`, `s44a_live.ex:20,31`, and `services/bvi-svc/ex/bvv_web/lib/bvv_web/live/objections_live.ex:12`, `lookup_live.ex:35` all carry TODOs in mount/render paths. The command centre and public verification LiveViews render with incomplete data plumbing.

17. **Governance and boundary domain logic is partial.** `services/governance-svc/crates/by-election/src/register_subset.rs:20` and `kit_plan.rs:17,47`; `services/boundary-svc/crates/boundary-worker/src/fairness.rs:14` and `shapefile.rs:12`. By-election register subsetting, kit planning, district fairness metrics, and shapefile ingest are unfinished.

18. **eel-auth-py audit and JWKS modules contain placeholder bodies.** `packages/eel-auth-py/src/eel_auth/jwks.py:15` and `audit.py:23` flagged as bare `pass`. The Python auth mirror is consumed by every FastAPI service, so these need review (may be benign hooks, but the scanner treats them as stubs).

### Process debt

19. **Service template ships with TODO markers.** `packages/service-template-py/{{cookiecutter.svc_name}}/py/{{cookiecutter.svc_name}_api}/eel_server.py:13,14`. Every new Python service scaffolded from the template inherits stub markers that the completeness scanner will flag.

20. **In-flight work is unlanded on main.** The modified cvr-field-officer Flutter app (screens, models, api client, l10n, widget/offline-queue tests, new integration tests) plus untracked `tools/softhsm/` and `tools/spectral/` sit uncommitted alongside regenerated `Cargo.lock` and traceability artifacts. Until committed, the workspace is not reproducible from HEAD and the two new tool crates are invisible to other developers.

## Suggested priority

Security first: items 5, 6, 7, 8 (silent-success stubs on signed/anchored paths). Then build integrity: 11, 12, 13, 14 (crates invisible to CI). Then the coverage program: 1, 2 (the traceability gap is the plan's own blocking gate). The rest follows.
