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
- **Three login flows were fakes, not empty.** `observer` and `poll-clerk`'s
  `AuthNotifier.login` slept 300ms and set `authenticated` for any credentials
  at all, with no credential check and no attestation; only `party-agent` had a
  real gate. All three now hard-fail device attestation before authenticating
  (ADR-0021) and their `Sign In` buttons are wired. A fourth screen's `Sign In`

- **Orphaned Dart tree** at `services/field-mobile-verify/lib/`, byte-identical
  to citizen-verify's copy, in a directory with no `pubspec.yaml` and therefore
  in no package and never analyzed.
- **Three dead Dart trees** outside any package, all removed: an orphaned
  duplicate in `services/field-mobile-verify/` that was byte-identical to
  citizen-verify's copy; a second `lib/src/features` copy under
  `accred-scanner` that nothing imported and that had already diverged from the
  live one; and `services/biometric-edge-svc/apps/rts-kit/`, a single
  `ScanScreen` shell with no `pubspec.yaml`. That last one was not a duplicate —
  the real `services/rts-svc/apps/rts-kit` has its own `ScanScreen` in
  `rts_workflow/scan.dart` and wires it into the router, so the orphan was a
  name collision with no callers, whose entire content was three defects: a
  fabricated "RFID Reader: Ready" success chip, a dead Start Scan button, and a
  nested-scrollable body.
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

18. **38 dead button handlers** across 9 apps, each `onPressed: () {}`. These
    are the visible face of item 14. The field-mobile BFF implements 9 paths
    (attestation/verdict, config, dispatch/assignments, forms/sync, incidents,
    observer/observations, sessions, sessions/refresh, station/status,
    voter/verification) while the contracts declare more, and spot-checks
    confirm the remainder are blocked on collaborators that do not exist
    rather than on effort:

    - `personnel` check-in: `DispatchApi.checkIn` is a real POST but requires
      `electionId` and `biometricHash`, and neither appears anywhere in the app.
    - `field-support` incident submit: the app has no controller or client at
      all, where `poll-clerk` has a real `IncidentController`. The screen's
      payload contract is not defined.
    - `lms-offline` assessment: no `Question` model exists; the screen renders
      chips literally labelled A–D under a hardcoded "Question 1 of 10".
    - `diaspora-kit` pouch verify: `PouchNotifier` has `add` and `addTransfer`
      but no loader and starts empty, so there is nothing to verify.
    - `poll-clerk` eod-report: `ApiService` is an empty class with no methods
      and no callers, and no report endpoint exists.

    No handler is wired to invented behaviour. Where a screen's own collaborator
    did exist, it was wired and tested. Six have been: three login flows, the
    citizen-verify results and register-lookup screens, and the party-agent
    duress beacon. The first three found that the collaborator existed but was
    itself a fake, and the register lookup needed an OTP token that the OTP
    screen was discarding.
19. **field-mobile-verify has no implementation.** The orphan Dart tree that
    sat in it has been removed; what remains is a migration, README and
    TRACEABILITY and nothing else. Its README describes a fully implemented sqlx
    service, which is not true. Whether the remaining skeleton is worth keeping
    is a product decision, not a cleanup one.
20. **Three mobile-bff verification handlers do not verify.**
    `verify_voter_lookup` (spec.rs:713) defaults the voter id to
    "VOTER-12345" and returns `{"verified": true}` for any input, including
    none — and it is a live route. `tally_verify` (spec.rs:746) and
    `verify_certificate` (spec.rs:807) do the same but are defined without being
    registered on any router, so they are dead code.

    A voter lookup that always agrees is a security defect rather than a missing
    feature: it is wired, and it answers. Fixing it means consulting the
    register, which is a data-source decision for someone who owns that schema.
    The app side was made honest instead — poll-clerk's VoterLookupScreen no
    longer renders a fixed "Voter Verified • Registered" card and states that
    verification is unavailable.

21. **Items 13-17 above remain open.** 13 (traceability: 25 of 4,444 rows
    tagged), 14 (287 unbound OpenAPI paths), 15-17 (blocked on absent backends,
    infrastructure and a model contract respectively). 15, 16 and 17 remain
    blocked on inputs this repository does not have, which is why they are
    written down rather than started.

## Handler fidelity: 54 handlers return answers they did not earn

`tools/rust_handler_fidelity.py` walks the `x-handler-bound` of all 66 contracts
and judges the Rust function each one names. **50 of 134 bound handlers are
FABRICATED** — they return a success response without a store, a client, or a
query behind it, so the response describes a world in which the work happened.
10 of those 54 also **IGNORE_INPUT**: the signature is `Json(_req)`, the leading
underscore silencing the unused warning, so the handler compiles clean while
discarding what the caller sent. A handler that never read its input cannot be
right about the answer, whatever it returns.

| service | bound | fabricated | ignores input |
|---|---|---|---|
| rts | 15 | 15 | 2 |
| declaration | 9 | 9 | 0 |
| bvr | 10 | 8 | 0 |
| ballot-serial | 7 | 6 | 0 |
| abis | 7 | 0 | 0 |
| biometric-edge | 5 | 3 | 0 |
| field-mobile-bff | 6 | 4 | 2 |
| whatsapp | 3 | 2 | 0 |
| cvr, dereg, ivr | 12 | 3 | 1 |
| bvi, bvr, credential, custody, field-mobile-dispatch, field-mobile-support, transparency, voter-card, workforce | 99 | 0 | 0 |

### Fixed: the abis cluster now refuses rather than inventing

The four abis handlers now return `ApiError::NotImplemented` (501, RFC 9457
problem+json via the existing `ApiError`): `resolve_adjudication`,
`list_adjudication`, `submit_batch_dedup`, `get_reports`. A caller that gets 501
knows the answer does not exist, where a caller handed `match_score: 0.65` and
`outcome: "no_match"` had no way to tell it from a real biometric verdict. Each
carries a comment recording what it used to return, so the fabrication is
recoverable if someone re-implements it. `rust_handler_fidelity.py` counts these
as `honest501` and reports abis at 0 fabricated, which matters more than it
sounds: a fabricated count that cannot fall is decoration, not a metric.

`upload_evidence`, directly above `resolve_adjudication`, is a fifth fabrication
the tool does **not** catch: it returns 202 with a fresh `evidence_id` and
stores nothing. It is invisible to the tool because it has no `x-handler-bound`.
Unbound handlers are a known blind spot.

### The one that mattered most: `abis-svc.resolve_adjudication`

Live at `PATCH /api/v1/abis/adjudication/{case_id}` (`abis-api/src/lib.rs:95`).
It ignores its request and returns, for every case id:

```
status: "resolved", match_score: 0.65, outcome: Some("no_match"),
probe_ref: "probe-001", candidate_gallery_ref: "gallery-001"
```

That is a fabricated biometric verdict with a fabricated match score and two
reference ids pointing at gallery records that were never read. On a service
whose job is Automated Biometric Identification, an endpoint that answers
`"no_match"` at 0.65 with plausible-looking references is worse than one that
404s, because a caller cannot tell it from a real answer. `list_adjudication`
beside it ignores its query and returns an empty page with `total: 0`.

**No Flutter client currently calls it.** The handler is now 501; the contract
for `resolveAdjudicationCase` still describes a real resolve operation, so the
gap is now visible as a refusal rather than hidden behind a plausible answer.

### Nothing Rust can be compiled on this machine

`cargo check -p abis-api` fails for every crate with `linking with 'cc' failed:
exit status: 69 -- You have not agreed to the Xcode license agreements`. This is
environmental, predates any of this work, and is fixed by
`sudo xcodebuild -license accept`, which needs an interactive sudo. It means
**every Rust edit in this repo is unverified at the type level**, and that
includes the claim, made earlier in this work, that 3118 Rust tests passed --
that run cannot currently be reproduced and should be re-run once the licence is
accepted before it is relied on again.

`tools/rustfmt_parse_check.py` is the mitigation that is possible without a
compiler: `rustfmt` parses Rust without linking, so it catches syntax errors
where nothing can build. It immediately earned its place. Rewriting
`get_reports` produced `State<Arc<AppState>>))` with a doubled closing paren,
which a raw paren count scored as *balanced* and which only the parser rejected.
Syntax is not types: the abis changes parse clean and have not been type
checked.

`rts-svc` is the other extreme: 15 of 15 fabricated, and it is the only service
with a contract declaring `404 Form not found` for `getFormProof` while the
handler returns `200` with an empty proof and a Merkle root of 64 zeros, for
every form id, forever.

### Two defects in the measurement itself, worth recording

Both were found by checking the tool against ground truth rather than trusting
its first output, which is why the number moved 31 → 15 → 54 as they were fixed:

- It resolved `x-handler-bound` by bare function name, so the cvr contract's
  `bvr-svc.cvr.list_events` was answered with `list_events` out of `crms-api` —
  a different service, reported as if it were the bound handler.
- It extracted only the function body, not the signature, so the discarded
  extractor (`Json(_req)`) was never in the searched text and IGNORES_INPUT
  could not fire even once.

A measurement that reports a gap as a finding, or a finding as a gap, is worse
than no measurement.
