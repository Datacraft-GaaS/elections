# Anti-Rigging Capabilities

This document establishes, from code review, the anti-rigging (election-integrity) capabilities actually built into the eel platform. Every claim is cited to its implementation; capabilities are graded **IMPLEMENTED** / **PARTIAL** / **STUB** based on what the code does today, and each section ends with any caveats that qualify the guarantee. Companion volumes: [architecture.md](architecture.md), [security.md](security.md), [anti-rigging-opportunities.md](anti-rigging-opportunities.md) (wave 1: ten further controls — all implemented), and [anti-rigging-next.md](anti-rigging-next.md) (wave 2: identified future opportunities).

Rigging threats addressed: ballot stuffing; multiple voting; retroactive alteration of results; tally drift or arithmetic manipulation; unauthorised early/late declaration; single-officer override of results at a station; tampering with physical materials (ballot boxes, RFID covers, seals); compromised kits; silent replacement of the published tally; equivocation (showing different results to different parties); loss of auditability after the fact.

---

## 1. One-person-one-vote: biometric deduplication and write-level uniqueness

**IMPLEMENTED.**

- Every identification event lands in `identification_events` with a **partial unique index that permits at most one *matched* outcome per voter per election**:
  `services/bvi-svc/migrations/V3__identification_events.sql:24-28` — `CREATE UNIQUE INDEX identification_events_voter_election_uq ON identification_events (voter_id, election_id) WHERE outcome = 'matched'`. Even a buggy or colluding kit cannot record two successful identifications for the same voter.
- The application layer enforces the same rule before the database: `services/bvi-svc/crates/bvi-api/src/store.rs:27` (`StoreError::AlreadyIdentified`) raised by `record_identification` (`store.rs:260`), surfaced to callers as an RFC 9457 conflict via `ApiError::already_identified` (`services/bvi-svc/crates/bvi-api/src/error.rs:74`).
- Event identity deduplication: `event_id UUID NOT NULL UNIQUE` on the same table (`V3__identification_events.sql:5`) prevents replay of a captured identification submission.
- ABIS match outcomes are a closed domain — `matched / no_match / fallback_used / ambiguous` (`services/bvi-svc/crates/bvi-api/src/store.rs:42-43`) — so a kit cannot invent an ad-hoc "pass" verdict; only `matched` rows satisfy the uniqueness constraint.
- Citizen-facing lookups are enumeration-defended: identical payload shape for found/not-found plus a minimum 300 ms + jitter response window (`services/bvi-svc/ex/bvv_web/lib/bvv_web/live/lookup_live.ex:11,26-31`), and **every lookup, found or not, is audit-logged with hashed actor/IP context** (`services/bvi-svc/crates/bvi-api/src/store.rs:583-587`, `routes/voters.rs:47-50` `LookupAuditCtx`), making bulk-enumeration probing attributable.

## 2. Ballot material control: ballot-serial reconciliation

**IMPLEMENTED** (pure logic + API in `services/rts-svc/crates/ballot-serial/`).

- The assigned-serial arithmetic is reconciled at close of poll: `unused = (high − low + 1) − deposited − spoilt`, with **any mismatch raising an explicit hold that is never silent** — FR-M63-023/024/032 are encoded in `services/rts-svc/crates/ballot-serial/src/reconciliation.rs:46` (`reconcile`) returning `ReconciliationStatus::Held { mismatch }`.
- Serial-range membership checks (`aggregate.rs:66` `in_range`) prevent out-of-range serials entering the stream; station-level reconciliation API at `api.rs:198` (`router`).
- Property tests pin the boundary behaviour (zero spoilt, all spoilt, all unused) per FR-M63-032.

## 3. Results-capture integrity: signed, event-sourced Form 34A/34B/34C

**IMPLEMENTED.**

- Results are **event-sourced** (ADR-0013): Form 34B/34C totals are derived exclusively by folding immutable 34A events — no mutable running totals to edit: `services/rts-svc/crates/rts-tally/src/aggregator.rs:9` (`aggregate_34b`), `aggregator.rs:37` (`aggregate_34c`). Station compilation de-duplicates streams (`compiled_stations` check, `aggregator.rs:15-17`).
- Publication events carry a **real Ed25519 producer signature over canonical bytes**, produced inside the caller's transaction and emitted through the transactional outbox so anchoring is atomic with the business write: `services/rts-svc/crates/rts-tally/src/tlog_client.rs:105-113` (`build_anchor_envelope`: `eel_signature = "ed25519:<base64>"` over `canonical_bytes_for_signing()`), and `tlog_client.rs:170-193` (`append_leaf`: outbox emit + commit). Signing keys come from `EEL_TLOG_SIGNING_KEY`; key identity from `EEL_TLOG_KEY_ID` defaulting to the workload SVID (`tlog_client.rs:198-205` `from_env`).
- Each anchor carries a SHA-256 `data_hash` of the leaf payload so downstream verifiers can recompute it against the published bundle (`tlog_client.rs:82-85`).
- Party-agent countersignature on published forms is supported as an explicit event type (`agent_countersign.v1`, endpoint `POST /api/v1/rts/ro/agents/countersign`, `services/rts-svc/crates/rts-ro-api/src/handlers.rs:135-158`), and polling-station session recordings are hash-anchored to the transparency log every 60 s (`handlers.rs:160+` `session_recording`).

## 4. Tally integrity: drift detection, deterministic declaration rules, BFT consensus

### 4.1 Tally drift detection — IMPLEMENTED
Independent aggregators' county tallies are cross-compared; the first divergence between any aggregator and the reference produces a `DriftReport { aggregator_id, candidate_id, expected, actual }`: `services/rts-svc/crates/rts-tally/src/drift.rs:20` (`DriftDetector::record_tally`). A compromised or faulty aggregation path cannot silently diverge.

### 4.2 Deterministic declaration mathematics — IMPLEMENTED
Presidential declaration is computed, not decided: 50%+1 majority, ≥25% in ≥24 counties, runoff and tie outcomes are derived by pure functions with property tests pinning boundary cases (exactly 24 vs 23 counties, exactly/at/below 25%): `services/rts-svc/crates/declaration/src/rules.rs:60-78` (`reaches_county_threshold`, `county_threshold_count`, `evaluate`), proptest rules in `declaration/src/proptest_rules.rs`. No operator input can flip the outcome arithmetic.

### 4.3 BFT quorum certificates on tally blocks — IMPLEMENTED (library)
HotStuff-style consensus over tally state transitions: a `QuorumCertificate` requires **2f+1 (= QUORUM = 3) Ed25519 signatures over the canonical `(msg_type, view_number, block_hash)` bytes**, verified against replica public keys, with explicit `InsufficientSignatures` failure: `services/rts-svc/crates/bft-tally/src/certificate.rs:34-99` (`from_votes`, `verify`). View ordering prevents rollback to older views (`certificate.rs:109` `is_higher_than`); pacemaker, persistence, and replica loop in `pacemaker.rs`, `persistence.rs`, `replica.rs`.

### 4.4 ZK / FHE tally — IMPLEMENTED (libraries, verified by test suites)
- `services/rts-svc/crates/zk-tally/` implements a **Groth16 SNARK over BLS12-381** proving that the sum of station-level 34A vote counts equals the published national tally without revealing station tallies (`src/lib.rs`, circuit/prover/verifier/setup modules; 20 passing tests including proof round-trips). Note the crates live outside the rts-svc sub-workspace but are members of the root workspace.
- `fhe-tally/` (encrypt leaf → homomorphic aggregate → threshold decrypt → prove) and `bft-tally/` carry full passing test suites as well.

## 5. Public verifiability: the append-only transparency log

**IMPLEMENTED core; signing chain PARTIAL (WP-2.2 pending).** Service root: `services/transparency-svc/`.

- **Append-only storage**: leaves are inserted with sequence-assigned indexes under `UNIQUE (index)` and never updated or deleted (`services/transparency-svc/migrations/V1__initial_schema.sql:5-19`); the sequencer hashes each canonical event (producer SPIFFE ID + timestamp + sequence included) into BLAKE3 leaves — `crates/tlog-domain/src/leaf.rs:16` (`LeafFormat::hash`). Retroactive edits to any consumed event are detectable because they change the Merkle root.
- **Merkle proofs for anyone**: inclusion proofs locate a leaf by hash and return the sibling path plus root, verifiable offline against the signed tree head via constant-time root comparison — REST `crates/tlog-api/src/rest/proofs.rs:22` (`get_inclusion_proof`), gRPC `crates/tlog-api/src/grpc/mod.rs:70` (`get_proof`), verification primitive `packages/eel-crypto-rs/src/merkle.rs:236` (`verify_inclusion_against_root_leaf`). Consistency proofs prove old roots are prefixes of new ones (`rest/proofs.rs:61`, `merkle.rs:279` `consistency_proof`).
- **Signed tree heads at three cadences**: 1 s / 60 s / 5 min STH production over the full tree (`crates/tlog-sequencer/src/sth_producer.rs:41` `produce_sth`; true Merkle root via `crates/tlog-sequencer/src/proofs.rs:32` `current_root`). Every head is **signed before persistence** over the canonical bytes `tree_size || root_hash || cadence`: PKCS#11 `eel-sth` key (CKM_ECDSA P-384, softHSM in dev, network HSM in production per ADR-0005) or an explicitly configured development Ed25519 key — unsigned placeholder heads are no longer written (`crates/tlog-sequencer/src/signer.rs`, `SthSigner::from_env`; signature + key fingerprint stored in the row).
- **Third-party witnessing**: witnesses fetch and cosign STHs — `crates/tlog-witness/src/handler.rs:32` (`post_cosign`), IAB client signs the length-prefixed root message (`iab_client.rs:64` `cosign_message`, `:74` `sign_sth`); cosignatures are registered per `(sth_id, witness_id)` with UNIQUE constraint (`migrations/V1__initial_schema.sql:61-70`).
- **Split-view (equivocation) detection**: two different roots at the same tree size raise a hard alert — `crates/tlog-witness/src/split_view.rs:20` (`SplitViewDetector::observe` logs `SPLIT VIEW DETECTED`); operators can sweep all witnesses via `tlog-admin` ("Check all witnesses for split views", `crates/tlog-admin/src/main.rs:34`).
- **Atomic anchoring**: publication snapshots reach the log through the transactional outbox → Debezium → Kafka → tlog-ingest path (`crates/rts-tally/src/tlog_client.rs:7-12` module doc), so a result commit and its transparency anchor succeed or fail together.

**Caveats (residual):**
1. STH signatures are verified end-to-end only against the configured signer backend; production trust requires the network-HSM `eel-sth` key and the M-of-N custodian ceremony (`crates/hsm-signer/`) to be exercised at deployment time — softHSM covers development.
2. gRPC is mounted on a separate address (`GRPC_ADDR`, default `0.0.0.0:50051`) alongside REST (`crates/tlog-api/src/main.rs`); both share one `ApiState` and fail startup together if either port cannot bind.

## 6. Independent verification and audits

### 6.1 Risk-limiting audits (RLA) — IMPLEMENTED (statistics engine)
Data model is complete (`AuditRun`, `SampledBallot`, `EntropySource` for public randomness beacons, `AuditManifest` batch structure, witness sign-off records: `services/governance-svc/crates/rla/src/models.rs`), and the API surface exists (`rla/src/handlers.rs`). The statistics engine is now real: **BRAVO** ballot-polling risk-limiting audits via pairwise winner/loser assertions with multiplicative t-statistic updates against the exact-tie null (`rla/src/engine.rs` — `PairwiseAssertion::update_t`, `compute_t_statistic`, `audit_complete`, `should_trigger_recount`), plus deterministic seeded sample drawing over the audit manifest using SHA-256 counter-mode PRNG without replacement (`engine.rs` `draw_sample_positions`, `draw_sample`) so any party can re-derive and verify the draw. Ten unit tests pin the math (clean samples confirm; wrong-winner evidence never does; determinism, duplicate-freeness, bounds).

Caveat: the HTTP handlers in `rla/src/handlers.rs` still return static responses and are not yet backed by a store — the engine is complete but not exposed operationally.

### 6.2 Parallel manual count — IMPLEMENTED (library)
- Statistically stratified sampling of 462 stations (1%) across nine strata summing to 46,229, using Neyman/proportional allocation with largest-remainder distribution and a seeded, reproducible LCG draw without duplicates: `services/governance-svc/crates/parallel-manual-count/src/sample.rs:26` (`neyman_allocate`), `sample.rs:53` (`sample_stations`).
- Manual-vs-platform divergence is computed per candidate per station with absolute and relative measures and station-level worst-case reporting: `parallel-manual-count/src/divergence.rs:30` (`compute_divergence`). Feeds from the same RLA manifest structures (`rla_feed.rs`).
- Exposed as a service: `eel-governance-parallel-manual-count` binary (`src/main.rs`, default port 8097, plan seed from `EEL_PMC_SEED`) serving the sample plan, manual-count submissions (validated against the plan), platform-tally ingest, and live divergence reporting; divergence reports can be forwarded to the RLA service via `rla_feed.rs::submit_to_rla`.

### 6.3 Deterministic recount / reproducibility — IMPLEMENTED
`reproducibility/make-reproduce/scripts/replay-tally.sh` replays a signed evidence bundle's events through `bin/replay-tally-rs` and re-derives the Merkle root independently of the platform (`scripts/replay-tally.sh:3-12`), producing a JSON report; `reproduce.sh` and `multi-arch-verify.sh` re-build artifacts bit-for-bit. Anyone holding the bundle can check that the announced root matches the recorded events — altering results after the fact breaks the replay.

## 7. Physical custody: hash-chained custody ledger and seal binding

**IMPLEMENTED** (library + partial wiring; see caveats).

- Custody transitions form a **BLAKE3 hash chain** — each transition commits `prev_hash` and binds `BLAKE3(prev_hash ‖ canonical_json(this_transition))`: `services/inventory-svc/crates/rfid-cover-custody/src/domain.rs:275-276`. Reordering, dropping, or editing any historical transition breaks every subsequent link.
- Physical seals are bound to kits cryptographically: seal photos are hashed and signed, binding messages are fixed-size digests, and verification recomputes both — `services/inventory-svc/crates/physical-seal-binding/src/seal_photo.rs:36` (`binding_message`), `seal_photo.rs:47` (`compute_photo_hash`), `verify.rs:16` (`verify_kit_signature`), `verify.rs:36` (`verify_bound_event`).
- Tamper events flow operationally to the command centre: the ECC cover-tamper panel subscribes to `eel:inventory:cover_tamper` PubSub and renders/resolves alerts (`services/ecc-svc/ex/ecc_live/lib/ecc_live_web/live/cover_tamper_live.ex:27,71,92`), backed by the telemetry feed in `cover_tamper_live.ex` → ecc-api `/v1/rfid/tamper-alerts`.

**Caveats:** the Django admin now replays the **same BLAKE3 chain** as the Rust writer — `views.py::_verify_chain` recomputes `BLAKE3(prev_hash || canonical_json(transition))` with byte-exact canonical serialisation (`_canonical_transition_bytes`), and the model reflects the real BYTEA columns plus `officer_svid` (`rfid_covers/models.py`). Seal-binding routes remain not fully wired into the served inventory router.

## 8. Kit/device trust: attestation, revocation, anomaly signals

**PARTIAL.**

- **Attestation**: kits present Play Integrity / App Attest / TPM-quote tokens which are verified into tiered verdicts (`MeetsStrong` / `MeetsDevice` / legacy warning), with tampered-token rejection: `services/biometric-edge-svc/crates/kit-attest/src/verify.rs:50` (`verify_attestation`). Caveat: token verification is structural; vendor back-end calls are call-out stubs (flagged in `docs/system/services/biometric-edge.md`).
- **Revocation**: a stolen/compromised kit serial is blacklisted immediately with reason + authorising officer, idempotent per Idempotency-Key: `services/bvi-svc/crates/bvi-api/src/routes/kits.rs` (`revoke_kit`), store enforcement in `store.rs:399` (`PgStore::revoke_kit`).
- **Telemetry anomalies**: device telemetry and distress signals (`services/bvi-svc/crates/bvi-api/src/routes/signals.rs`) feed ECC monitoring; speech-integrity media analysis flags campaign-offence categories `hate_speech | incitement | misinformation` at the schema level (`services/ecc-svc/migrations/V1__speech_integrity.sql:38` CHECK constraint) with content-hash dedup (`url_sha256`, `store.rs` duplicate-media guard).

## 9. Procedural controls in software: Section 44A dual-officer verification

**IMPLEMENTED.**

- A station's result-related confirmation follows a strict two-person state machine (`officer1_confirmed → complete`); officer 2 can only complete a row already in `officer1_confirmed`, enforced identically in Postgres (`WHERE status = 'officer1_confirmed'` UPDATE ... RETURNING) and the in-memory fake: `services/ecc-svc/crates/ecc-api/src/store.rs:565-600` (`complete_s44a_officer2`).
- One confirmation per station: `ON CONFLICT (station_code) DO NOTHING` + explicit conflict error (`store.rs:531-563` `insert_s44a_officer1`, `StoreError::S44aConfirmationExists` → HTTP 409 in `handlers.rs` `store_error_response`).
- Both steps are audited with officer IDs and timestamps (`officer1_id/officer2_id/officer*_confirmed_at` columns, `services/ecc-svc/migrations/V2__s44a_confirmations.sql`), and the operator console drives the same API rather than writing locally (`services/ecc-svc/ex/ecc_live/lib/ecc_live/live/s44a_live.ex:33-46`).

## 10. Cross-cutting process guarantees that anti-rigging rests on

- **Idempotency everywhere (ADR-0011)**: mandatory Idempotency-Key + 24 h snapshot replay (`services/bvi-svc/crates/bvi-api/src/idem.rs:19-65`) prevents double-application of any mutating request — including vote-affecting writes — under retry storms or attacks.
- **Single publish path**: all cross-service state moves through the transactional outbox (ADR-0015; `packages/eel-outbox-rs`), eliminating side-channels where unofficial tallies could diverge from official ones.
- **Hybrid logical clocks** (`packages/eel-hlc-rs`) order events consistently across sites, undermining later-reordering attacks on the event log.
- **Constant-time comparison** of secrets and roots (`packages/eel-crypto-rs/src/constant_time.rs`) removes timing oracles from verification paths.
- **Continuous integrity auditing**: the completeness scanner (`tools/completeness`) fails CI on new stubs/skips, and traceability markers (`packages/eel-trace-rs`) tie tests to anti-rigging requirements (e.g. `#[trace("FR-M10-028")]` on Merkle-root reorder tests, `rts-tally/src/merkle.rs:29-52`; EC-OPS-02 documented on the no-double-ID index, `V3__identification_events.sql:21`).

---

## Capability status matrix

| # | Control | Threat answered | Status | Anchor |
|---|---|---|---|---|
| 1 | Biometric dedup + unique matched-vote index | multiple voting | IMPLEMENTED | `V3__identification_events.sql:24` |
| 2 | Lookup audit + enumeration/timing defence | voter-roll probing | IMPLEMENTED | `routes/voters.rs`, `lookup_live.ex` |
| 3 | Ballot-serial reconciliation holds | ballot stuffing / material loss | IMPLEMENTED | `ballot-serial/src/reconciliation.rs:46` |
| 4 | Signed, event-sourced Form 34A→34C | result alteration | IMPLEMENTED | `rts-tally/src/{aggregator,tlog_client}.rs` |
| 5 | Tally drift detection across aggregators | manipulated aggregation | IMPLEMENTED | `rts-tally/src/drift.rs:20` |
| 6 | Deterministic declaration rules | premature/fake declaration | IMPLEMENTED | `declaration/src/rules.rs:78` |
| 7 | BFT quorum certificates (Ed25519, 2f+1) | compromised tally node | IMPLEMENTED (lib) | `bft-tally/src/certificate.rs:60` |
| 8 | Append-only Merkle log + proofs + witnesses | retroactive tampering, equivocation | IMPLEMENTED | `transparency-svc` crates |
| 9 | STH signing via PKCS#11/softHSM (`eel-sth`, ECDSA P-384; dev Ed25519 fallback) | forged tree heads | IMPLEMENTED | `tlog-sequencer/src/signer.rs`, `sth_producer.rs` |
| 10 | Split-view detection + witness crosscheck | equivocation | IMPLEMENTED | `split_view.rs:20`, `tlog-admin` |
| 11 | Deterministic recount from evidence bundles | post-hoc falsification | IMPLEMENTED | `make-reproduce/scripts/replay-tally.sh` |
| 12 | Stratified parallel manual count | electronic-vs-paper divergence | IMPLEMENTED (service binary) | `parallel-manual-count/src/main.rs`, `sample.rs:53` |
| 13 | Risk-limiting audit statistics (BRAVO + seeded sampling) | residual error quantification | IMPLEMENTED (engine); HTTP handlers pending store backing | `governance-svc/crates/rla/src/engine.rs` |
| 14 | BLAKE3 custody hash chain + seal binding | physical material tampering | IMPLEMENTED (partial wiring) | `rfid-cover-custody/src/domain.rs:276` |
| 15 | Kit attestation + revocation blacklist | compromised/stolen devices | PARTIAL | `kit-attest/src/verify.rs:50`, `kits.rs revoke_kit` |
| 16 | Section 44A dual-officer state machine | single-officer override | IMPLEMENTED | `ecc-api/src/store.rs:565` |
| 17 | ZK / FHE tallying (Groth16 over BLS12-381; FHE aggregate/decrypt/prove) | privacy-preserving verifiable tally | IMPLEMENTED (libraries, full test suites) | `rts-svc/crates/zk-tally/`, `fhe-tally/` |

## Priority gaps to close (in order)

1. Back the RLA service HTTP handlers with a store so the BRAVO engine is operationally reachable (`governance-svc/crates/rla/src/handlers.rs`).
2. Exercise the network-HSM `eel-sth` key path (softHSM covers development) at deployment, including the M-of-N custodian ceremony in `services/transparency-svc/crates/hsm-signer/`.
3. Wire seal-binding routes into the served inventory router.
4. Complete kit-attestation vendor back-end verification calls.
