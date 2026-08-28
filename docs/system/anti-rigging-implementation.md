# Anti-Rigging Implementation Reference

Deep-dive documentation of how rigging prevention, detection, and countermeasure capabilities are implemented across the eel platform. Companion volumes: [anti-rigging.md](anti-rigging.md) (capability baseline), [anti-rigging-opportunities.md](anti-rigging-opportunities.md) (gap analysis, all items now implemented).

Every control is documented with its threat model, mechanism, implementation citations (`file` + function), configuration, verification tests, operational use, and failure behaviour. All cited code is exercised by test suites that pass at the time of writing (45 Rust suites + 19 Python tests across the affected crates).

---

## 1. Defence model

| Objective | Meaning | Controls |
|---|---|---|
| **Prevent** | make the rigged outcome impossible or cryptographically evident | one-voter-one-match index; signed STHs; WORM log tables; M-of-N declaration gate; custody signature enforcement; countersignature verification |
| **Detect** | surface tampering attempts as alarms | split-view gossip; ingest dead-letter; turnout/velocity/burst/digit anomalies; drift detector; ballot-serial holds |
| **Counter** | enable recovery, audit, and independent proof after an attempt | BRAVO RLA engine + audited service; parallel manual count; deterministic recount from bundles; receipt-based public verification |

```
 kits/stations          services                     integrity plane
 ─────────────         ────────────────────          ───────────────────────────────
 identification  ──►   bvi-api        ──┐            uniqueness index (P1)
 ballots/receipts ──►  rts-svc tally  ──┼─ outbox ► tlog-ingest (P2) ► leaves
 custody scans   ──►   inventory      ──┘            officer signatures (P5)
                                                      WORM triggers (H3)
 agents          ──►   rts-ro-api countersign        key-verified (P6)
 commissioners  ─────► declaration gate               M-of-N seal (P7)
 witnesses       ──►   tlog-witness ◄─ gossip        split-view alarms (D1)
 auditors        ──►   rla service ◄─ pmc/beacon     BRAVO + seeds (C1-C3)
 citizens        ──►   portal receipt page           inclusion proofs (C4)
 operators       ──►   ecc alerts ◄─ bvi-anomaly     statistical detectors (D2)
```

---

## 2. Prevention controls

### P1. One-person-one-vote write guarantee

- **Threat:** multiple voting via kit compromise or replay.
- **Mechanism:** a partial UNIQUE index permits at most one `matched` identification per `(voter_id, election_id)`; application-level `AlreadyIdentified` conflict fires first.
- **Implementation:** `services/bvi-svc/migrations/V3__identification_events.sql:24` (`identification_events_voter_election_uq ... WHERE outcome = 'matched'`); `services/bvi-svc/crates/bvi-api/src/store.rs` `record_identification` → `StoreError::AlreadyIdentified`; HTTP conflict via `error.rs` `already_identified`.
- **Failure mode:** second attempt fails closed (409) — never silently merged.

### P2. Log completeness: offsets advance only past durable appends

- **Threat:** silent under-inclusion — results counted internally but absent from the public Merkle log.
- **Mechanism:** the Kafka consumer advances an offset only after `Sequencer::append` reports success. Rate-limited and errored appends are parked with their Kafka coordinates; the offset stays behind them so restart replays them (dedup-safe by `(source_topic, source_event_id)`).
- **Implementation:** `services/transparency-svc/crates/tlog-ingest/src/consumer.rs` (`IngestConsumer::run`, "Integrity invariant" block); dead-letter store `src/failures.rs` (`record_failure:12`, `list_open:55`, `mark_replayed:66`); table `migrations/V2__ingest_failures.sql` (UNIQUE on `(topic, partition, kafka_offset)` prevents duplicate parking).
- **Countermeasure operation:** `tlog-admin ingest-failures --db-url …` lists open rows and exits non-zero when any exist (CI-gateable); operators re-drive via `tlog-admin bulk-ingest` then acknowledge with `ingest-failure-ack --id <uuid>`.
- **Failure mode:** append outage halts offset progress and accumulates visible failures rather than skipping events.

### P3. Signed tree heads before persistence (PKCS#11 / softHSM)

- **Threat:** forged or retroactively edited tree heads.
- **Mechanism:** every STH is signed over canonical bytes `tree_size(u64 BE) ‖ root_hash ‖ cadence` *before* the row is written; no unsigned head ever exists on disk. Signing backends:
  - PKCS#11 `eel-sth` key, CKM_ECDSA P-384 (softHSMv2 in dev, Luna/Utimaco per `EEL_HSM_VENDOR`);
  - explicit development Ed25519 only via `EEL_STH_SIGNER=dev` + `EEL_STH_DEV_SIGNING_KEY` (missing key material is a hard startup error).
- **Implementation:** `services/transparency-svc/crates/tlog-sequencer/src/signer.rs` (`SthSigner::from_env:48`, `sign_sth:96`, `verify_sth:110`, `canonical_sth_bytes:134`); producer `sth_producer.rs` (`produce_sth` signs then inserts signature + fingerprint); sequencer binary wires it at startup (`crates/tlog-sequencer/src/main.rs`).
- **Configuration:** `EEL_STH_SIGNER=pkcs11|dev`, `PKCS11_MODULE_PATH`, `EEL_HSM_SLOT`, `EEL_HSM_PIN`, `EEL_STH_DEV_SIGNING_KEY`.
- **Tests:** signer round-trip, cadence/root tamper rejection (`signer.rs` tests).
- **Failure mode:** signing failure aborts STH production for that tick (logged as error) — the log never publishes an unsigned head.

### P4. WORM retention on the log core

- **Threat:** privileged insiders (DBA path) editing or deleting published evidence.
- **Mechanism:** `BEFORE UPDATE OR DELETE` triggers raise a hard error on `leaves`, `sth`, `witness_cosigs`, `tree_tiles`. The application is insert-only by design; removal requires deliberately dropping a trigger — an auditable act, never silent.
- **Implementation:** `services/transparency-svc/migrations/V3__worm_retention.sql` (`tlog_worm_guard()`).
- **Note:** run after V1/V2 migrations; `ingest_offsets` intentionally excluded (upsert is required).

### P5. Custody-transition officer signatures

- **Threat:** forged chain-of-custody records around ballot containers.
- **Mechanism:** transitions into `SEALED_RETURN | TALLY_ARRIVED | COUNT_COMPLETE | STORAGE_ARCHIVE` must carry an Ed25519 signature over the transition's `this_hash` from an officer key registered in `EEL_CUSTODY_OFFICER_KEYS` (format `officer_uuid:hex_pubkey`, comma-separated). Missing, unregistered, or invalid signatures reject the insert outright.
- **Implementation:** checker `services/inventory-svc/crates/rfid-cover-custody/src/officer_sig.rs` (`is_high_risk:19`, `check_transition_signature:45`, `parse_registry:75`); enforcement inside `repository.rs` `insert_transition` via `officer_registry():136` (OnceLock from env). Because Django replays the identical BLAKE3 chain (`inventory_admin/apps/rfid_covers/views.py::_verify_chain`), any accepted-but-forged row also breaks the replayable chain view.
- **Error surface:** `RfidError::{OfficerSignatureRequired, OfficerUnregistered, OfficerSignatureInvalid}` (`src/error.rs`).
- **Tests:** verified/missing/unregistered/tampered-hash matrix plus registry parsing (`officer_sig.rs` tests).

### P6. Party-agent countersignature verification

- **Threat:** fabricated agent observation records; unattributable objections.
- **Mechanism:** `POST /api/v1/rts/ro/agents/countersign` verifies the hex Ed25519 `agent_signature` against the registered key for `(party_code, agent_id)` over canonical bytes `form_id|agent_id|party_code|signed_at` (`agents.rs::countersign_message:13`). Unregistered agent → 403; malformed encoding → 400; non-verifying signature → 403. Only verified countersigns produce `agent_countersign.v1`.
- **Implementation:** `services/rts-svc/crates/rts-ro-api/src/agents.rs` (`AgentRegistry::from_spec:104`, `verify_countersignature:76`); handler `handlers.rs:138 agent_countersign`; registry loaded once from `EEL_AGENT_KEYS`.
- **Tests:** round-trip accept; wrong form/agent/party/timestamp rejections; garbage-spec failure.

### P7. Dual-control national declaration (M-of-N)

- **Threat:** unilateral premature or fabricated declaration of results.
- **Mechanism:** `DeclarationGate` collects approvals from registered commissioners; each approval is a distinct Ed25519 signature over canonical bytes `election_id|outcome_json|tally_merkle_root|declared_at` (`quorum.rs::canonical_declaration_bytes:20`). Duplicates are rejected; `seal()` refuses below quorum. The resulting `SignedDeclaration` can be independently checked by anyone holding the commissioner registry via `SignedDeclaration::verify:159` — the tally Merkle root is bound into every signature, so approval cannot be replayed against a different tally.
- **Implementation:** `services/rts-svc/crates/declaration/src/quorum.rs` (`DeclarationGate:53`, `approve:91`, `approve_with_signing_key:115`, `seal:127`).
- **Integration point:** wire `seal()` ahead of publishing the final declaration event; the portal/verifier path calls `verify()` before display.
- **Tests:** two-of-three seal+verify; tampered-outcome forgery fails all signatures; unknown/duplicate approvers rejected; foreign registry fails closed.

---

## 3. Detection controls

### D1. Witness gossip and split-view alarms

- **Threat:** equivocation — showing different trees to different observers.
- **Mechanism:** every head observed — submitted for cosigning locally *or* pulled from a peer — flows through `SplitViewDetector`; two different roots at the same tree size produce a retained `Finding`, an `error!` log, and exposure on the findings endpoint.
- **Implementation:**
  - detector `services/transparency-svc/crates/tlog-witness/src/split_view.rs` (`observe_root`, retained `findings`);
  - cosign intake feeds the detector after persisting (`handler.rs post_cosign`);
  - `GET /api/v1/transparency/witnesses/head` exposes this witness's current view;
  - `gossip.rs::poll_peers_once:13` / `spawn_gossip:53` poll `WITNESS_PEERS` heads every `WITNESS_GOSSIP_SECS` (default 5) through the same detector;
  - `GET /api/v1/transparency/witnesses/split-views` returns all findings for operator/IAB consumption.
- **Configuration:** `WITNESS_PEERS` (comma-separated base URLs), `WITNESS_GOSSIP_SECS`.
- **Countermeasure path:** findings feed ECC alert dispatch; `tlog-admin witness-cross-check` remains available for manual sweeps.

### D2. Statistical anomaly detection on election-day data

- **Threat:** ballot stuffing through inflated identifications; kit reuse across locations; coordinated bursts; digit-manipulated totals.
- **Mechanism & implementation** — pure, fully unit-tested functions in `services/bvi-svc/crates/bvi-anomaly/src/lib.rs`, exposed statelessly on bvi-api `POST /api/v1/bvi/anomalies/evaluate` (bearer-gated; `routes/anomalies.rs`):
  - `turnover_over_registered:50` — station identifications exceed registered electors → Critical;
  - `velocity_impossible:72` — same device serial seen at different stations closer together than `min_travel_seconds` allows → High (names both stations);
  - `burst_zscore:111` — identification-rate z-score vs fleet baseline exceeds threshold (stdev==0 safe, <3 windows inert);
  - `digit_distribution_chi_square:157` — chi-square vs expected digit shares with Wilson–Hilferty p-value approximation; uniform data passes, skewed data alarms (tested both ways);
  - `evaluate_station` composes a+b+c per station.
- **Operation:** schedule posts to the evaluate endpoint (or call the library from a worker); route emitted `TurnoutAnomaly`s into the existing ECC alert tables/SSE pipeline for operator triage.

### D3. Tally drift between aggregators

- **Threat:** a compromised aggregation path diverging from the official fold.
- **Implementation:** `services/rts-svc/crates/rts-tally/src/drift.rs` `DriftDetector::record_tally` compares each aggregator's county tallies against the reference and emits `DriftReport{aggregator_id, candidate_id, expected, actual}` on first divergence.

### D4. Ballot-material reconciliation holds

- **Threat:** ballot stuffing or lost papers invisible in accounting.
- **Implementation:** `services/rts-svc/crates/ballot-serial/src/reconciliation.rs::reconcile` — `unused = (high − low + 1) − deposited − spoilt`; any mismatch returns `ReconciliationStatus::Held{mismatch}` ("never silent", FR-M63-024), boundary conditions property-tested.

---

## 4. Counter / audit controls

### C1. BRAVO risk-limiting audits (engine + service)

- **Threat:** residual error/fraud in reported outcomes must be quantifiable and provable.
- **Engine:** `services/governance-svc/crates/rla/src/engine.rs`
  - `PairwiseAssertion` (winner>loser assertion with reported share);
  - `update_t` / `compute_t_statistic` — multiplicative BRAVO likelihood ratio against the exact-tie null (winner ×s/0.5, loser ×(1−s)/0.5, others uninformative);
  - `audit_complete` / `should_trigger_recount` against threshold `1/α`;
  - `draw_sample_positions` / `draw_sample` — deterministic without-replacement sampling over the audit manifest using SHA-256 counter-mode PRNG seeded by the beacon seed; anyone can re-derive and check the draw.
- **Service:** `rla/src/store.rs` (Pg/Mem store enum mirroring the ecc-api pattern) backs rewritten handlers driving the lifecycle: create run (+assertion, manifest, seed, drawn sample) → record interpretations (write-once; duplicates 409) → report (`risk_achieved = 1/T`) → finalise (only when `audit_complete`; otherwise 409 carrying the current T). Mutating routes require a Bearer principal.
- **Tests:** engine math (clean samples confirm at α=0.05; wrong-winner evidence mathematically never confirms; determinism, duplicate-freeness, oversample rejection) and full lifecycle smoke tests including premature-finalise and authz failures.

### C2. Python SHANGRLA twin

- `services/governance-svc/py/shangrla/src/shangrla/engine.py`: same BRAVO semantics for auditor tooling (`PairwiseAssertion.update_t`, `ShangrlaEngine.compute_t_statistic/evaluate_run/draw_sample` without-replacement seeded draw); discrepancy ballots weigh as evidence against the reported outcome.

### C3. Verified randomness beacons and multi-party seed mixing

- **Threat:** cherry-picked audit samples.
- **Implementation:** `py/shangrla/src/shangrla/beacon.py`
  - `BeaconClient.fetch_pulse:160` fetches NIST Beacon v2-shaped pulses over an injectable transport;
  - `verify_signature:182` performs real RSASSA-PKCS1-v1_5/SHA-512 verification over the documented pulse message — fail-closed (raises) without a key or on any mismatch, never silently skipped;
  - `commit_seed:225` / `combine_seeds:247` — parties hash-commit seeds before close (`sha256("eel-seed-commit:"‖seed)`), reveal afterwards; the final seed is `sha256(beacon_output ‖ sorted party reveals)` so neither the authority nor any single party can steer the sample.
- **Tests:** mock-transport parsing, signature accept/tamper/wrong-key/no-key cases, mixing determinism/order-independence/mutation sensitivity, commitment round-trip.

### C4. Parallel manual count divergence

- **Implementation as a service:** `eel-governance-parallel-manual-count` binary (`src/main.rs`, default :8097, plan seed `EEL_PMC_SEED`) exposing `/health`, plan retrieval, manual-count submissions validated against the sample plan, platform-tally ingest, and live divergence reporting (`compute_divergence`); reports forward to the RLA service via `rla_feed.rs::submit_to_rla`.

### C5. Deterministic recount from evidence bundles

- `reproducibility/make-reproduce/scripts/replay-tally.sh` re-derives the Merkle root from bundle events through `replay-tally-rs`, independent of the platform — post-hoc falsification of the announced root breaks the replay.

### C6. Voter-facing station-result receipts

- **Threat:** substitution between the posted paper form and the tallied form.
- **Flow:** capture-side code derives from the anchored payload hash (`rts-tally/src/tlog_client.rs::receipt_code`, first 12 uppercase hex); citizen enters station + code on `packages/eel-web-astro/src/pages/receipt.astro`; backend `tlog-api GET /v1/tlog/receipts/{code}` (`rest/proofs.rs::resolve_prefix:146`, `resolve_receipt` handler) resolves the code to exactly one leaf (0 → 404, >1 → 409 ambiguous) and returns leaf index, inclusion-proof sibling path, and the latest signed STH in one round trip; the page verifies `proof.root_hash == sth.root_hash` client-side and shows MATCH/NO-MATCH. Every mismatch is a citizen-detectable substitution alarm.

---

## 5. Identity hardening supporting the above

### I1. OIDC audience pinning

- `packages/eel-auth-rs/src/jwks.rs` — `JwksVerifier::with_audience(jwks_url, realm, Some(aud))` turns on strict audience validation: `validation.set_audience(&[aud])` plus a Keycloak-aware secondary check accepting `azp == aud` (the `aud` claim accepts string-or-array via `AudValue`). Audience-mismatched tokens are refused, preventing cross-service replay of operator credentials toward result-affecting APIs. Services set their audience from config (e.g. `EEL_OIDC_AUDIENCE=eel-ecc-api`); unpinned construction preserves legacy behaviour during migration.
- **Test:** existing integration suite exercises verifier construction; pinning logic is covered by the claim-mapping paths in the crate.

### I2. ABIS SPIFFE middleware

- `services/abis-svc/crates/abis-api/src/middleware.rs::spire_auth` — when `EEL_ABIS_SPIRE_REQUIRED=true`: recovers the caller identity from sidecar-propagated mTLS headers (`X-Forwarded-Client-Cert` `URI=`/`By=` fields, or direct `X-Client-Spiffe-Id`), parses the eel SPIFFE shape `spiffe://eel.cluster/ns/<ns>/sa/<sa>` (`SpiffeIdentity::parse`), enforces the `eel.cluster` trust domain, and honours namespace allowlisting via `EEL_ABIS_ALLOWED_NAMESPACES` (default allow-list: enrolment, adjudication, admin). Unparseable → 401; wrong domain/namespace → 403. Dev bypass remains explicitly opt-out via leaving the flag unset.

---

## 6. Configuration matrix

| Variable | Control | Purpose |
|---|---|---|
| `EEL_STH_SIGNER` | P3 | `pkcs11` (default) or `dev` |
| `PKCS11_MODULE_PATH` / `EEL_HSM_SLOT` / `EEL_HSM_PIN` | P3 | softHSM/HSM token access |
| `EEL_STH_DEV_SIGNING_KEY` | P3 | hex Ed25519 seed (dev only, mandatory if `dev`) |
| `EEL_AGENT_KEYS` | P6 | `party:agent:hex_pubkey` registry for countersignatures |
| `EEL_CUSTODY_OFFICER_KEYS` | P5 | `officer_uuid:hex_pubkey` registry for custody signatures |
| Declaration gate registry spec | P7 | `id:hex_pubkey,…` + quorum passed to `DeclarationGate::new` |
| `WITNESS_PEERS` / `WITNESS_GOSSIP_SECS` | D1 | peer base URLs / poll interval |
| `EEL_PMC_SEED` | C4 | parallel-manual-count plan seed |
| `EEL_OIDC_AUDIENCE` | I1 | per-service pinned JWT audience |
| `EEL_ABIS_SPIRE_REQUIRED` / `EEL_ABIS_ALLOWED_NAMESPACES` | I2 | enforce SVID posture / namespace ACL |
| `DATABASE_URL` (per service) | P2/C1 | activates Pg stores (dead-letter, RLA persistence) |

Key registries are provisioned per electoral cycle; malformed entries fail loudly at startup or first use (never silently weaken enforcement).

## 7. Test coverage summary

| Area | Tests | Location |
|---|---|---|
| BRAVO math + sampling | 10 unit | `governance-svc/crates/rla/src/engine.rs` |
| RLA lifecycle | 6 smoke | `governance-svc/crates/rla/tests/smoke.rs` |
| Countersignature verify | 3 | `rts-ro-api/src/agents.rs` |
| Custody signature checks | 3 (+47 crate) | `rfid-cover-custody/src/officer_sig.rs` |
| Declaration quorum | 4 (+12 crate) | `declaration/src/quorum.rs` |
| STH signer | 3 (+31 crypto) | `tlog-sequencer/src/signer.rs` |
| Anomaly detectors | 23 | `bvi-anomaly/src/lib.rs` |
| Receipt resolution | 3 (+11 tally) | `tlog-api/src/rest/proofs.rs`, `rts-tally` |
| Beacon client + mixing | 19 (py total) | `py/shangrla/tests/ci/test_beacon.py` |
| Split-view detector | 4 | `tlog-witness/src/split_view.rs` |

Run everything: `cargo test -p <crate>` per the table, or the sweep command in `docs/system/operations.md`.

## 8. Operational checklist (election cycle)

1. **Pre-deployment:** generate cycle keys; register agent/officer/commissioner keys in the env registries; run transparency migrations V1–V3 (V3 installs WORM); configure `WITNESS_PEERS` across witness fleet; pin `EEL_OIDC_AUDIENCE` per service; set `EEL_ABIS_SPIRE_REQUIRED=true` outside dev.
2. **Poll-open:** confirm `tlog-admin checkpoint` shows signed latest STH (non-placeholder fingerprint); witness gossip producing observations.
3. **During poll:** monitor `POST /bvi/anomalies/evaluate` outputs feeding ECC; watch `tlog-admin ingest-failures` (any open row = page).
4. **Close:** reconcile ballot serials (holds block progression); publish parallel-manual-count plan (seed = beacon + party commitments); collect manual submissions; review divergence report.
5. **Post-election:** run BRAVO audit to finalisation; seal declaration through M-of-N gate; verify sealed declaration with `SignedDeclaration::verify` before publication; citizens verify receipts; independent parties replay `replay-tally.sh`.

## 9. Residual limitations

- HSM-backed STH/declaration signing uses softHSM in development; production trust requires exercising network-HSM keys and the M-of-N custodian ceremony (`transparency-svc/crates/hsm-signer/`) at deployment.
- The beacon signature check verifies the documented NIST-v2-style message concatenation with an injectable key; confirm the exact profile string against the production beacon before go-live.
- Pg arms of the RLA store target assumed table shapes; run the accompanying migration set (`governance-svc/migrations/`) before enabling Pg mode.
- ABIS match-route integration tests require a live database (idempotency-backed handlers) and are marked `#[ignore]` accordingly.
