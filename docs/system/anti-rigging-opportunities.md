# Anti-Rigging Opportunities: Ten Further Controls

Companion to [anti-rigging.md](anti-rigging.md). This volume identifies ten further opportunities to identify, prevent, or counter rigging beyond the baseline capabilities, plus three honourable mentions.

**STATUS: ALL TEN OPPORTUNITIES AND ALL THREE HONOURABLE MENTIONS ARE NOW IMPLEMENTED** (verified by test suites; per-section *Implementation* citations added). The original gap analysis is retained for context. Wave-2 candidates (not yet built) are in [anti-rigging-next.md](anti-rigging-next.md).

---

## 1. Cryptographically verify party-agent countersignatures — IMPLEMENTED


> **Implementation:** `rts-svc/crates/rts-ro-api/src/agents.rs` — `AgentRegistry` from `EEL_AGENT_KEYS`; `verify_countersignature` over canonical `form|agent|party|signed_at`; handler rejects unregistered agents and non-verifying signatures with 403.

**Threat:** fabricated evidence that party agents witnessed results; agents' objections recorded but deniable later.

**Current gap:** `POST /v1/rts/ro/agents/countersign` logs and accepts anything — no key, no signature, no binding to the form's content (`services/rts-svc/crates/rts-ro-api/src/handlers.rs:135-158`).

**Existing hooks:** the platform already has the exact pattern to copy: producer signatures over canonical bytes (`services/rts-svc/crates/rts-tally/src/tlog_client.rs:105-113`) and Ed25519 verification primitives (`packages/eel-crypto-rs/src/ed25519.rs`).

**Implementation path:** register per-party agent verification keys (SpiceDB relation `agent` on `Election/#`); require `{agent_id, party_code, signature}` where `signature = Ed25519(agent_key, canonical_bytes(form_id, form_hash))`; verify at intake, reject or flag mismatch, emit `agent_countersign.v1` carrying the verified key id so the countersignature is independently checkable from the event stream.

## 2. Stop advancing ingest offsets past failed appends — IMPLEMENTED


> **Implementation:** `tlog-ingest/src/consumer.rs` advances offsets only after a successful append; failures park in `tlog_ingest_failures` (`migrations/V2__ingest_failures.sql`, `src/failures.rs`); triage via `tlog-admin ingest-failures` / `ingest-failure-ack --id`.

**Threat:** silent under-inclusion — a result event can fail to enter the public transparency log while the platform's internal tally still counts it, producing an unverifiable official outcome.

**Current gap:** `tlog-ingest` logs append failures but then saves the Kafka offset regardless, so failed appends are never retried (`services/transparency-svc/crates/tlog-ingest/src/consumer.rs:96-116` — both `Ok(None)` rate-limit and `Err(e)` branches fall through to `offset_tracker::save_offset`).

**Existing hooks:** the schema already tracks offsets per partition (`migrations/V1__initial_schema.sql:73-79`); the sequencer returns typed errors distinguishable per cause (`crates/tlog-sequencer/src/lib.rs` `SequencerError`).

**Implementation path:** advance the offset only on success; park failures in a dead-letter table (`tlog_ingest_failures`); add a reconciliation job that compares per-topic message counts against `leaves` rows per domain and raises an ECC alert on drift. This converts under-inclusion from silent to alarmingly loud.

## 3. Operationalise the RLA service behind a store and authorization — IMPLEMENTED


> **Implementation:** `governance-svc/crates/rla/src/store.rs` (Pg/Mem store enum) + rewritten handlers driving create→sample→interpret→report→finalise with bearer checks, duplicate-interpretation 409s, and finalise gated on `engine::audit_complete`; lifecycle smoke tests included.

**Threat:** audit theatre — a risk-limiting audit that exists as math but cannot be run cannot constrain a fraudster.

**Current gap:** the BRAVO engine is complete (`services/governance-svc/crates/rla/src/engine.rs`) but the HTTP handlers return static JSON with no persistence or authz (`rla/src/handlers.rs`).

**Existing hooks:** migrations for audit tables already exist (`services/governance-svc/migrations/V1..V4*.sql`); models cover runs, samples, entropy sources, manifests, witnesses, reports (`rla/src/models.rs`); the parallel-manual-count feed can push divergence inputs (`crates/parallel-manual-count/src/rla_feed.rs::submit_to_rla` posts to `/v1/audit-inputs`).

**Implementation path:** back handlers with a sqlx store mirroring the ecc-api `Store` enum pattern (Pg/Mem, `services/ecc-svc/crates/ecc-api/src/store.rs`); persist sample draws and audited interpretations; require SpiceDB `audit.run` permissions on every mutating route; finalise only when `engine::audit_complete` is true, emitting the signed `AuditReport`.

## 4. Verified public randomness beacon for audit seeds — IMPLEMENTED


> **Implementation:** `py/shangrla/src/shangrla/beacon.py` — `BeaconClient.fetch_pulse`, fail-closed RSA-SHA512 `verify_signature` (injectable key), `combine_seeds` over hash-committed party seeds (`commit_seed`/`verify_commitment`); mock-transport tests.

**Threat:** cherry-picked audit samples — whoever picks the seed can avoid auditing suspicious stations.

**Current gap:** the `EntropySource` model exists (beacon URI, pulse index, output value: `governance-svc/crates/rla/src/models.rs`) but nothing fetches or verifies a beacon anywhere in the repo; `SamplePlan.generate` takes any seed (`crates/parallel-manual-count/src/plan.rs:24`).

**Existing hooks:** both RLA engines already accept arbitrary seed strings and are deterministic given them (Rust `engine.rs draw_sample_positions`; Python `shangrla/engine.py::draw_sample`).

**Implementation path:** implement a beacon client (NIST Beacon v2 or drand League of Entropy) that verifies the beacon's own signature before accepting a pulse; mix three sources — beacon pulse + a seed contributed by each contesting party at close of poll (hash-committed before polls close, revealed after) — into the final seed; persist the full provenance chain in `EntropySource` and publish it alongside the plan so anyone can re-derive the draw.

## 5. Enforce custody-transition officer signatures — IMPLEMENTED


> **Implementation:** `inventory-svc/crates/rfid-cover-custody/src/officer_sig.rs` + enforcement in `repository::insert_transition`: high-risk transitions require an Ed25519 officer signature over `this_hash` from `EEL_CUSTODY_OFFICER_KEYS`; missing/unregistered/invalid all hard-fail.

**Threat:** forged custody records — a tampered ballot box shown as continuously sealed.

**Current gap:** `CoverStateTransition.officer_signature` is persisted but never written-with-proof nor verified (`services/inventory-svc/crates/rfid-cover-custody/src/repository.rs:142-154` binds it blind; no verify call exists outside seal-binding tests).

**Existing hooks:** the chain already hashes what must be signed — `this_hash = BLAKE3(prev ‖ canonical_json)` (`crates/rfid-cover-custody/src/hash_chain.rs:14`); Django replays the identical chain (`inventory_admin/apps/rfid_covers/views.py::_verify_chain`); hsm-signer provides signing infrastructure and role keys.

**Implementation path:** make `officer_signature = HSM_sign(officer_key, this_hash)` mandatory for the high-risk transitions `STATION_CLOSED → SEALED_RETURN → TALLY_ARRIVED → COUNT_COMPLETE`; verify at insert time in `repository.rs` and again in the Django chain view; a missing/invalid signature on those states quarantines the tag automatically (`QUARANTINE` state already exists in the FSM, `services/inventory-svc/migrations/V051__rfid_ballot_covers.sql:72-77`).

## 6. Turnout-plausibility and impossible-velocity analytics — IMPLEMENTED

> **Implementation:** `services/bvi-svc/crates/bvi-anomaly/` — `turnover_over_registered`, `velocity_impossible` (per-device travel-time feasibility), `burst_zscore` detectors plus `digit_distribution_chi_square`; exposed statelessly on bvi-api `POST /api/v1/bvi/anomalies/evaluate` (`routes/anomalies.rs`, bearer-gated). 23 unit tests.

**Threat:** ballot stuffing through kits — inflated identifications, or one kit "servicing" multiple stations.

**Current gap:** no plausibility checks exist on `identification_events` or `turnout_rollups`; alerts in ECC are fed only by upstream services.

**Existing hooks:** rich raw material is already captured — station-coded identification events with timestamps and device serials (`services/bvi-svc/migrations/V3__identification_events.sql`), station-level rollups with registered-voter denominators (`bvi-turnout/src/lib.rs`), and an alert pipeline with SSE push to operators (`services/ecc-svc/crates/ecc-api/src/sse.rs`, `ecc_live` alerts LiveView).

**Implementation path:** three detectors as a scheduled job writing into the existing ECC alert tables: (a) station turnout > 100% of registered; (b) velocity impossibility — same device serial identifying voters at two stations closer together than travel time allows; (c) burst anomaly — identification rate z-score vs fleet baseline. All three need only SQL over existing tables plus the alert writer.

## 7. Dual-control declaration signing (M-of-N quorum) — IMPLEMENTED


> **Implementation:** `rts-svc/crates/declaration/src/quorum.rs` — `DeclarationGate` M-of-N approvals with duplicate rejection; `SignedDeclaration::verify` gives portal/observers independent quorum checking; tampering outcome bytes breaks every signature.

**Threat:** unilateral premature or fabricated national declaration.

**Current gap:** `KeyRole::Declaration` ("signs M-of-N election result declarations", `packages/eel-crypto-rs/src/lib.rs:61`) and the full M-of-N custodian ceremony machinery exist (`hsm-signer/src/ceremony.rs` — open/approve/assert_approved with quorum enforcement), but no result-publication path invokes either.

**Existing hooks:** the declaration arithmetic is deterministic and property-tested (`services/rts-svc/crates/declaration/src/rules.rs::evaluate`); the 34C Merkle root and anchoring path exist (`rts-tally/src/{merkle,tlog_client}.rs`).

**Implementation path:** gate publication of the final declaration on `CeremonyStore::assert_approved` for a `KeyRole::Declaration` ceremony with quorum ≥2; sign `canonical(34C_root, declared_outcome)` inside the HSM; attach the signature to the anchored envelope so the portal (`packages/eel-web-astro`, verifier-rs) refuses to display a declaration lacking a quorum-valid signature.

## 8. Pin OIDC audience validation per service — IMPLEMENTED


> **Implementation:** `packages/eel-auth-rs/src/jwks.rs` — `JwksVerifier::with_audience(...)` enables true audience validation (`aud` string-or-array plus Keycloak `azp` fallback); services opt in via config so migration is per-service.

**Threat:** cross-service replay of operator credentials — a token minted for a low-risk console presented to a result-affecting admin API.

**Current gap:** `JwksVerifier` sets `validate_aud = false` (`packages/eel-auth-rs/src/jwks.rs:105`), so every JWT from the realm is accepted by every service regardless of intended audience.

**Existing hooks:** all services already construct the verifier centrally; the contract layer carries tier metadata (`x-eel-tier` in `packages/openapi-contracts/*/openapi.yaml`) that maps cleanly to expected audiences.

**Implementation path:** add `expected_aud: Option<String>` to the verifier config; set it per service from env (e.g. `EEL_OIDC_AUDIENCE=eel-ecc-api`); reject mismatches. One small crate change plus per-service config; closes credential-replay across all eighteen services at once.

## 9. Voter-facing station-result receipts (cast-and-verify for results) — IMPLEMENTED

> **Implementation:** tlog-api `GET /v1/tlog/receipts/{code}` (`rest/proofs.rs::resolve_receipt`) resolves a receipt code to a unique leaf and returns the inclusion proof together with the latest STH in one round trip; capture-side `rts-tally/src/tlog_client.rs::receipt_code` derives the printed code from the anchored payload hash; citizen page at `packages/eel-web-astro/src/pages/receipt.astro` shows MATCH/NO-MATCH client-side.

**Threat:** substitution between the form physically posted at a polling station and the form actually tallied — the classic mid-transit swap.

**Current gap:** the portal exposes log proofs generically, but nothing binds a station's posted paper to its logged leaf in a form a citizen can check.

**Existing hooks:** every published form is already hashed and anchored (`rts-tally` merkle + tlog pipeline); inclusion proofs are public over REST (`transparency-svc crates/tlog-api/src/rest/proofs.rs::get_inclusion_proof`); the Astro public site exists (`packages/eel-web-astro`).

**Implementation path:** print a short verification code (first 8 hex of the form's canonical hash) on each posted Form 34A; the citizen enters station code + code on the portal, which looks up the leaf by recomputed hash, fetches the inclusion proof, verifies it against the latest signed STH in-browser (verifier-rs WASM), and shows match/mismatch. Mismatches surface instantly as ECC alerts — turning every voter and agent into a substitution detector.

## 10. Automated witness gossip and split-view alarms — IMPLEMENTED


> **Implementation:** `tlog-witness/src/handler.rs` feeds every cosigned head through the detector and serves `GET .../head` + `GET .../split-views`; `tlog-witness/src/gossip.rs` polls `WITNESS_PEERS` heads on `WITNESS_GOSSIP_SECS` through the same detector.

**Threat:** equivocation — showing different trees to different observers goes unnoticed because nobody compares notes mechanically.

**Current gap:** the `SplitViewDetector` library works (`services/transparency-svc/crates/tlog-witness/src/split_view.rs:20`) but is wired into no binary; witnesses cosign independently without sharing observations; `tlog-admin` crosscheck is manual.

**Existing hooks:** witness HTTP endpoint for cosignature submission (`tlog-witness/src/handler.rs:32 post_cosign`); cosignature registry with UNIQUE constraints (`migrations/V1__initial_schema.sql:61-70`); ECC alert dispatch path (as in #6).

**Implementation path:** run `SplitViewDetector` inside `post_cosign` so every observed head is compared against all prior heads at the same tree size; add a periodic tlog-witness job that pulls peers' latest STHs and feeds them to the same detector; on detection, write a `split_view_detected` event (topic already declared in `deploy/service.yaml`) consumed by ECC with page-the-IAB severity. Optionally anchor the latest STH daily to an external timestamping service (the RFC 3161 TSA client already ships in `eel-crypto-rs` behind the `tsa` feature).

---

## Honourable mentions — ALL IMPLEMENTED

- ABIS SPIFFE middleware now validates real identities: parses Envoy-propagated XFCC / `X-Client-Spiffe-Id` SPIFFE URIs, enforces the `eel.cluster` trust domain and optional namespace allowlist (`EEL_ABIS_ALLOWED_NAMESPACES`) when `EEL_ABIS_SPIRE_REQUIRED=true` (`abis-api/src/middleware.rs`).
- Digit-distribution statistics ship as `digit_distribution_chi_square` (Wilson–Hilferty p-value) in `bvi-anomaly`, exposed alongside the other detectors on `POST /api/v1/bvi/anomalies/evaluate`.
- WORM retention: `transparency-svc/migrations/V3__worm_retention.sql` installs `BEFORE UPDATE OR DELETE` guards on `leaves`, `sth`, `witness_cosigs`, and `tree_tiles`; any tamper attempt is a hard SQL error.

## Deployment prerequisites (residual, non-code)

The code now enforces these controls; production trust additionally requires operational steps:
provision key registries (`EEL_AGENT_KEYS`, `EEL_CUSTODY_OFFICER_KEYS`, declaration gate spec)
at deployment; configure `WITNESS_PEERS` for gossip; run transparency migrations V2/V3;
point RLA/idempotency services at live databases so Pg stores activate; and exercise the
network-HSM path for STH/declaration keys per [anti-rigging.md](anti-rigging.md).
