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
fabricated                 : 114   (18%)
  of which contract-unbound: 74
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

**25 implementable** (was 221; 221 converted) — local work. Create/read/update
against `AppState`, or a computation over inputs the request already carries.
`eel-memstore-rs` is usually the whole requirement. These are worth doing, and are
how the ten crates already fixed were fixed.

Eighteen crates are now at zero fabrications. and the one remaining handler here is recorded as blocked: `county-coord`, `recall`, `referendum`,
`by-election`, `court-interface`, `ecc-api` (21 handlers, whose records took no
state at all — a complaint filed through the grievance portal could never be
retrieved, an escalation could never be resolved, and `get_audit_log` returned
`{"entries": []}`, which is an audit log that records nothing), and `review`
(10 handlers, where `submit_finding` answered 201 and dropped the finding and
`triage_finding`, `sign_recommendation`, `record_long_drip_review` and
`record_parliament_response` each returned 204 for **any** id).

`review::sign_recommendation` is worth singling out: the request carries an
`hsm_signature` and the handler discarded it, logging only the signer's name, then
answered 204 — so a recommendation could read as chair-signed with no signature
behind it. An empty signature is now refused with 422, and a presented one is
recorded verbatim to the audit trail. It is explicitly **not verified**: there is
no HSM public key in this workspace to check it against, and the log line says so
rather than leaving the reader to assume.

`review` also has no endpoint that creates a recommendation, so signing one is a
404. That is recorded rather than papered over by inventing a creation route.

`crms-api` exposed **a whole false-positive class in the classifier**, which is
worth recording before its own six fixes.

Four of its nine flagged handlers were not fabrications at all.
`begin_ro_review`, `mark_docs_uploaded` and `withdraw_candidate` are one line each
— `with_candidate(&state, id, |c| c.begin_ro_review())` — delegating to the real
candidate state machine in `crms_domain`. `evaluate_quota_rule` runs the crate's
own quota evaluation over the candidates supplied. All four were flagged because a
body-only heuristic cannot see through a delegation: the store is reached behind
an `&`, and a pure computation has no store at all.

Two signals fix this honestly rather than by exception. `STATE_MARKERS` now treats
handing state to a helper as a store access, and a new `delegates_elsewhere`
check recognises a call to a function defined in **another file** as real work
happening elsewhere. `evaluate_quota_rule` is additionally pinned in the
self-test at VERIFIED, because a pure computation over request input is real
capability and no heuristic will reliably separate it from a fabrication.

**This removed 10 handlers from the fabricated count with no code change at all**,
which is the same lesson as `redirect_legacy`: every one of these was a case where
the measurement was wrong and the code was right. Fabricated 161 -> 151 on the
classifier fix alone, then 151 -> 146 on the six real fixes below.

The six real ones: `check_symbol_collision` answered `collision: false` for every
request **without looking at the registered symbols**, so a genuine clash with a
symbol already registered to another party was reported as clear; it now compares
case-insensitively against the registry and names the conflicting registration.
`submit_primary_results` and `submit_primary_dispute` each minted an id and
discarded the submission, and `record_registration_decision` did the same for a
registration decision — all three now persist and 404 for an unknown parent.
`initiate_party_document_upload` minted an `upload_ref` and dropped the request.

`governance/forensics` is the clearest case of **a whole surface answering from
nothing**. All eight `/v1/forensics/*` endpoints took no state at all: `list_cycles`
and `list_estimates` returned a literal `vec![]`, `trigger_run` answered `202
Accepted` with an empty body for *any* id — including a cycle that did not exist —
and discarded the path id, and `get_public_report` answered `200` with an empty
`PublicReport` for every id, which reads as a real but blank report.

The one that matters most is `publish_replication`. A replication package is an
attestation that an analysis was produced and can be reproduced by someone else.
The crate already had the rule encoded and unused:
`ForensicsEngine::can_publish_replication` requires `CycleStatus::Complete`. The
handler never called it, so a **draft** cycle could publish a replication package
and a `data_hash` nobody had computed. It now calls the engine and answers `409`
with the actual status.

`trigger_run` is the honest-refusal case, and it is worth being precise about
where the boundary is. The RD/IV/DiD estimators live in
`services/governance-svc/py/forensics-ml` — a separate Python distribution
(`forensics-ml` 0.1.0) with no in-process or IPC bridge in this workspace. So the
run **does** transition the stored cycle to `RUNNING` with a real `started_at`,
because that transition is local state work, and it records
`AnalysisCycle::blocked_reason` naming the estimator that is absent. It does not
invent an estimate. The distinction the objective cares about is that a *blocked*
surface says so in a field a consumer can read, rather than answering `202` and
pretending.

Wiring this up also surfaced a contract bug worth recording: `AnalysisMethod`
carried `#[serde(rename_all = "SCREAMING_SNAKE_CASE")]`, which turns the `DiD`
variant into `DI_D`. That was never a chosen wire value, just an accident of the
variant name, and it meant `DIFF_IN_DIFF` could not be deserialized at all. The
variants are now renamed explicitly and pinned by
`engine::tests::wire_values_match_the_contract`, which also asserts `DI_D` is now
*rejected*. The two `wire()` helpers derive their string from the serde
representation rather than restating it, so a handler's conflict message and the
serialized enum cannot drift apart. (`CycleStatus` is the one enum here
serialized snake_case; the helpers serve both.)

`governance/intl-observer` repeats the forensics shape on a surface where the
severity is higher. `register_mission` assigned the literal
`workspace_id: "ws-stub"` to **every** mission while discarding the mission
itself, so two missions shared one workspace reference and none was retrievable.
Every list returned `vec![]`. And `submit_report` accepted a report against any
mission id — including one that had never been registered — with no check at all.

`ObserverEngine::can_submit_report` encoded the rule that **only an active
mission may file an observation report**, and the handler never called it. So the
question is what makes a mission active, because nothing did: registration set
`Registered` and there was no transition out of it, which would have left
`submit_report` permanently closed. Accreditation is what entitles a mission to
observe, so registration now sets `Active` when `accreditation_ref` is present and
`Registered` when it is not. That is a judgement call worth naming — it is the
one place this commit invents a rule rather than enforcing an existing one.

The engine's other check is a good example of a stub that cannot be trusted with
a store. `validate_joint_statement_participants` returns `!ids.is_empty()`, so a
joint statement could be published attributing itself to missions that do not
exist. The handler now calls the engine **and** resolves every participant
against the store, so an unknown mission is a `422` naming it and the statement is
not stored.

Scheduling a briefing also no longer marks attendance. `schedule_briefing` set
`attendance_confirmed` from nothing in particular; it is now explicitly `false`,
because attendance is confirmed by the mission, not by the act of scheduling.

`governance/legal-audit` was described in its own manifest as a "§39 rules engine,
per-event compliance verdicts, tlog-hashed (WP-5.1)". **None of those three things
existed.** All six `/legal-audit/*` handlers answered from `vec![]` or
`Default::default()`; `generate_report` answered `202 Accepted` with no body.

The shape worth dwelling on is `get_verdict`, which returned an **empty**
`ComplianceVerdict` for every record id. `passed` is `false` by default, so an
unevaluated record read as a *failed* compliance check with a rule id of `""` —
authoritative-looking, meaningful to nobody. An empty verdict is worse than a
missing one, because it answers the question.

What exists now:

- **`rules.rs`** — a real §39 rule set with section references. A record is
  non-compliant when it declares **no** legal basis, because there is no
  instrument justifying the transmission. A record declaring an instrument the
  rule set does not know is flagged `unrecognised-basis` rather than silently
  accepted, so an invented basis is visible. Every applicable rule is judged, so a
  record matching two rules yields two verdicts instead of one standing in for
  both.
- **A real hash chain.** Each verdict commits to its predecessor *and* to its
  position in the global sequence, so removing or reordering one changes every
  subsequent hash. `verify_chain_with_records` re-derives the hashes from the
  source records rather than trusting them, and `the_report_derived...` asserts
  the chain verifies inside the generated report.

Two bugs I introduced while building the chain, both caught by the tests and both
worth recording because they are the kind that pass a casual review:

1. `verdict_index` restarted at zero for each record, so two records at the same
   offset with the same rule and outcome produced **identical hashes**. Fixed by
   threading `first_index` through the global sequence.
2. Every verdict in a multi-verdict record chained to the *record's* predecessor,
   so all N verdicts pointed at the same parent rather than forming an ordered
   chain. Fixed by advancing the running predecessor within the record too.

**`tlog_inclusion_proven` is `false` on every verdict and in every report.** There
is no append-only log in this workspace to submit a leaf to, so the hash is a
local chain and is labelled as one. The old field was `tlog_hash`, which invited
exactly the reading this workspace cannot support; it is now `verdict_hash`, and
the inclusion claim is a separate field that is explicitly never true.

`party_summary` previously returned a zeroed summary for any party id — which
reads as "fully compliant". It is now a 404 for a party that has never
transmitted, and an unevaluated transmission counts as **non**-compliant rather
than being skipped.

`tests/compliance.rs` is 17 tests, all failing against the previous code, and
proven able to fail: forcing every rule to pass drops 5 of them. An earlier
attempt at that rebreak was itself a no-op — an empty basis still fell through to
the `unrecognised-basis` branch and still failed — which is worth noting because a
test that cannot fail is the failure mode this whole effort is about.

`governance/oag` is the only crate so far that had **no state module at all** —
not an empty one, not an unused one. `src/` was `handlers.rs`, `models.rs`,
`router.rs`, `lib.rs`, `main.rs` and nothing else, so there was nowhere to put
state even in principle. All seven handlers answered from `vec![]` or
`T::default()`: `get_engagement` echoed the path id into a default struct, so
asking for an engagement that had never existed returned a blank engagement with
`200 OK`, and evidence could be extracted from an engagement that was not there.

The one with real-world teeth is `record_response`. Every call minted a fresh
`response_id` for the same engagement, so the OAG's answer to an audit could be
silently replaced by a later one, and nothing recorded that a response had been
given. The lifecycle is now explicit on `EngagementStatus`:
`Scoping -> Active -> Responded -> Closed`, with `accepts_response()` false once
responded. Recording a response **moves the engagement to `Responded`**, so the
state lives on the engagement rather than being implied by the presence of a
response. `GET /oag/engagements/{id}/response` is new and returns `404` when
nothing has been answered, rather than an empty body that reads as "answered with
nothing to say".

`extract_evidence` now hashes what it actually stored — engagement, description
and extractor — so the digest is reproducible from the stored fields rather than
being an opaque value. `a_same_extract_yields_the_same_digest` pins that: two
extracts of identical content share a digest and differ only in id and timestamp.

One honest field worth naming. `PublishedReport::hash` was a non-optional
`String`, so registering a report without a digest recorded an empty string in a
field that reads like a content hash. This service does not hold the published
contents — they live in the OAG's own repository — so it cannot compute one. The
field is now `Option<String>`: null when the caller supplies nothing, and a
caller-supplied digest is preserved. A digest of the *metadata* would have looked
like a content hash without being one, which is the same mistake as legal-audit's
`tlog_hash` in a different place.

`governance/reporting` closes out the governance-svc legacy surfaces, and its five
fabrications were all the same shape: **a downstream artefact with no upstream
check**. A testimony workspace, a judiciary handover and an SC/IT pre-registration
each presuppose a report with a body behind it, and none of the three handlers
looked at the report at all. `ReportStatus::allows_downstream` encodes the rule
and all three now check it, so opening testimony against a report that is still
generating is a `409` and nothing is stored.

That required deciding what makes a report ready, and the answer is recorded
rather than assumed. A generated report starts at `GENERATING` and carries a
`blocking_reason`: the body is assembled outside this service and the reporting
data warehouse is not in this workspace, so this service records the request and
its digest but cannot render statutory content. `POST /reporting/reports/{id}/ready`
is the transition, and it is a fact about the report — marking it ready clears
the blocking reason. Without that, "ready" would have been a value the handlers
assumed, and the gate would have been circular.

`get_report` returned a blank `StatutoryReport` for **any** id with `200 OK`. So
asking for a report that had never been generated returned an empty report rather
than a 404, and — because `StatutoryReport::sig` was a non-optional `String` —
that blank report carried an **empty signature**. `sig` is now `content_hash`, the
SHA-256 of the canonical request, and there is nothing that could be mistaken for
an officer's cryptographic signature: no signing key or HSM is reachable from
this service. The judiciary handover and the SC/IT pre-registration got the same
treatment. `handover_ref` is the local id and says so; `confirmation_ref` is
`None` because no SC/IT committee endpoint exists here to issue one, and filling
it with a local id would imply a confirmation that was never given.

Handing a report to a court moves it to `DELIVERED`, and a delivered report
cannot return to generating — it is a historical record, and reversing that
would let a handed-over report be quietly rewritten.

That is the last of the governance-svc legacy surfaces: `forensics`,
`intl-observer`, `legal-audit`, `oag` and `reporting` are all at zero, taking the
crate total from 238 to 114.

`rts-svc/declaration` is the most consequential handler set in this tree, because
it is the one that decides who won a presidential election — and **all nine
handlers answered from the path id alone.** The `:id` was never checked against
anything.

The one that matters: `declare_result` returned `status: "declared"` for **any**
id, with no winner, no `declared_at`, and no evaluation. Meanwhile the crate
contains the actual Art. 138(4) winner-determination engine — `rules::evaluate`,
requiring BOTH a strict majority AND the 25%-in-24-of-47-counties threshold, with
Elections Act §§83-84 semantics and TLA+ invariants behind it — and
`crate::rules::evaluate` was **never called from anywhere in the repository.**

So the fix was to make the engine the authority rather than add a second
implementation. `DeclareRequest` now carries the tally, because without a tally
there is nothing to evaluate, and a "declared" status with no tally is precisely
the fabrication this must not produce. The outcome is derived and reported:

- A bare national majority **does not declare**. `a_majority_without_the_county_threshold_does_not_declare`
  is the test that matters most here: 900 votes in one county is not a win.
- An exact tie yields `tie_pending`, and EC-LEG-06 then makes the certificate
  endpoint a `409`. That rule lived in `Declaration::may_issue_certificate` and
  was **unreachable**, because `get_certificate` answered `not_issued` without
  reading anything.

`set_runoff` and `set_tie` were endpoints that *asserted* an outcome. They now take
a tally and accept it **only if the rules engine produces that outcome** — a `409`
naming what the tally actually produced otherwise. An endpoint that can set a
national result by assertion is the failure mode, not the fix.

`amend_declaration` had the same problem in miniature: it minted an amendment id
and dropped the lineage, so an amendment could be layered on any tally. It now
requires `previous_snapshot_hash` to match the tally the declaration is actually
on, and `amendment_count` exposes the append-only lineage. `judiciary_ack`
requires the petition window to be `petitioned` — an acknowledgement follows a
petition, and acknowledging an unopened window records a judicial act against
nothing.

State is a real event-sourced projection (`store.rs`): the `Declaration` aggregate
is folded from `StoredEvent`s, so the aggregate's own `apply` is the only thing
that decides state and a handler cannot set a status the aggregate would not have
reached.

**A latent panic the tests caught.** All nine routes were declared with axum 0.6
path syntax (`/api/v1/declarations/:id`). The workspace is on axum 0.8, where a
`:` segment is a hard error — so `router()` would have panicked at construction.
Nothing called it, so it was never caught. The paths are now `{id}`. This is worth
more than the fabrication count: the endpoint set was not merely fabricating, it
was unconstructable.

`tests/api.rs` is 27 tests, all failing against the previous code, and proven able
to fail: bypassing `evaluate` and hardcoding a winner drops 4 of them.

`rts-svc/ballot-serial` contains the single worst fabrication found in this tree,
and it is worth describing precisely because the code *looked* like it was doing
the work.

`trigger_reconciliation` did call the real arithmetic —
`reconciliation::reconcile`, which implements FR-M63-023/024 with a `Held` branch,
`flag_reason`, and EC-DAT-11 named in its own comments and tests. But it built its
own input rather than reading the assignment:

```rust
let counts = StationCounts {
    serial_low: 1,                              // invented
    serial_high: req.deposited_count + req.spoilt_count,  // derived from the answer
    ...
};
```

`assigned == deposited + spoilt` by construction, so `accounted > assigned` was
**arithmetically unreachable** and the hold branch never fired through the API —
even though `over_assigned_triggers_hold` proves it fires when called directly.
**A station could deposit 120 ballots against a 100-ballot issue and be cleared.**
The comment above it, "We don't have the serial range in the request (needs a DB
lookup in full impl)", is the whole defect stated plainly. There is a store now, so
the range is the one the station was actually assigned.

This is a different failure mode from the `declaration` commit and worth
separating. `declaration` had a correct engine that nothing called. `ballot-serial`
had a correct engine that was called **with fabricated inputs**, and the
fabrication was constructed so that the check could not fail. A test that calls
`reconcile` directly would have stayed green forever. The only thing that caught
it was an end-to-end test asserting a station that over-deposits is held.

Three more in the same crate:

- `get_reconciliation` returned `status: "pending", result_hold: false` for **any**
  station code. A station that had never been reconciled reported no hold, which
  is the opposite of what a hold is for.
- `resolve_hold` returned `cleared` for **any** id — clearing a hold that might not
  exist. `ResolveHoldRequest` has always carried `resolved_by_po` **and**
  `resolved_by_cro`; the handler read only the first. It now requires both, and
  refuses a station with no active hold.
- `record_spoilt` never checked the serial against the station's assigned range,
  so a spoilt ballot could be recorded outside it, inflating the spoilt count and
  masking the very discrepancy the hold exists to catch (FR-M63-014).

The reconciliation also now uses `max(recorded_spoilt_serials, request.spoilt_count)`,
because a station under-reporting spoilt ballots it had itself recorded through
this same endpoint is the easiest way to make the numbers balance.

`tests/api.rs` is 21 tests, all failing against the previous code, and proven able
to fail: restoring the synthetic `1..=deposited+spoilt` range drops 3 of them.

`rts-svc/rts-api` is where the zero Merkle root lived, and the fix is the one I
said earlier might only be a documented blocker. **It was not** —
`packages/eel-crypto-rs` has a real `MerkleTree` with `inclusion_proof` and
`verify_inclusion_against_root`, and it was sitting unused in the same repository.

`get_form_proof` returned:

```json
{ "merkle_root": "0000...0000", "merkle_proof": [] }
```

The zero root is not a Merkle root of anything. A client that verified the proof
would correctly reject it; a client that only checked "is a root present" would
accept it — and a zero root is the kind of value that survives a glance. The proof
is now a real inclusion proof over the Form 34A submissions this process received,
and it is **verified before it is served**, so a proof that would not verify is
never emitted. The test rebuilds the tree from the audit trail's leaf hashes,
checks the reported root reproduces, and runs the crate's own
`verify_inclusion` against the served proof and root.

What the root is *not* is stated on the response: `tlog_inclusion_proven: false`.
It is a root over one process's forms with no cross-process agreement on the leaf
set and no append-only log behind it. That is the same distinction legal-audit
needed, and the honesty is the point — the alternative is a field that invites
exactly the reading this workspace cannot support.

The other handlers in this crate were fabrications of a more mundane kind, and
two of them would have broken a real client outright:

- **`get_icr_job` minted a *new* `job_id` on every read.** A client polling one
  form's ICR job got a different id each time and could never correlate two polls.
  The job id is now derived from the form id, so it is stable across reads.
- **`get_ack` returned `status: "pending"` for any scan id**, so a form that had
  been submitted and scanned read as never received. The scan id is now derived
  from the form id, which also makes the endpoint reachable at all.
- **`get_snapshot` returned `merkle_root: None` and `form34c: None` for any
  scope/code.** There is now a real Form 34C consolidation: candidate votes summed
  across every station that reported them in a constituency, with a deterministic
  ordering so two reads agree.
- **`resolve_dispute` answered `resolved` for any dispute id** and never read the
  `countersign_token` the request carries. The token must now be one this service
  issued *and* belong to the running officer named on the dispute, so one
  officer's token cannot countersign another's dispute.
- `provision_token` returned `rts-token-stub-<uuid>` with a hardcoded
  `expires_at` of 2030-01-01. The token is now a digest over device id, svid and
  issue time, expiring in 24 hours.
- `ingest_event` answered `queued` for anything and stored nothing. It now
  requires a non-empty envelope `id` and `type`, and records the event.
- `sms_fallback` turned an unparsed encoded payload into a "received" form. It now
  stores the envelope verbatim and says the payload is not parsed — fabricating a
  form from bytes this service cannot read is the failure this effort is about.
- `compile_form34b`/`34c` answered `compilation_queued` for any input and recorded
  nothing. Jobs are now recorded, and each says that sealing needs the HSM
  ceremony.

**Form 34A arithmetic is now checked on receipt**: votes_cast must equal
valid + rejected + spoilt, must not exceed registered voters, and the candidate
totals must equal valid_votes. A form whose own numbers disagree is evidence of a
transcription fault and should not be consolidated. Nothing checked this before,
and the sum is exactly what Form 34C aggregation is built on.

`tests/http_surface.rs` is 32 tests, all failing against the previous code, and
proven able to fail: restoring the zero root and empty proof drops
`a_form_proof_carries_a_real_root_that_verifies`.

Three of my own test expectations were wrong before the code was: I asserted a
candidate-total mismatch on a payload that failed the votes_cast check first, and
I asserted a token was stable across issuance when including the issue time is the
point. Both corrected in the tests.

`bvr-svc/channel-gateway` had the fabrication I had earlier written down as
**genuinely blocked**: two webhook endpoints returning `{"challenge": "verified"}`
for any request. I had recorded that as "WhatsApp webhook signature verification
cannot be performed — the provider app secret is not in the workspace". That was
**too broad, and this commit is the correction.**

The signature check is ordinary HMAC-SHA256 over the raw body keyed by the app
secret. `packages` in this repo already does exactly that elsewhere — see
`field-mobile-svc/telemetry-ingest/src/hmac_verify.rs` — so the *algorithm* was
never the obstacle. Only the **key** is absent, because the app secret is issued by
the provider and must never live in a repository. So the secret is read from
configuration (`WHATSAPP_APP_SECRET`), and:

- with a secret configured, the signature is genuinely checked over the raw body
  and a mismatch is a `401`;
- with no secret, the endpoints return `503 unverifiable` and the message
  endpoints **refuse** — they no longer acknowledge anything.

`SignatureVerdict` has four states — `Valid`, `Invalid`, `Unverifiable`,
`Missing` — and `Unverifiable` is deliberately distinct from `Valid`. Collapsing
those two is exactly the bug. A missing secret is reported *before* a missing
header, because no header fixes a configuration problem and reporting it as a
missing signature points the operator at the wrong place.

The USSD and IVR handlers answered `response_text: "END"` to **every** input, so
a caller dialled a service code and was disconnected before seeing a menu. There
is now a real menu per service code with session state: an empty first turn shows
the menu, `0` cancels (and only `0`), a selection advances, and
`get_ussd_session_log` returns the whole recorded exchange instead of `log: []`
for any session id. An **unoperated service code is refused** — a gateway that
answers every code accepts traffic for services it does not run. The IVR is a
matching DTMF state machine. A session id is bound to the service code that
opened it, so state cannot be carried between services, and the caller's MSISDN is
hashed rather than stored.

**The existing contract tests were pinning the fabrication**, which is worth
recording. `ussd_session_closes_with_end` asserted `action == "end"` for every
input. `ussd_session_log_is_empty_list` asserted `log == []`. `whatsapp_verify_challenge`
asserted `200 OK` from the verification endpoints. And
`whatsapp_inbound_is_acknowledged` asserted that an **unsigned** body was
acknowledged. All four now pin the real contract: an unsigned or unverifiable
webhook is refused, which is the entire purpose of a verification endpoint.

**And a second measurement fix.** The two `receive_*` handlers are one line each —
`receive(state, headers, body).await` — delegating to a same-file helper that does
the work. `delegates_elsewhere` covers a call into *another* file; the same-file
case was still being flagged, which made two genuinely implemented handlers read
as fabrications. `reaches_store_via_local_helper` now accepts a same-file
delegation **only when that helper's own body carries a store or work marker** — so
a handler calling a same-file no-op is still correctly flagged. That is the fourth
time this classifier has been wrong, and the fourth time the code turned out to be
right.

`tests/contract_tests.rs` is 16 tests, and the 4 signature tests are proven able to
fail: accepting any webhook drops all four.

`training-svc/training-api` had the two most consequential fabrications left in
this tree, and both sat in endpoints a person relies on.

`submit_assessment` returned `score_pct: 100.0, passed: true` for **every**
submission and minted a certificate id. The enrolment id was discarded outright —
`let _ = (assessment_id, req)` — so a submission could be filed against an
enrolment that did not exist and still came back a pass. **Every candidate passed
every assessment.** The crate contains its own scoring engine, `exam::score_session`,
with blueprint weighting, all-or-nothing multi-select marking and a pass mark; the
handler never called it.

`verify_badge` returned `valid: true` for **any** badge number. That is the
endpoint a verifier trusts to check an accreditation credential before letting
someone near a count centre, and it accepted every string presented to it. Badges
are now issued by the approval decision — the registrar may assign the number, and
where they do not one is derived from the application so an approved application
always has something real to check — and `verify_badge` answers 404 for an unknown
number and `valid: false` for one past its validity.

`get_certificate` returned a URL for any `certificate_id` with an `expires_at` of
**now**, so every certificate resolved and every certificate was already expired.

Two design points worth recording:

- **The assessment scores against the bank the exam module serves.** `SpecState`
  now holds the same `ExamApiState`, and `build_router` merges both surfaces over
  one store, so questions registered at `POST /v1/exams/questions` are the questions
  a spec-surface submission is marked against. A candidate cannot score themselves,
  because a submitted option that the bank does not offer maps to no index and
  counts as wrong. With no questions registered the endpoint answers `503` naming
  the route that fixes it, because scoring against nothing is how every submission
  reached 100%.
- **The weighted score exposed a behaviour worth having.** The scorer divides by
  the blueprint's *total* weight, so a candidate who answered one of four sections
  scores that section's weight, not a quarter of the pass mark. My first test
  asserted 100% for a partial answer; the code was right and the test was wrong.
  `a_partial_submission_scores_proportionally` now pins 35% for one section of
  the PO blueprint, and asserts it is a fail.

The CMS handlers echoed their input and stored nothing: `list_content` was always
empty, `get_content` returned a bare id for anything, `delete_content` answered
204 for any id, and translations were dropped so a content item could accumulate
none. All of it is stored now, updates bump a version, and a translation is a 409
rather than a silent overwrite.

**And a third measurement bug in the classifier, this one a real resolution
failure rather than a heuristic.** `verify_badge` and `submit_assessment` each
exist **twice** in `training-api`: once in the spec surface, once in the internal
`accreditation`/`lms` modules. The bare-name index took whichever file sorted
first, so a route registered in `spec.rs` was being judged against the body in
`accreditation.rs` — a different function. Bodies are now indexed under
`file::fn` and every route carries the module that registered it, so the name means
what the registering module means by it. That is the fifth time this classifier
has been wrong, and the first time the bug was in *which* function it was reading
rather than how it judged it.

`tests/spec_surface.rs` is 22 tests, all failing against the previous code, and
proven able to fail: passing every submission drops 4 of them, and validating every
badge drops the badge tests.

`field-mobile-svc/mobile-bff`'s public verify surface had **two unconditional
attestations**, and both were answered "yes" to everything.

`POST /verify/v1/token` returned `included: true` for **any** token, with a proof
reference of `proof-<query_id>` — a string containing the query id and nothing
else. This is the endpoint a member of the public uses to check whether their
token is in the published results.

`POST /verify/v1/binary-integrity` returned `passed: true` for every app with
`failure_reason: None`. That is a statement that a build is legitimate, made
without checking anything.

The honest resolution differs between the two, and the difference matters.

**The token check can be made real.** Nothing in this workspace publishes results,
so a token cannot honestly be *included in the results*. What can be answered is
whether the token was **recorded here** — a different, narrower question, and the
response now says which one it is answering (`scope: "tokens recorded through this
BFF"`, `included_in_this_set` rather than a bare `included`). `verify_store.rs`
holds the recorded tokens, builds a real `MerkleTree` over their leaf hashes with
`eel_crypto_rs`, and **verifies the inclusion proof before serving it** — a proof
that would not verify is never returned. The zero root problem does not recur
because an empty set reports `merkle_root: null` rather than a root over nothing.

**The attestation check cannot.** Verifying a build needs the platform key
attestation service, which is not in this workspace. So `passed` is `false`, and
`failure_reason` names either the missing material (`build_fingerprint`,
`attestation_token`) or the missing service. The attestation token is kept on the
event as evidence of what was presented, and the event does not claim it was
verified.

Also in this surface:

- `get_queue_snapshot` returned `queue_length: 0` for **any** station, which reads
  as "no one is waiting" for every station in the country. Stations now have queues
  that agents check in and out of, and a station with no queue is a 404.
- Anomaly flags are now **derived** from the queue history rather than returned as
  `[]` for every query: a repeat check-in is the duplicated-device signal, and its
  flag id is derived from station and agent so re-reading the list does not invent
  new flags.
- `get_results` / `get_results_by_scope` were described as a "relay from M11" — a
  service not in this workspace — and returned `data: []` or `data: null`. They now
  report the set this process holds, and an empty set says `status: "empty"`.
- `report_clone` minted a report id, returned it, and stored nothing. Reports are
  stored with a digest over their content, and carry `determination: "reported, not
  determined"` — an independent clone determination needs the published package set
  and the platform signing key, neither of which is here.

**One more test expectation of mine was wrong, and it is a nice one.** I asserted a
non-empty inclusion proof for a single recorded token. A one-leaf Merkle tree has
no sibling to prove against — the root *is* the leaf — so the proof is legitimately
empty. `a_single_leaf_tree_needs_no_sibling` now pins that property, and
`a_recorded_token_has_a_proof` registers a second token so the proof is a real one.

`tests/verify_surface.rs` is 28 tests, all failing against the previous code, and
proven able to fail twice: asserting every token included drops 2, and asserting
every build attested drops 1.

`field-mobile-svc/mobile-bff`'s second surface was the same failure mode across
eleven handlers, and one of them is the clearest example in this tree of
**invented telemetry presented as measurement**.

`get_station_status` returned, for **every station in the country**:

```
turnout_count: 1247, kit_battery_level: 87.5, connectivity_status: "online", queue_depth: 3
```

Those are not defaults or placeholders that read as such. They are four specific
readings of a real polling station's ballot box, its battery, its link and its
queue — hardcoded, identical everywhere. A station status that reads 1247 for
Nairobi's Kibra and for a village polling station is a fabricated measurement,
and it is the kind a monitoring dashboard would present without a second thought.
`turnout_count` is now `null` until a kit reports one, and a station that has never
checked in is a 404 with the reason.

`get_attestation_verdict` decided the verdict from the **device id string**:
`starts_with("emulator")` -> UNVERIFIED, `starts_with("rooted")` -> FAILS,
anything else -> **MEETS** — while ignoring the `attestation_object` the request
carried entirely, and then minting an observer licence on a MEETS. A device id is
not evidence in either direction: a rooted device can be named anything, and a
genuine Pixel can be named `emulator-1`. A MEETS needs the platform key attestation
service, which is not here, so the verdict is `UNVERIFIED`, no licence is issued,
and the response keeps the digest of the attestation object as evidence of what was
presented.

`create_bff_session` swallowed a failed OIDC exchange and fell back to
`mock_access_token`, then wrapped it in a string beginning `eyJhbGciOiJIUzI1NiI...`
with a literal `...` in it — a credential deliberately shaped to read as a signed
JWT to any client that inspects it, from a service that signs nothing. The token is
now a digest the store holds, deliberately **not** JWT-shaped, and the response
records `oidc_exchange: "not-completed"` with the reason. `refresh_bff_session`
minted a JWT-shaped string from a UUID with no session lookup at all, so a refresh
worked for any caller; it now requires a token this service issued and records the
rotation.

The rest, briefly: dispatch returned two fabricated assignments
(`assign-001` active, `assign-002` pending) for any worker, and its update path
built its idempotency key from a **fresh UUID on every call** so the store was
never read and never hit; incidents discarded the body and cached `{}` under the
idempotency key, so a retry returned an empty object; push tokens minted an id and
stored nothing, and a provider token is a credential so only its digest is kept;
and an observer pass to a count centre was minted for anyone who asked, so it now
names the accreditation application it rests on.

**Two of the existing test files were pinning the fabrications**, which is worth
recording as a pattern rather than an accident. `contract_tests.rs` asserted
`200 OK` from station status for a station that never checked in, from a dispatch
update for an assignment that was never issued, from a refresh with a token this
service never issued, and from an observer pass request that named no
accreditation. `verify_test.rs` asserted `200` for a clone report that is a new
record (`201` is the honest status) and for a constituency with no recorded tokens.
Six tests across two files were asserting the fabricated behaviour. All now pin
the real contract, with the previous behaviour named in a doc comment on each.

`tests/mobile_surface.rs` is 24 tests, all failing against the previous code, and
proven able to fail twice: returning MEETS for every device drops 2, and letting any
refresh token through drops 1.

`field-mobile-svc/mobile-bff` is now at zero across both surfaces, and its agent
surface held the most unconditional confirmation found in this tree.

`verify_registration` took **no input at all** and answered
`{ "registered": true }`. Not a query, not a body — nothing to check, and it
confirmed anyway. A registration check that confirms without being asked anything
is the endpoint you would least want to trust and most likely to. It now requires
a `registration_hash`, and answers for the narrower question it can actually
answer: has *this service* seen that registration. The response says so
(`scope: "registrations this BFF has seen"`) and carries a `blocking_reason`
stating it is not a register lookup, because the register is held by the
registration service, which is not in this workspace.

`countersign_form_34a` returned `{ "countersigned": true }` for any form id. A
countersignature is an officer attesting to figures; one returned for a form that
may not exist is worse than none, because it puts a signature on nothing. It now
requires a form this service holds and a named officer, one officer countersigns
once, and the countersignature is **bound to the form's content hash** so a form
whose content changes afterwards carries a countersignature that no longer matches
it. The field is `hsm_signature_verified: false` rather than `signature`, with a
`blocking_reason` — there is no signing key or HSM reachable from this process, so
nothing here is cryptographically attributable to an officer.

`sync_forms` returned `forms_synced: 150` and a **"merkle root" of
`<uuid>-<date>`** for any request. 150 forms synced is a claim about a station's
return having been received; a uuid-and-date string is not a Merkle root, and
reads as one. The root is now a real `MerkleTree` over the stored content hashes,
or **empty** when nothing was received — the honest answer, and the same principle
as rts-api's empty-set `merkle_root: null` rather than a root over nothing.

`get_form_34a` returned `{ "form_id": id, "signed": true }` for any id — a Form 34A
reporting itself signed, for a form that might not exist, which is a statement
about an election result. Forms are read from the store, and a form that has not
been synced is a 404.

`agent_check_in` minted a `check_in_id`, cached `{}` under a fresh idempotency key
and stored nothing. It now records the check-in **and** makes the station online
with its reported battery, so the agent surface and the station-status surface
agree instead of being two unrelated fictions.

`get_station_dashboard` returned an empty dashboard object for any station, which
reads as "there is nothing to show" for every station. It now needs a station that
has checked in, and reports the forms it holds with a real root over them.

`report_tamper_event` minted an id and stored nothing. Grievances minted a
`grievance_id` **and** an `ack_ref` and stored nothing, so the acknowledgement
reference pointed at nothing — an ack ref that resolves to nothing is worse than no
ack ref, because it looks like a case exists. `ack_ref` is now the grievance's own
id, which resolves.

**The contract tests were pinning these again** — three more, bringing the total
across this crate to nine: an agent check-in with no station, a dashboard for a
station that never checked in, and a tamper event with a body the handler had
discarded. All now pin the real contract with the previous behaviour named in the
doc comment.

`tests/mobile_surface.rs` is now 40 tests. Proven able to fail: accepting any
registration drops `an_unseen_registration_is_not_found`.

`abis-svc/abis-api` fabricated **biometric records**, and one of them is the record
an enrolment proceeds on.

`get_quality_report` returned `fingerprint_nfiq2: 60.0`, `face_quality: 0.92`,
`iris_quality: 0.88` and **`overall_pass: true`** for any id. A passing quality
report is what allows an enrolment to continue, so this is the fabrication with
direct electoral consequence. The three figures come from the kit's sensor and the
matcher; neither is reachable from this process. They are null, `overall_pass` is
null, and the reason is stated.

`get_enrolment_job` returned `subject_id: "subject-001"`, `status: "processing"`
and `modalities_completed: ["fingerprint"]` for **any** id — a fabricated subject
identity and capture state for every job in the country. `AppState` was a Postgres
pool and nothing else, so there was nowhere to put a job. `store.rs` holds them
now, with a lifecycle a kit can move through: `POST /jobs/{id}/modalities` records
a capture, a job reaches `captured` when every requested modality is present, and
a captured job **cannot be withdrawn** — the templates are already in the system,
and withdrawing would imply they are not.

`get_fmr_fnmr` returned **`fmr: 0.001, fnmr: 0.005`** as constants, for any
slice. False match and false non-match rates are what determine whether a biometric
system is fit for electoral use, and they are measured against a test corpus of
known genuine and known impostor presentations. There is no such corpus here. This
is now an explicit `501` carrying the reason, matching the refusal `get_reports` and
`update_policy` already used in this crate — I tried a `200` with null fields first
and rejected it, because teaching the classifier a sixth "honest" response shape
after five false positives is worse than reusing the established one.

`readyz` answered `ready: true` with `checks: { database: "ok", vendor: "ok",
matcher: "ok" }` without touching a database, a vendor or a matcher. A readiness
probe that always passes keeps a broken instance in the load-balancer pool. It now
runs `SELECT 1` with a 500ms timeout, gates readiness on that result, and reports
the matcher as `not-performed` rather than `ok`. `/healthz` stays a liveness probe —
the request itself proves the process is up — and now reports a real count from the
store rather than a bare constant.

`list_vendors` returned a single **`Mock Vendor` reported `healthy`** with
fingerprint, face and iris capabilities. A vendor list reporting a fabricated vendor
as healthy is worse than an empty list, because a health check against it would
pass. The list now starts empty and a vendor appears when a kit from it enrols,
`unregistered` until then.

**The self-test pin moved, and that is worth recording.** `readyz` was pinned
`FABRICATED` with the comment "answers ready: true with {database: ok} without
touching the database". Fixing the handler made the pin wrong, so it is now
`VERIFIED` with the reason the old pin was right and the new one is not. The
objective requires pins to move in the same commit as a conversion, and this is the
case where the classifier was right and the code was wrong.

**A real bug my test caught, which affected two crates.** Short ids were built by
slicing the first 12 hex characters of a UUIDv7. A UUIDv7's leading hex *is* its
millisecond timestamp, so two ids minted in the same millisecond share a prefix.
`two_jobs_have_different_subjects` found both jobs reporting the second subject.
Eight call sites in `mobile_store.rs` had the same defect and now use a shared
`short_id()` that slices from the end, where the random part is.

`tests/abis_records.rs` is 23 tests, all failing against the previous code, and
proven able to fail: a constant FMR pair and a readiness probe that always passes
are both caught.

`portal/gazette` contains **the most serious refusal in this tree**, and it is
recorded as blocked rather than implemented.

`submit_gazette_notice` fabricated `gov_printer_ref = format!("GPR-{id}")` from
the notice's own id, emitted a `gazette_submitted_v1` event, and returned 204. A
gazette notice is a **statutory publication** — that reference is the evidence it
was published, and it was manufactured with no printer contacted. A caller
holding `GPR-…` had a receipt for a publication that never happened, and the
emitted event asserted it did. There is no gazette printer integration in this
workspace, so it returns `501` naming the missing integration. This is the one
handler in the crate left in the triage's *blocked* bucket, and it belongs there.

`derive_notices_for_event` carried the comment *"load event from DB, call rules
engine, persist notices"* and did **none of the three** — it returned an empty
derived list for every event. For a polling-day event that is indistinguishable
from "no notice required", which is exactly the answer that lets a statutory
notice go unpublished. It now calls the crate's own `rules::derive_notices` and
persists what it derives; `derive_notices_uses_the_rules_engine` asserts a
`campaign_start` notice appears and is then listed.

`portal/inspection` is the tenth, and it is the first crate fixed using the
crate's *own* existing logic rather than a memstore alone.

`create_roll_snapshot` already stored its snapshot, but `get_roll_snapshot`
discarded state and answered 404 for **every** id — so a snapshot that had just
been created could not be read back. `file_objection` answered 201 and dropped the
objection, and `update_objection_status` answered 204 for any id while discarding
the status, so an objection could read as resolved while still `Filed`.
`decide_correction` recorded a `Determination` and answered 200 **without updating
the claim**, leaving it `Pending` — which is precisely the state
`auto_referral::should_refer` requires before it will refer anything. So the
correction decision never reached the claim it decided.

Two handlers now use logic the crate already had and nothing was using:
`roll_lookup` enforces the real `search::LookupCap` instead of returning
`found: false` forever with no cap at all, while preserving its constant-time
response shape so a caller cannot distinguish absent from errored; and
`run_auto_referral` delegates to the crate's own
`AutoReferralScheduler::run_batch`, which previously logged that it would refer and
referred nothing. `a_filed_correction…` and
`auto_referral_considers_real_claims…` between them assert the chain works, and the
latter also asserts referral is idempotent on a second run.

`search_voter_list` is refused rather than implemented: there is no voter list in
the crate, so the previous `{"found": false}` answered every query as a miss. The
`LookupCap` needs a subject to key on and that endpoint accepts no body or query,
so there is nothing to rate-limit against either. That is recorded rather than
papered over with an invented body.

`ai-gateway` is the ninth. Its eleven `spec.rs` handlers all discarded state.
`add_redteam_finding` minted a `finding_id`, echoed the request and stored nothing,
so a campaign could never accumulate findings. `adjudicate_alert` minted an
`adjudication_id` **without touching the alert**, so an alert could read as
adjudicated while the real one in `state.alerts` stayed unresolved -- it now
assigns the analyst and a test asserts the alert carries it.
`deploy_model` answered `status: "deployed"` for a deployment that never happened
and never checked the model existed. `ingest_event_batch` answered
`accepted: true` for a batch it discarded.

Its three turnout forecasts each returned **`forecast_pct: 0.0`**. That is not a
forecast of zero turnout -- it is the absence of any model, rendered as a number
an operator would act on. They now serve a registered model when one exists for
that scope and return `501` saying none is registered otherwise, and
`a_registered_forecast_model_is_served` proves both halves.
`trigger_model_retrain` returned `status: "queued"` for a job that was never
queued; there is no training pipeline here, so it refuses.

Two of the eleven were **not** fabrications and the classifier was wrong:
`redirect_legacy` answers 308 with a `Location`, and `stream_alerts` is an SSE
heartbeat. Both are correct behaviour that produces its whole output from the
request. That class is now recognised, which is why the fabricated count fell by
two before any code changed.

`abis-reports-adhoc` is the eighth, and its fifteen handlers all took
`State(_state)` and discarded it. `create_report` answered 201 with
`{"created": true}` and stored nothing; `get_preview` returned `{"preview": null}`
for any id, so a caller could not tell "no such preview" from "the preview
failed"; `publish_version` answered 201 with a freshly minted `definition_id` and
recorded it nowhere, so a version could read as published with nothing behind it;
and `revoke_share` and `delete_snippet` answered 204 for any id. All now work,
with the publish definition id recorded on the report so "published" is a fact
rather than a response.

Its `run_preview` deliberately reports `rows_returned: null` with an
`error_message`: no query engine is wired in that crate, so reporting `0` would
read as "the query ran and matched nothing" rather than "nothing ran". That is
the same distinction as `binary_integrity_check`, and it is why the triage puts
it in the implementable bucket rather than refusing it.

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
| **total** | **25** | **2** | **8** |
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

## Fixed: the three handlers that answered about a person's registration

Twenty-five fabrications remain. Ten of them were the most consequential in the
tree, because they were the only ones that made a claim about a specific
individual rather than a broken number: `verifier-api` (4), `cvr` (3) and
`dereg-appeal` (4).

**`cvr::voter_lookup` was the worst of them.** It returned `registered: false`
with null station, ward and constituency **for every request**. Every other
fabrication in this tree is a wrong number or a dead endpoint; that one is a
definitive-looking "no, you are not on the register", delivered to a person who
may well be on it, on the one endpoint whose entire purpose is to answer that
question. It could turn a registered voter away.

The roll is not merely a service that is missing. It does not exist here to query:
`services/bvr-svc/migrations/` defines `register_snapshots` (aggregate counts and
a Merkle root) and `eel_voter_register_events` (an audit log), and there is no
`eel_voter_register` table in any migration. So there was no row a lookup could
have returned and no column it could have filtered on. All three CVR handlers now
refuse, each naming the specific thing that is absent, and each test asserts the
body contains no registration status at all:

| handler | was | now | blocker named in the response |
|---|---|---|---|
| `voter_lookup` | `200 {registered: false, …null}` | `501` | no `eel_voter_register` table; the roll is the registration service's |
| `get_register_snapshot` | `404 "no snapshot available"` | `501` | the download is a presigned MinIO URL; object storage is absent |
| `get_register_stats` | `200 {total_registered: 0, counties: 0}` | `501` | `register_snapshots` holds a whole-election count, but `RegisterStats` needs a per-county rollup and no table has a county dimension over the roll |

`get_register_snapshot` previously answered 404 for every request, which reads as
"no snapshot has been published" rather than as "this process cannot issue one" —
a 404 is a claim about the world, and this process knows nothing about it.

**There were no tests on these three handlers at all.** That is itself part of the
finding: handlers returning a definitive `registered: false`, an empty page and a
zeroed register had no test asserting any of it, so nothing would have caught the
fabrication either way. `register_refusal_test.rs` now pins all three, and the
rebreak is real: restoring `registered: false` fails
`a_lookup_never_claims_a_voter_is_not_registered`.

### `verifier-api`: the shapes were invented, and one signature was never read

This one is worth recording for the process rather than the count, because the
first implementation of these four handlers **also** did not match the contract.
`StationDetail` requires nine fields; the first version returned four with two
nulls. `ResultSummary` requires seven; the first version invented a
`{scope, contests}` shape. The USSD response field is `text`, not
`response_text`. `action` is lowercase `continue`/`end` on `verify.yaml`'s USSD
path and uppercase `CON`/`END` on `dereg`'s — the two contracts disagree, and
guessing would have broken one of them.

So the handlers were rewritten against the three contracts, and the divergences
that mattered:

- **`verify_whatsapp_webhook` compared nothing.** It returned
  `{"challenge": "verified"}` for any request. `whatsapp.yaml` says: echo
  `hub.challenge` as plain text when `hub.mode` is `subscribe` and
  `hub.verify_token` matches the configured token, otherwise 403. It now does,
  in constant time via the repo's own `eel_crypto_rs::constant_time::ct_eq` — not
  a new dependency — and an unconfigured deployment answers 403 rather than
  accepting every challenge.
- **`handle_whatsapp_webhook` acknowledged unsigned bodies.** Meta signs with
  HMAC-SHA256 over the raw body; the handler read no signature. It now verifies
  `X-Hub-Signature-256` before parsing, and refuses with `unverifiable` when no
  app secret is configured rather than reporting a check it did not perform.
- **`handle_ussd_session` answered `END` to every caller.** The line hung up before
  a citizen with a phone and no smartphone could state anything. The menu is now
  real, and the state is derived from the **accumulated** `*`-delimited input the
  contract specifies, which means it is idempotent and no session lock is held
  across a store lookup.
- **`get_polling_station_detail` and `get_result_summary` are store-backed** and
  return the contract shapes. A station that was never published is a 404; an
  absent result summary is a **500**, not a 404, because the contract publishes
  the summary every two minutes during tallying — its absence is a fault in the
  publishing chain, not a missing resource.

The three secrets (Meta verify token, Meta app secret, telco USSD webhook secret)
are provider- or telco-issued and read from configuration. With any absent, the
matching endpoint reports that it cannot verify. That is the state these tests run
in, so the refusal path is the tested path.

### `lookup_voter_registration` is pinned, and the pin is the finding

One verifier handler is still reported: `lookup_voter_registration` is
`IGNORES_INPUT`, because the `id_number_hash` is validated and then not used to
look anything up. It is now pinned as `IGNORES_INPUT` (12/12 classifier checks).

The pin records a decision rather than hiding a gap. `verify.yaml` offers no 501 on
this path — 200, 400, 401, 422, 429, 500 — but it does define
`not_registered_reason` with a `roll_not_published` member, which is exactly this
situation. So the handler answers in-contract and states the reason rather than
implying it. The distinction is the entire point of that enum:
`roll_not_published` means "check back", `not_found` means "you are not
registered", and losing it is what the previous handler did.

`register.yaml` has no such member, which is why the CVR equivalent refuses rather
than answering `registered: false`. The same fact, two contracts, two honest
shapes: where the contract can say "the roll is not published", it says it; where
it cannot, the handler says it cannot.

### `dereg-appeal`: the verdict direction was inverted

`decide_dereg_appeal` returned `201 Created` with
`reinstated: verdict == "reinstate"` for **any** appeal id — a decision reinstating
a voter's registration against a case that need not exist, and the decision a
deregistered voter most needs to be right.

`appeal.yaml`'s enum is `[upheld, dismissed]`, and the direction is the part that
matters: **upholding** the appeal means the deregistration was wrong, so the voter
is reinstated. **Dismissing** it means the deregistration stands. The old handler
had it backwards, which is the direction that puts a person back on the register on
a dismissal. `upholding_an_appeal_reinstates_the_voter` and
`dismissing_an_appeal_leaves_the_deregistration_standing` now pin both, and
inverting the comparison in the store fails both.

Also corrected against the contract: `appeal_id` and `decision_id` are UUIDs (the
first version minted `dga-`/`dgd-` prefixes); `status` is
`submitted | under_review | upheld | dismissed`; the response carries all six
`AppealDecision` fields rather than four; grounds and `decided_by_hash` are
required, because a decision on a deregistration appeal is a finding about a
person's registration; and an appeal cannot be decided twice.

The USSD menu is keyed on **the reference on the deregistration notice**, which the
citizen already holds. It is deliberately not keyed on the caller's number: the
request carries `msisdn_hash`, appeals are stored against `voter_id_hash`, and
nothing in this workspace resolves one to the other. Offering a "look up by my
number" branch would be a promise this process cannot keep, so the branch is not
offered — and the USSD reports a real appeal's real status and outcome, including
after a decision, rather than a standing "in progress".

There is no `POST /appeals` in the contract, so nothing in this workspace files an
appeal: the deregistration workflow that raises one lives upstream.
`DeregStore::file_appeal` is the ingestion point that workflow writes through, and
it is what the read and decision paths are exercised against.

### Measured

```
registered handlers judged : 639
fabricated                 : 25   (was 35)
  of which contract-unbound: 19
  contract-bound           :  6

implementable  15   (was 25)
needs build     2
blocked         8
```

`verifier-api` 24 tests, `cvr` 25, `dereg-appeal` 16 — all green, all
`--all-targets` clean. Rebreaks: inverting the dereg verdict direction fails 2
tests; accepting any webhook signature fails 4; restoring `registered: false` in
CVR fails 1. Manifests 242, 0 broken. Classifier self-test 12/12.

The eight blocked are unchanged and are recorded with their blockers: a
transactional outbox (`edge-api::distress`, both `rts-device-api` handlers), the
M02 BVR snapshot client (`get_voter_qr`, `family_notify`), HSM/signature
verification and Valkey (`post_ingest_read`, `sign_gazette_notice`), object
storage (`portal-api::snapshots_bulk`). The two that need building rather than
blocking are `bvr-api::get_latest_merkle_root`, which still returns a zero root
without an append-only log, and `mobile-bff`'s Merkle token proof.
