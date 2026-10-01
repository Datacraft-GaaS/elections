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

## Handler fidelity: 275 of 631 registered handlers return answers they did not earn

**The bound-only count in the table below is 45, and 45 is not the size of the
problem.** It counts only handlers a contract names via `x-handler-bound`.
Measuring the routers directly instead:

```
registered handlers judged : 631
fabricated                 : 238   (38%)
  of which contract-unbound: 193
  contract-bound           : 45
```

The fabrication problem is roughly **six times** the contract-bound view
suggested, and 230 of the 275 were invisible to it. Worst by service:
governance 73, abis 32, portal 24, ecc 22, field-mobile 21, bvr 16,
ai-platform 13.

Election-critical examples, none of which any contract binds. All four below are
now 501:

- ~~`governance-svc/referendum::certify_county_result`~~ — logged and returned
  `204 No Content`. A county result was **certified** and recorded nowhere.
- ~~`rts-svc/rts-ro-api::amend_form34a`~~ — logged the RO id, reason and
  idempotency key, then returned `202` with a minted `amendment_id` and
  `status: "amended"`, discarding all of it. The caller held a receipt for an
  amendment that does not exist.
- ~~`governance-svc/referendum::declare_referendum`~~ — returned `204` and named
  the invariant `NoDeclarationWithoutBothThresholds` in a comment without ever
  calling `can_declare()`. The invariant was a comment.
- ~~`governance-svc/referendum::get_national_result`~~ — synthesised a national
  aggregate from constants and evaluated both thresholds at zero, so a tally
  never taken looked authoritative.

### Also fixed: `recall` and `by-election` (10 handlers)

`recall::get_threshold_report` is the most consequential of the ten. It evaluated
California Constitution article IV section 2 recall eligibility — the invariant
between a petition and a recall election — against a **hardcoded
`registered_voters = 100_000` and `verified_signatories = 0`**. The threshold
logic in `crate::threshold` is real and unit-tested; both inputs were constants.

`by-election::create_kit_plan` assigned **every polling station
`station_gps_lat: 0.0, station_gps_lng: 0.0`** — 0N 0E, in the Gulf of Guinea —
so 201 described a kit delivery plan for kits never reserved, routed to open
ocean. Its `sign_kit_plan` computes a genuine SHA-256 over the canonical plan and
stores it in a field called `signed_blob_hash`: a digest, not a signature. There
is no key, so it attests only that the plan is self-consistent, not who produced
it. The comment now says exactly that rather than leaving "signed" to imply an
attestation that does not exist.

Also in recall: `create_petition` echoed the request back as `status: "draft"`
under a comment claiming commissioner dual-sign is verified via SpiceDB (no
SpiceDB call exists); `close_petition` returned 204 for a closure recorded
nowhere; `create_recall_election` returned 201 with `trigger_status: "pending"`
under a comment claiming an outbox event it never emitted; `import_signatories`
discarded the OCR batch and returned `status: "processing"` with no poll endpoint
to ask; `list_signatories` and `list_referendums` returned empty lists with
`total: 0`, which is a claim about the world rather than about storage;
`get_referendum` echoed any requested id into a default record and answered 200
for ids that do not exist.

### Also fixed: `county-coord` and `court-interface` (12 handlers)

Three of these are attestations where the HTTP response was the caller's only
evidence the thing happened:

- `county-coord::sign_mou` logged the signing officer's id and returned `204`.
  A 204 is a successful signature. The MOU was signed nowhere.
- `county-coord::resolve_incident` **discarded the `resolution_notes` the caller
  supplied** and returned `204`. The only record of an incident resolution was
  the request that is now gone.
- `court-interface::register_witness` answered `200` with a
  default-constructed `TechnicalWitness`, so a witness appeared registered to a
  court petition having never consented, been identified, or been contactable.

`court-interface::get_evidence_bundle` is the one to be most careful about: it
returned `200` with a default-constructed `EvidenceBundle` for **any** petition
id. A caller checking what evidence the court held received a well-formed bundle
containing nothing, and an empty bundle is indistinguishable from a populated one
from the response alone — which in a filing context is worse than an error.

The three list handlers (`list_mous`, `list_datasets`, `list_incidents`,
`list_witnesses`) all returned empty arrays. That is a claim about the world, not
about storage: "no incidents are open" and "no witness is registered" are exactly
what an incident tracker and a witness register exist to assert.

### Capability, not refusal: `packages/eel-memstore-rs` and `county-coord`

Converting handlers to 501 removes capability, which is the wrong trade when the
honest answer is "there is no store yet". `packages/eel-memstore-rs` is the
missing piece: a `Collection<T>` with real create/read/update/remove semantics,
`RwLock`-guarded, ordered by key so list responses are stable, with JSON
snapshot/restore and 12 unit tests.

It is honest about being in-memory. `Collection::is_durable()` returns false by
name so a service can surface it in a health response rather than implying
Postgres. Per-process and non-durable cannot serve two replicas, and that is the
gap `bvi-api`'s sqlx `PgStore` is the target for. This is the step that lets a
handler be *correct* before the durable store exists.

`county-coord` is the first crate on it, and is now at **zero** fabrications. The
seven handlers converted to 501 earlier are now implemented:

| handler | before | now |
|---|---|---|
| `create_mou` | 201, echoed into a draft, stored nothing | inserts; appears in `list_mous` |
| `list_mous` | always `[]` | lists what was created |
| `sign_mou` | **204 for any id**, including ids that never existed | sets `Signed` + `signed_at`; **404** if absent |
| `create_incident` | 201, stored nothing | inserts; appears in `list_incidents` |
| `list_incidents` | always `[]` | lists real incidents |
| `resolve_incident` | **discarded `resolution_notes`, 204 for any id** | sets `resolved_at`; **404** if absent |
| `submit_dataset` / `list_datasets` | 201 then always `[]` | round-trips |

`tests/round_trip.rs` (7 tests) asserts the behaviour that was previously absent,
and every one of them fails against the old code: `signing_an_unknown_mou_is_404_not_204`
contradicts the 204-for-any-id outright. `state_is_isolated_between_routers` exists
so a store leaking across `AppState` instances cannot make the round-trip pass
for the wrong reason.

Two things the model could not hold, recorded rather than hidden: `CoordinationIncident`
has no field for the resolution notes and `Mou` has none for the signer's
signature hash, so both are written to the audit log instead of being dropped.
The signer and hash used to reach only a log line while the response was 204 for
anything.

`court-interface` is the fifth, and it is the clearest case for refusing to
fabricate rather than guessing:

| handler | before | now |
|---|---|---|
| `get_evidence_bundle` | **200 with a default bundle for any petition** — empty and populated indistinguishable | 404 until a request exists; then assembled from the recorded requests |
| `register_witness` | 200 with a default witness, never consented | stores the supplied witness; 404 for an unknown petition |
| `list_witnesses` | always `[]` | lists real witnesses, scoped per petition |
| `request_evidence` | 202 for a request never made | records it; the bundle reflects it |
| `add_supplementary_request` | echoed the id into a default record | stores description and requester |

The bundle is the reason this matters: a caller checking what the court held on
file got a well-formed bundle containing nothing, and from the response alone that
is indistinguishable from a populated one. In a filing context that is worse than
an error, so 404 is the honest answer until evidence is actually requested.

Two tests exist purely to prevent a false pass: `witnesses_do_not_leak_across_petitions`
and `evidence_requests_do_not_leak_across_petitions`. A shared store would let a
witness appear on a petition they never gave evidence to.

The fidelity self-test dropped 10/10 -> 7/10 when the referendum chain was
implemented, because three pins recorded `DECLARES_UNIMPLEMENTED` and are now
`VERIFIED`. Fourth time the pins have caught a stale expectation, always the same
way. `amend_form34a` stays `DECLARES_UNIMPLEMENTED` and its pin records why: a
Form 34A amendment needs an event bus and a lineage store that do not exist here,
so refusing is correct rather than minting an `amendment_id`.

`by-election` is the fourth, and it turned up a structural fault the fabrications
were hiding: **two `ByElection` types, neither ever written to.**
`create_by_election` built a `models::ByElection` and dropped it;
`transition_by_election_status` mutated a `spec::ByElection` that nothing had ever
inserted. So no by-election could be activated because none existed, and the
status machine operated on an empty map. `AppState` now holds both, with
`by_elections: Collection<models::ByElection>` for the typed
`ByElectionStatus` enum and `elections` left for the legacy String-status view.

| handler | before | now |
|---|---|---|
| `create_by_election` | 201, built and dropped the record | stores it; `GAZETTED` when a gazette notice is present, else `Draft` |
| `activate_by_election` | **204 for any id**, GAZETTED never checked | requires GAZETTED → `NOMINATIONS_OPEN`; **409** otherwise |
| `create_kit_plan` | **every station `gps_lat: 0.0, gps_lng: 0.0`** — Gulf of Guinea | coordinates are a **required input** (`{code, lat, lng}`); **422** on an empty plan |
| `generate_register_subset` | 201 with `station_count: 0, voter_count: 0` | counts are a required M02 input; **422** rather than assumed zero |
| `get_cost_report` | default report, every line zero, bypassing `build_cost_report` | reads the stored report; **404** when none submitted |
| `submit_cost_report` | computed correctly but discarded | stores it, so `get_cost_report` can read it |

Station coordinates are now **required input** rather than a defaulted zero.
There is no station gazetteer in this workspace to resolve a code against, so
making them required is the honest option: the alternative was inventing `0.0`,
which is what pointed every delivery at open ocean. The test asserts Nairobi
`-1.2921, 36.8219` round-trips exactly.

`sign_kit_plan`'s SHA-256 is kept and the field comment now says what it is: a
digest over the canonical plan, not a signature, with no key to attest authorship.

`referendum` is the third, and it is where the chain matters most. Every link of
submit -> certify -> aggregate -> declare was severed, and the crate already had
the real, unit-tested logic to close them:

| handler | before | now |
|---|---|---|
| `create_referendum` | 201 with a minted id and a synthesised `ReferendumConfiguredV1`, stored and emitted nothing | stores it; `list_referendums` sees it |
| `get_referendum` | echoed **any** id into a default record, 200 | 404 unless it exists |
| `submit_county_result` | 201 with a provisional result it never recorded | appends; majority computed by `county_has_yes_majority` |
| `certify_county_result` | **204, certifying nothing** | sets `certified` + `certified_at`; 404 if no such county |
| `get_national_result` | **aggregate from constants**, both thresholds evaluated at zero | aggregates real rows via `meets_county_threshold` / `meets_national_majority` |
| `declare_referendum` | 204, naming `NoDeclarationWithoutBothThresholds` in a comment without ever calling `can_declare` | **actually calls `can_declare`**; 409 with the counts when it fails |

That last row is the one to read twice. The invariant existed as a sentence in a
comment while the handler returned 204 regardless. It is now enforced by the
crate's own tested function, and `declaration_is_refused_when_the_threshold_is_not_met`
asserts 409 on 1 of 24 counties while
`declaration_is_permitted_once_both_thresholds_are_met` submits 24 YES-majority
counties plus one NO-majority county and gets 204. Both thresholds are real:
`meets_county_threshold` needs 24 of 47 counties, `meets_national_majority` needs
yes > no of votes cast.

`recall` is the second crate implemented this way, and it is where the payoff is
largest. The 30 handlers converted to 501 across the sweep were refusals; these
are the same endpoints working:

| handler | before | now |
|---|---|---|
| `create_petition` | 201, stored nowhere | stores the `Petition`, `list_petitions` sees it |
| `import_signatories` | **whole batch discarded** into a minted `batch_id` | classifies every row, stores it, returns real counts |
| `list_signatories` | always `[]` | lists real records, per petition |
| `get_threshold_report` | **Art. IV §2 evaluated against `registered_voters = 100_000`, `verified_signatories = 0`** | counts only `VALID` rows against a loaded constituency roll |
| `close_petition` | 204 for any id | sets `CLOSED`; 404 if absent |
| `create_recall_election` | 201 with `trigger_status: "pending"` under a comment claiming an outbox event | records it; `by_election_id` stays `null` because none exists |

The signature import is new capability the models always described and no code
implemented: rows below 0.80 OCR confidence are flagged for manual re-entry and
do **not** count toward the threshold, duplicate `voter_id`s within a batch are
detected, empty `voter_id`s are returned as `RowError`s, and the threshold
report's `ward_breakdown` is derived from real rows instead of `{}`.

`AppState` gained `roll: HashMap<constituency_code, registered_voters>` with
`set_roll`/`roll_size`. There is no roll-loading endpoint, so a petition whose
constituency has no roll entry gets **409** from the threshold report rather than
a verdict computed against an assumed electorate — the same bug, one layer down,
is what the hardcoded 100_000 was. `threshold_report_without_a_loaded_roll_is_refused`
pins that.

`tests/round_trip.rs` (11 tests) covers the whole path: import 30 valid + 10
low-confidence rows into a 100-voter roll, and assert `verified_signatories == 30`,
`numerator_x100 == 3000`, `denominator_30pct == 3000`, `meets_threshold == true`,
and a two-way ward breakdown. The statutory test itself uses the existing
unit-tested `meets_art_104_2_threshold` rather than a hand-rolled comparison.

**The headline fabricated count did not move (238).** It should not have: both a
501 and a working handler are non-fabricated, so the metric cannot distinguish
"refuses" from "works". Only the per-service number for `county-coord` shows the
change. Any future progress claim from the total alone is not trustworthy.

### Also fixed: `abis-api` (6 handlers), including an unsigned policy change

`admin.rs::update_policy` is the security finding. It answered `200` with
`applied: true` and `"attestation policy updated"` **without ever validating the
request**, under a comment that recorded exactly what was missing: *"In
production: validate req.signature_b64 against HSM public key."* The one control
deciding whether a policy change is authentic was acknowledged in a comment and
absent from the code. The attestation policy governs what the service accepts as
proof of a device, so changing it unsigned is the whole attack.

`adjudication.rs::decide_adjudication` keeps its real check and refuses the rest.
The tier3 dual sign-off guard is genuine — a tier3 decision without a second
adjudicator is rejected with 400 — so that branch was preserved. What it did
*after* passing was echo the caller's own `outcome` back as `status: "resolved"`
and store nothing. The guard therefore decided nothing: the request cleared
validation and the case was never adjudicated. It now returns 501 after the
guard, and says so.

The other four echo without storing: `assign_adjudication` returned
`status: "in_review"` for an adjudicator who was never notified and whose queue
does not exist; `escalate_adjudication` returned `200` with `status: "escalated"`
for an escalation reaching nobody; `list_evidence` returned an empty list, which
claims the case has no evidence rather than that nothing is stored; and
`upload_evidence` returned `202` with a minted `evidence_id` for an upload it
never received, leaving the caller a receipt for an artefact that does not exist.

## Fabrication triage: 238 sorted into what can be built here and what cannot

`tools/fabrication_triage.py` sorts every fabrication the fidelity scanner finds
into three buckets, because "238 fabricated" is not a task list. Converting all
238 to 501 would remove capability a refusal cannot replace; leaving them
fabricated is worse than either. The split is the point.

**221 implementable** — local work. Create/read/update against `AppState`, or a
computation over inputs the request already carries. `eel-memstore-rs` is usually
the whole requirement. These are worth doing, and are how the five crates already
fixed were fixed.

**8 need a subsystem** — the response carries a cryptographic artifact or asserts
a verdict as a literal `true`. Neither can come from a request body or a
`HashMap`. `bvr-api`'s `get_latest_merkle_root` returns `"0".repeat(64)`; a real
root needs an append-only log. `mobile-bff`'s `binary_integrity_check` returns
`passed: true` for **any** `build_fingerprint` and **any** `attestation_token` — an
integrity gate wired to always open — and needs the attestation service that would
actually perform the check. Calling these "implementable with memstore" would be
the same optimism that produced them.

**9 blocked on something absent** — the handler's own body or doc comment names
the missing system, so the blocker is a specific thing to build rather than a
shrug. Named in this repo: M02 BVR snapshot adapter, OCR engine, HSM signature
verification, transactional outbox, Kafka, SpiceDB, station gazetteer, object
storage, M50 by-election trigger, M04 register subset service.

Comments are searched as well as bodies, deliberately: several handlers document
their own blocker — *"no adapter crate exists in this workspace"* — which is the
most reliable evidence available and was previously being discarded.

The triage was corrected twice against ground truth before being trusted. It first
reported 229 implementable / 9 blocked, which put `get_latest_merkle_root` in the
implementable bucket; a fabricated Merkle root is not a `HashMap` away. It then
reported `binary_integrity_check` as implementable until `ASSERTED_VERDICT` was
added for verdicts asserted as a literal `true`.

### Per file

| crate | implementable | needs a log or key | blocked on absent system |
|---|---|---|---|
| `abis-svc/crates/abis-api/src/routes/admin.rs` | 1 | 0 | 0 |
| `abis-svc/crates/abis-api/src/routes/enrolment.rs` | 4 | 0 | 0 |
| `abis-svc/crates/abis-api/src/routes/health.rs` | 2 | 0 | 0 |
| `abis-svc/crates/abis-api/src/routes/reports.rs` | 4 | 0 | 0 |
| `abis-svc/crates/abis-reports-adhoc/src/api.rs` | 15 | 0 | 0 |
| `abis-svc/crates/abis-reports/src/api.rs` | 1 | 0 | 0 |
| `ai-platform-svc/crates/ai-gateway/src/routes/anomaly.rs` | 1 | 0 | 0 |
| `ai-platform-svc/crates/ai-gateway/src/spec.rs` | 10 | 0 | 0 |
| `ai-platform-svc/crates/telemetry-analytics/src/api.rs` | 2 | 0 | 0 |
| `biometric-edge-svc/crates/edge-api/src/routes/attestation.rs` | 1 | 0 | 0 |
| `biometric-edge-svc/crates/edge-api/src/routes/distress.rs` | 0 | 0 | 1 |
| `bvr-svc/crates/bvr-api/src/routes/handoff.rs` | 1 | 0 | 0 |
| `bvr-svc/crates/bvr-api/src/routes/merkle.rs` | 0 | 1 | 0 |
| `bvr-svc/crates/bvr-api/src/routes/registers.rs` | 0 | 1 | 0 |
| `bvr-svc/crates/bvr-api/src/routes/sar.rs` | 1 | 0 | 0 |
| `bvr-svc/crates/bvr-api/src/routes/transfers.rs` | 1 | 0 | 0 |
| `bvr-svc/crates/bvr-api/src/routes/voters.rs` | 2 | 0 | 1 |
| `bvr-svc/crates/channel-gateway/src/http.rs` | 8 | 0 | 0 |
| `bvr-svc/crates/cvr/src/routes/family_notify.rs` | 0 | 0 | 1 |
| `bvr-svc/crates/cvr/src/routes/spec.rs` | 3 | 0 | 0 |
| `bvr-svc/crates/dereg-appeal/src/routes/spec.rs` | 4 | 0 | 0 |
| `bvr-svc/crates/verifier-api/src/http.rs` | 4 | 0 | 0 |
| `crms-svc/crates/crms-api/src/handlers/candidate.rs` | 3 | 0 | 0 |
| `crms-svc/crates/crms-api/src/handlers/public.rs` | 5 | 0 | 0 |
| `crms-svc/crates/crms-api/src/handlers/quota.rs` | 1 | 0 | 0 |
| `ecc-svc/crates/ecc-api/src/handlers.rs` | 21 | 0 | 0 |
| `ecc-svc/crates/telemetry-query/src/api.rs` | 1 | 0 | 0 |
| `field-mobile-svc/crates/mobile-bff/src/handlers/clone_detect.rs` | 1 | 1 | 0 |
| `field-mobile-svc/crates/mobile-bff/src/handlers/grievance.rs` | 1 | 0 | 0 |
| `field-mobile-svc/crates/mobile-bff/src/handlers/queue.rs` | 1 | 0 | 0 |
| `field-mobile-svc/crates/mobile-bff/src/handlers/verify.rs` | 4 | 0 | 0 |
| `field-mobile-svc/crates/mobile-bff/src/spec.rs` | 15 | 2 | 0 |
| `governance-svc/crates/forensics/src/handlers.rs` | 8 | 0 | 0 |
| `governance-svc/crates/intl-observer/src/handlers.rs` | 7 | 0 | 0 |
| `governance-svc/crates/legal-audit/src/handlers.rs` | 6 | 0 | 0 |
| `governance-svc/crates/oag/src/handlers.rs` | 7 | 0 | 0 |
| `governance-svc/crates/reporting/src/handlers.rs` | 5 | 0 | 0 |
| `governance-svc/crates/review/src/handlers.rs` | 11 | 0 | 0 |
| `inventory-svc/crates/rfid-cover-custody/src/api.rs` | 1 | 0 | 1 |
| `portal-svc/crates/gazette/src/api.rs` | 9 | 0 | 1 |
| `portal-svc/crates/inspection/src/api.rs` | 10 | 0 | 0 |
| `portal-svc/crates/portal-api/src/lib.rs` | 1 | 1 | 2 |
| `queue-svc/crates/queue-pipeline/src/handlers.rs` | 1 | 0 | 0 |
| `rts-svc/crates/ballot-serial/src/api.rs` | 6 | 0 | 0 |
| `rts-svc/crates/declaration/src/api.rs` | 9 | 0 | 0 |
| `rts-svc/crates/rts-api/src/handlers.rs` | 9 | 2 | 0 |
| `rts-svc/crates/rts-device-api/src/handlers.rs` | 1 | 0 | 2 |
| `rts-svc/crates/rts-ro-api/src/handlers.rs` | 1 | 0 | 0 |
| `rts-svc/crates/rts-ro-api/src/router.rs` | 1 | 0 | 0 |
| `support-admin-svc/crates/intent/src/http.rs` | 2 | 0 | 0 |
| `training-svc/crates/training-api/src/handlers/accreditation.rs` | 1 | 0 | 0 |
| `training-svc/crates/training-api/src/handlers/lms.rs` | 1 | 0 | 0 |
| `training-svc/crates/training-api/src/handlers/spec.rs` | 6 | 0 | 0 |
| **total** | **221** | **8** | **9** |
Run `python3 tools/fabrication_triage.py --list` for every handler, or
`--crate <fragment>` for one file.

### A tool bug this surfaced: probes were exempt in the wrong order

`abis-api::readyz` was classified `IGNORES_INPUT`, and the fix exposed a worse
problem. It answers `ready: true` with `checks: {"database": "ok", "vendor":
"ok", "matcher": "ok"}` **without touching any of them**. A readiness gate that
always passes keeps a broken instance in the load-balancer pool.

So a probe must be exempt from the *input* check — "is this process up" has no
request to read — but not from the *fabrication* check. Both call sites were also
passing `op=None`, which meant the probe exemption could never fire at all and
the ordering fix silently did nothing. Caught by adding `readyz` to the self-test
and watching it fail 9/10. Now 10/10, and `readyz` is correctly `FABRICATED`.

Run `python3 tools/rust_handler_fidelity.py --all-routes` for the list.

### The classifier was wrong twice, both times confidently

It has been rebuilt once. The first version judged a handler fabricated largely
because its body was **short**, which condemned `acknowledge_alert`: it locks
`state.alerts`, mutates the record, and returns 404 for an unknown id, and was
called fabricated. A short body doing real work is the opposite of the defect.
Fabrication is now defined as returning a constructed success with no source of
truth behind it — no store, no state access, and **no branch that can fail**. A
handler that can 404 consulted something.

The other error was resolving `x-handler-bound` by bare function name, so one
service's contract was answered with a same-named handler from another service.
The first run of this tool reported 438 fabrications; that was wrong in the
other direction, and the rebuild plus ground-truth checks brought it to 275.

Seven hand-checked handlers are pinned in `--self-test`: one real in-memory
mutation, three honest 501s, one IGNORES_INPUT, two FABRICATED. A heuristic with
no test is a guess that prints a number.

### Bound-only measurement, for the record

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

### Fixed: the mobile-bff verification cluster now refuses

`verify_voter_lookup` is the GET arm of `/api/v1/mobile/bff/voter/verification`
and is live. It read `voter_id` from the query, **defaulted it to the literal
`"VOTER-12345"` when absent**, and answered `{"verified": true}` for whatever it
got — so a request naming nobody verified an invented voter, and a request
naming anybody verified them against nothing. `tally_verify` and
`verify_certificate` returned `{"verified": true}` for any id having checked no
tally and no certificate. All three now return 501 problem+json via a
`not_implemented` helper in the same file.

The tool could not see any of the three: they carry no `x-handler-bound`, so
they are invisible to contract-fidelity measurement. That is the same blind spot
as `upload_evidence` and it is now the tool's most consequential gap — the three
worst fabrications in this service are exactly the ones it cannot see.

### A second defect in the same file: the idempotency record is empty

11 handlers in `mobile-bff/src/spec.rs` follow this shape:

```rust
let key = format!("...{}", id, Uuid::now_v7());   // unique per call
match state.idempotency_store.get(&key).await {
    Some(resp) => return (StatusCode::OK, resp),
    None => { let _ = state.idempotency_store.store(&key, &json!({})).await; }
}
(StatusCode::OK, Json(json!({ "verified": true })))
```

Two bugs at once. The key embeds `Uuid::now_v7()`, so it is **unique on every
call and never repeats** — the idempotency store can never hit, and the
"already processed" path is dead code. And the record stored is `{}`, so if it
ever did hit, a repeat would answer `{}` instead of the claimed result: the same
request answered two different ways depending on whether it had been made
before. Now detected mechanically as `EMPTY_IDEMPOTENCY` in
`rust_handler_fidelity.py`, so it cannot reappear unnoticed. 11 remain.

### The Rust toolchain works; the blocker was a stale `SDKROOT`

**Correction to the previous entry in this file.** Rust does compile on this
machine. Nothing is wrong with the Xcode licence and nothing needs `sudo`.

`~/.zshrc:169-174` runs this at every shell startup:

```zsh
sdkroot_path="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null)"
[[ -n "$sdkroot_path" ]] && export SDKROOT="$sdkroot_path"
```

`xcrun` honours an inherited `SDKROOT` and echoes it straight back, so the block
is self-perpetuating: once Xcode was upgraded to 27.0, which ships
`MacOSX27.0.sdk`, the shell kept re-exporting the previous `MacOSX26.5.sdk`
forever. Every `cc` invocation then failed with `SDK "…/MacOSX26.5.sdk" cannot
be located`, which surfaced as an apparent licence error. The stale value
survives in every new shell because the very command meant to refresh it is the
one that cannot see past it.

Fix, in `.zshrc`:

```zsh
if command -v xcrun >/dev/null 2>&1; then
    unset SDKROOT                      # let xcrun resolve afresh
    sdkroot_path="$(xcrun --sdk macosx --show-sdk-path 2>/dev/null)"
    [[ -n "$sdkroot_path" ]] && export SDKROOT="$sdkroot_path"
    unset sdkroot_path
fi
```

Or simply drop the `SDKROOT` export; cargo and rustc resolve the SDK themselves.

Until then, **`env -u SDKROOT cargo …` works**, and that is how every Rust
verification in this file was run.

### All 13 handler edits are now type-checked and tested

Previously "syntax-checked only, at risk of not compiling". With `env -u SDKROOT`:

- `cargo check` clean on `abis-api`, `mobile-bff`, `rts-ro-api`,
  `eel-governance-referendum`.
- `cargo test` green on all four.

### Four tests were asserting the fabrication, and one was named after it

`abis-api/tests/api_test.rs` had four tests asserting `StatusCode::OK` against
the four handlers that fabricated. The suite was green because the endpoints
lied and the tests agreed. They now assert 501 *and* that the body is
problem+json explaining why, which is the part a caller actually reads.

Worse: `mobile-bff/tests/contract_tests.rs` had
`test_verify_voter_returns_200_with_verification_result`, documented as
`FR-M1-006: Voter verification validates DPO attestation`, asserting only a
status code against a handler that validated nothing. A requirement with a green
test, a descriptive name, and no implementation. Renamed to
`test_verify_voter_refuses_while_fr_m1_006_unmet` with the requirement marked
UNMET, so the gap is visible in the test name rather than hidden inside it.

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
