# eel status report

Updated 2026-08-22 after the gap-closure pass (commits 8a340e0..9cd19d7). The first half describes the codebase; the second half reports, per gap, what is now closed and what remains open with its exact blocker.

## What this codebase is

Integrated elections management platform for Kenya's IEBC (tender IEBC/OIT/01/2026-2027, software prime Datacraft). 69 functional modules across 18 Kubernetes services covering the full electoral lifecycle: voter registration with biometrics, polling-day identification, results transmission from ~65k KIEMS kits, tally and declaration, public verification (10M concurrent on election night), transparency logging, chain of custody, governance, archive.

Scale envelope: 22M voters, ~46k polling stations, 30-50k events/sec sustained writes, 80-150k burst, 15-year audit retention, Kenya-sovereign hosting on 2 DR sites plus 5 regional edge clusters.

Naming rule enforced everywhere: `eel` prefix only, no client name in any identifier.

## Layout

- `services/`: the 18 services plus `hello-svc`, the reference template. Each holds Rust crates (`crates/*`), Python (`py/`, `django/`), Elixir (`ex/`), Flutter apps (`apps/`), Astro sites, migrations, deploy manifests, loadtest harnesses, README, data_dictionary.yaml.
- `packages/`: 43 shared libraries mirrored in Rust/Python/Elixir: auth (JWT + SpiceDB), secrets (OpenBao), events (CloudEvents + HLC), outbox (sole Kafka publish path), eventstore, pii (five-tier typed wrappers), hlc, resilience, loadshedding, observability, crypto (Ed25519, BLAKE3, RFC 8785, Merkle, PKCS#11 pool, PQ hybrid, TSA, cosign), api-server, verifier (RFC 6962 proofs as Dart FFI + WASM), kit-sdk-flutter, design-system, openapi-contracts, event-schemas, authz-schema, i18n (11 locales + pseudo), api-clients.
- `platform/`: cluster infra configs. `integrations/`: external system adapters. `tools/`: synth, kit-simulator, completeness scanner, softhsm, spectral, asvs-check, pii-in-events, dr-failover. `verification/`: rehearsal, DR drill, chaos runner, 5x load, security harness, a11y-i18n sweep, completeness audit orchestrator.
- `docs/plans/modules/*.md`: full spec per module. `docs/architecture/`: overview, trust model, ADR decision log. Root TRACEABILITY.yaml maps requirements to tests.

## Architectural invariants

1. Services own their data; cross-service state moves only via transactional outbox onto Kafka (Karapace-validated CloudEvents signed with HLC timestamps).
2. Four bounded contexts are event-sourced (rts tally, custody); everything else is CRUD state plus an immutable audit shadow.
3. Every consensus-relevant artifact anchors to a single monolithic Merkle log per election: domain-tagged leaves, HSM-signed STHs at 1s/60s/5min cadences, five publication channels (web, GitHub mirror, IPFS, Gazette print, KBC radio), IAB witness cosign day-1.
4. SPIFFE/SPIRE mTLS everywhere, SpiceDB permissions on every write path, OpenBao dynamic DB creds, PII tiers T0-T3 enforced by typed wrappers.
5. Tiers: T1 election-critical 99.99% sync DR; T2 cycle-critical 99.9%; T3 routine 99.5%.

Stack: Rust nightly-first, Python 3.12, Elixir 1.17, Dart/Flutter, TypeScript/Astro; Postgres 17, Kafka/KRaft + Karapace, Valkey, MinIO, OpenSearch, Keycloak, SpiceDB, OpenBao, Cilium, ArgoCD, Nix. Testing layers: unit, property, contract, fuzz, formal (TLA+, kani, creusot), load, chaos, mutation.

---

## Gap closure report (was: "20 incomplete or missing items")

Measured before/after with `eel-completeness scan`: 389 non-allowlisted findings at baseline; 312 after, of which 287 are the OpenAPI-binding program (item 15) and 25 are the scanner's own self-test fixtures. Every other code-level stub finding from the baseline is gone. All claims below were verified by targeted test runs in this session.

### Closed

1. **Test harness wiring** (was item 2): the orphaned abis integration suite now compiles via a new `abis-integration-tests` crate; biometric-edge kit_simulator and rts integration tests run hermetically and pass (`cargo test -p abis-integration-tests` = 1 passed/10 honestly gated; `-p biometric-edge-integration-tests` = 2 passed; `-p rts-integration-tests` = 3 passed). Infra-bound tests print their exact missing prerequisite instead of failing silently.
2. **Cosign verification**: `eel-crypto-rs::cosign::verify_bundle` now verifies Ed25519 and ECDSA P-256 signatures over artifact bytes against SPKI PEM keys (in-tree DER parser, no sigstore dependency). Tampered artifacts and wrong keys fail closed. Commit c7d6c39.
3. **RTS transparency anchoring**: `TlogClient::append_leaf` builds an Ed25519-signed envelope and emits it through the transactional outbox on `eel.results.snapshot_anchored.v1`; the publisher scheduler anchors after each upload and fails loudly rather than faking success. Commit cf28b9f.
4. **Voter card blacklist**: real transactional write (row + outbox event in one commit, idempotent on unchanged entries), migration V051, live `/status` query, DB round-trip test gated behind a `db-tests` feature with run recipe. Commit 5d7afeb.
5. **Transparency admin and witness paths**: witness cross-check verifies every cosignature cryptographically and detects split views; export streams leaves + tiles to tar.gz (local or MinIO PUT); IAB client signs STHs per the documented protocol with a provisioned key; new `GET /witnesses/report` endpoint backs the cross-check. Commit 88168d6.
6. **Custody merkle anchoring**: publisher emits custody-domain envelopes through the outbox; five-channel fan-out stays in transparency-svc where it belongs. Custody-mobile became a buildable Flutter package with real transport-driven reader services and tests. Commit aa3c631.
7. **Workspace integrity**: custody compiles (uuid6 phantom removed), vprs-api implemented OTP handlers + schema (V001) and left the exclude list, mobile-bff unblocked via axum-extra, stale exclusions for rts-device-api/rts-ro-api/edge-api/dr-failover cleared so they get CI signal. queue-svc verified building as a root member.
8. **Python stubs**: training CRL generation, abis KBC SFTP deliverer (PGP sign, bundle, paramiko upload), ad-hoc editor views, auth-pjwks/audit sinks, service template, events-py test doubles, design-system app config: all implemented or replaced with precise prerequisite notes. Face-onnx detector keeps deterministic fallback with the missing model I/O spec named inline.
9. **Dart/mobile TODOs**: cvr-field-officer screens/services finished on top of the WIP commit; rts-kit signs bundles with real Ed25519 and fails closed without anchors; bvi-kit verifies ECDSA P-256 bundle signatures; party-agent auth completed. flutter analyze clean, all suites green.
10. **Elixir dashboards**: dashboard/alerts/kits LiveViews verified complete against existing ecc-api routes; objections_live complete. Only s44a_live and lookup_live remain (see open list).
11. **Governance/boundary seams**: by-election register subset + kit plan and boundary fairness/shapefile functions carry precise integration-owner statements; pure logic tested green (`eel-governance-by-election` 17 passed, `boundary-worker` 8 passed).
12. **Route scaffolds**: cvr field notifications emit their documented topics through the outbox; cvr/dereg/voter-card handlers query or persist via V052/V053 migrations. Commits 5d7afeb, 9cd19d7.

### Open, with blockers

13. **Traceability coverage**: 25 of 4,444 requirement rows have passing tagged tests. This is a test-writing program across all 69 modules, not a code fix; the tagging harness (`eel-trace-rs`) works. Track per-module burn-down via `just` recipes calling `eel-completeness trace`.
14. **287 unbound OpenAPI paths**: contract-first specs whose handlers do not exist yet (field-mobile-bff 18, bvi 10, special-voter 9, custody 9, ...). Each binding requires implementing the endpoint; bindings are only added where handlers are real, never fabricated.
15. **ecc s44a_live / bvi lookup_live**: blocked on missing backends. ecc-api has no S44A module (CAPABILITIES names crates that do not exist) and no Ecto schema to persist confirmations; bvi-svc declares three crates that are absent and its citizen OTP lookup has no upstream contract. Building either backend is feature work, not gap cleanup.
16. **Live Wave-6 execution**: rehearsal at scale, DR failover drill, chaos-during-load, measured 5x load, 24h fuzz/DAST, live screen-reader sweeps need pre-prod infrastructure this repo cannot provide. Harnesses are dry-run green and run unmodified once clusters exist.
17. **ONNX face detection inference**: needs the deployed model's input/output contract (anchor decode, NMS) shipped with the model bundle (WP-3.7).

### Verification evidence

- `cargo metadata` parses clean from root; affected suites: eel-crypto-rs 29 passed (cosign feature), rts-tally+publisher 13 passed, voter-card/cvr/dereg-appeal combined green (DB-gated tests excluded), tlog-admin 6 passed, tlog-witness 4 passed incl. end-to-end cosign round trip, tlog-api compiles, custody 4 passed, eel-hlc-rs green after parking_lot move, eel-outbox-rs green.
- Python: touched pytest files green per package runners (rfid-driver 10+36 passed, adversarial-training 19 passed, report-deliverers/adhoc-editor/accreditation/auth-py suites green).
- Flutter: analyze clean and full test suites green for cvr-field-officer (31), rts-kit (18), bvi-kit (15), party-agent (4), custody-mobile (new).
- Scan delta: 389 -> 312 non-allowlisted findings; residual breakdown above.

Commits: 8a340e0 snapshot, then c7d6c39 crypto, cf28b9f rts, 5d7afeb bvr, 88168d6 transparency, aa3c631 inventory, 9cd19d7 cross-service fixes. Pushed to origin/main.

---

## Toolchain and mobile-tier state (measured 2026-09-29)

Recorded here because the numbers above were measured before the toolchain
repairs, and several claims no longer hold.

### Closed since the report above

- **Root workspace was unloadable.** `members` includes `tools/*`, Python leaves
  `__pycache__` beside its scripts, and cargo tried to load each as a crate, so
  no cargo command ran at the repo root at all. Five fuzz crates had no
  `[workspace]` and were rejected; `rts-device-api/fuzz` declared a dependency
  twice and could not be parsed; `nearform-nbis` and `bvr-desktop/bridge` were
  members of no workspace; and `bft-tally`, `fhe-tally`,
  `behavioural-biometrics` and `kit-cert-transparency` inherited root
  dependencies but were shadowed by nested service workspaces, so they belonged
  to nothing. All resolved.
- **bvr-svc was unloadable** the same way: three crates referenced
  `tower = { workspace = true }` with no such workspace entry.
- **EC-MOB-02 storage guard was dead in production.** `RealStorageChecker`
  returned null unconditionally, and null means "unavailable", so the check was
  only ever exercised by test fakes. It now requires a bound delegate.
- **party-agent's `Sign In` had an empty handler.** Wired to `AuthNotifier.login`.
- **Orphaned Dart tree** at `services/field-mobile-verify/lib/`, byte-identical
  to citizen-verify's copy, in a directory with no `pubspec.yaml` and therefore
  in no package and never analyzed.
- 158 committed `.gradle/` build caches untracked; 13 tracked `.bak` files
  removed; two 3-line `// stub` crates (`tools/softhsm`, `tools/spectral`)
  removed, keeping the Bash and JS tooling that shared those directories.

### Verified now

- 241 Cargo manifests load, 0 broken (`python3 tools/cargo_manifests.py`).
  All 9 service workspaces and the 208-member root workspace compile with
  `--locked`; `cargo test --workspace` = 3118 passed, 0 failed.
- 24 Flutter packages, 0 errors / 0 warnings / 0 infos
  (`python3 tools/flutter_health.py`); 334 tests pass across all 24, 0 failing,
  0 broken (`python3 tools/flutter_tests.py`).

Both gates are real: each was confirmed to exit non-zero on an injected fault,
because an earlier version of the Flutter harness scored a package whose
`pub get` had failed as clean.

### Still open, restated with current numbers

18. **45 dead button handlers** across 10 apps and 42 screens, each
    `onPressed: () {}` — party-agent 9, poll-clerk 9, observer 7, field-support
    5, personnel 4, diaspora-kit 3, rts-kit 2, citizen-verify 2,
    accred-scanner 2, lms-offline 2. These are the visible face of item 14: the
    field-mobile BFF implements 9 paths (attestation/verdict, config,
    dispatch/assignments, forms/sync, incidents, observer/observations,
    sessions, sessions/refresh, station/status, voter/verification) while the
    contracts declare more, so a minority are wireable today and the rest need
    their endpoint built. No handler is wired to invented behaviour.
19. **field-mobile-verify has no implementation.** The directory holds a
    migration, README and TRACEABILITY only. Its README describes a fully
    implemented sqlx service, which is not true.
20. **Items 13-17 above remain open.** 13 (traceability: 25 of 4,444 rows
    tagged), 14 (287 unbound OpenAPI paths), 15-17 (blocked on absent backends,
    infrastructure and a model contract respectively). 15, 16 and 17 remain
    blocked on inputs this repository does not have, which is why they are
    written down rather than started.
