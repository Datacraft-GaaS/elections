# Anti-Rigging Opportunities — Wave 2

Third volume in the integrity series: [anti-rigging.md](anti-rigging.md) (baseline capabilities), [anti-rigging-opportunities.md](anti-rigging-opportunities.md) (wave 1: ten controls + honourable mentions — all implemented), and this volume (wave 2).

**STATUS: ALL TEN OPPORTUNITIES AND ALL FOUR HONOURABLE MENTIONS ARE NOW IMPLEMENTED**, designed for **low/intermittent-connectivity operation** throughout (see "Offline-first operation model" at the end). Implementation citations follow each entry; the original gap analysis is retained for context.

---

## N1. Commit–reveal dual-channel Form 34A submission — IMPLEMENTED


> **Implementation:** `rts-ro-api/src/commit_reveal.rs` — `POST /api/v1/rts/forms/commit` (officer signature verified against `EEL_RO_OFFICER_KEYS`; idempotent; accepts SMS-relayed commitments late/out-of-order), `POST .../arrive` → matched / uncommitted / quarantined (409 carrying both hashes), `GET .../commitments` for offline reconciliation merges.

- **Threat:** in-transit alteration — the form transmitted by the station is replaced or edited between capture and tally entry.
- **Mechanism:** at close of poll the station emits an immediate, tiny **commitment** over the form (SHA-256 of canonical bytes, signed by the presiding officer) over a *different* transport than the full upload — e.g. SMS/USSD summary path via field-mobile-svc. The later full upload must hash to the committed value; mismatch quarantines the form and pages ECC.
- **Existing hooks:** the receipt-code derivation already computes the canonical form hash at capture (`services/rts-svc/crates/rts-tally/src/tlog_client.rs::receipt_code`); the USSD/SMS store-and-forward path exists in `services/field-mobile-svc` (wa-bot/ussd flows); s44a-style officer signing primitives are established platform-wide.
- **Build path:** add `POST /api/v1/rts/forms/commit` accepting `{form_hash, officer_signature}` (idempotent per station); on full-form arrival compare against any prior commitment; emit `form_committed.v1` so verifiers can see both values in the log.

## N2. Register-extract bundle anchoring with kit-side binding

- **Threat:** substitution of the voter register extract loaded onto a kit — swap the roll, control who can be identified.
- **Mechanism:** every register bundle distributed to kits gets a Merkle root anchored in the transparency log; kits verify bundle signature **and** the log anchor before opening the poll; the anchor id rides along on every identification event for cross-checking.
- **Existing hooks:** bvi-api already distributes config/bundle data to devices (`services/bvi-svc/crates/bvi-api/src/routes/config.rs`, kit_configs migrations); kit-side verification precedent exists in rts-kit ("signs bundles ... fails closed without anchors", per service status notes); Merkle + signing primitives in `packages/eel-crypto-rs`; transparency anchoring pipeline proven by rts-tally's tlog_client.
- **Build path:** add `bundle_root` column + anchoring to the bundle distribution flow; extend device config response with `{bundle_id, merkle_root, log_anchor}`; kit refuses poll-open without a verifying anchor; identification events carry `bundle_anchor` for post-hoc consistency sweeps.

## N3. Canary voter records (registry honeypots) — IMPLEMENTED


> **Implementation:** `bvi-api/src/routes/canaries.rs` (`EEL_VOTER_CANARIES`); hook in `voters.rs` fires on found AND not-found paths with byte-identical citizen responses and sha256-redacted logging; hits queue in a 1000-entry ring drained via bearer-gated `GET /api/v1/bvi/anomalies/canary-hits` (queue tolerates disconnected monitoring).

- **Threat:** bulk probing/exfiltration of the voter register through citizen-facing lookups (enumeration at scale, targeted intimidation).
- **Mechanism:** seed synthetic "canary" voters into the lookup path whose identities should *never* be queried legitimately (they belong to no real polling division). Any canary hit is proof of systematic enumeration — alarm with source context. Timing defence already equalises responses; canaries make bulk scraping *evident*, not just hard.
- **Existing hooks:** the lookup path is centralised and audit-logged with hashed actor/IP context (`services/bvi-svc/crates/bvi-api/src/routes/voters.rs`, `store.rs::voter_lookup_audit`); the anomaly pipeline (`bvi-anomaly`) provides the alert sink; EC-SEC-10 timing defence ensures canaries don't change observable behaviour.
- **Build path:** canary table + flag in the store; check in `voter_station` after resolution; route hits into ECC Critical alerts with the audit-row correlation id. Keep canary set secret-split (OpenBao) so even operators can't prune them silently.

## N4. Hash-chained configuration-action ledger for the election window — IMPLEMENTED


> **Implementation:** new shared crate `packages/eel-audit-chain-rs` — `AuditChain` BLAKE3 chain, JSONL `FileStore` (append-first durable write; verify-during-offline reload), axum `record_middleware` recording actor (SPIFFE header)/operation/query-hash on mutating routes without ever failing requests on storage errors.

- **Threat:** insider tampering with *configuration* rather than data — match thresholds, revocation lists, freeze flags changed mid-election without trace.
- **Mechanism:** generalise the custody BLAKE3 chain (`rfid-cover-custody/src/hash_chain.rs`) into an append-only, hash-chained **admin action ledger**: every mutating admin/config call appends `{actor_svid, operation, params_hash, prev_hash}`; the chain head is periodically anchored like other evidence. Any out-of-band DB edit breaks replay, exactly as for custody transitions.
- **Existing hooks:** chain implementation to copy is tested and reusable (`hash_chain.rs::compute_this_hash/verify_chain`); SPIFFE actor identity is available once ABIS-style middleware is enabled (`abis-api/src/middleware.rs::SpiffeIdentity::parse`); anchoring path identical to rts-tally's outbox client.
- **Build path:** shared crate `eel-audit-chain` (or extend eel-outbox) + axum middleware writing entries on mutating routes of governance/support-admin/ecc APIs; nightly `verify_chain` sweep job.

## N5. Recording-continuity certification for polling-station video — IMPLEMENTED


> **Implementation:** `rts-ro-api/src/recording_continuity.rs::certify_session` — continuity over on-device anchor SEQUENCE numbers so batch/out-of-order uploads certify identically to in-order streams; gaps listed by seq; unclosed flagged. Endpoint `POST /api/v1/rts/ro/session-recording/certify`.

- **Threat:** gaps in station session recordings precisely where irregularities occur — recordings exist but prove nothing if gappable at will.
- **Mechanism:** session recording anchors every 60 s (`rts-ro-api session_recording` → transparency log); certification requires **continuity**: consecutive anchor hashes chaining across the whole open→close window with no gap exceeding one interval. A certified-recording flag becomes an input to result acceptance; gaps escalate instead of silently degrading.
- **Existing hooks:** start/anchor/stop endpoints exist (`services/rts-svc/crates/rts-ro-api/src/handlers.rs:185-194 session_recording`, hashes forwarded to transparency-svc); quarantine semantics precedent in the custody FSM.
- **Build path:** continuity checker over anchored hashes per station/session (pure function + tests, mirroring `officer_sig.rs` structure); emit `recording_certified.v1 | recording_gap.v1`; surface certification state next to each 34A in the RO console.

## N6. Threshold-encrypted tallies until close — IMPLEMENTED


> **Implementation:** new crate `rts-svc/crates/tally-threshold` — Shamir k-of-n over BLS12-381 Fr (`split_key`/`combine_shares`) plus keyed-BLAKE3 seal/open with MAC tag (`seal_payload`/`open_payload`). Encryption entirely at capture; decryption is an M-of-N ceremony needing no network until shares are collected. Doc-comment flags keystream as dev-grade (swap for AEAD behind the same API for production).

- **Threat:** real-time trend leakage enabling *targeted* fraud after early results are visible to insiders (the leak itself is the vulnerability).
- **Mechanism:** 34A payloads are encrypted at capture under the cycle threshold public key; decryption requires M-of-N commissioner shares at count time. Trend dashboards operate on counts-without-content until decryption, removing the incentive structure for early-result manipulation.
- **Existing hooks:** the FHE crate already models encrypt→aggregate→threshold-decrypt→prove (`services/rts-svc/crates/fhe-tally/{encrypt_leaf,aggregate,decrypt}.rs`); M-of-N ceremony semantics implemented in `hsm-signer/src/ceremony.rs`; KeyRole::CycleKek exists in `packages/eel-crypto-rs/src/lib.rs`.
- **Build path:** wrap leaf payloads under CycleKek-derived key shares at sequencer ingest; gate decryption behind `CeremonyStore::assert_approved`; keep the existing Groth16 proof path intact over plaintext-at-count.

## N7. Daily signed integrity statement appended to the log — IMPLEMENTED


> **Implementation:** `tlog-admin integrity-statement --db-url --signing-key-hex …` — builds `{tree_size, root_hash, witness_cosignatures, open_ingest_failures, generated_at}` from local DB state, signs canonical bytes, exits non-zero when open failures exist. Monitors alert on statement *age*, not liveness.

- **Threat:** *absence* attacks — a compromised component quietly stops publishing STHs/witness cosignatures/anomaly reports; nothing notices a missing thing unless someone is looking.
- **Mechanism:** a scheduled job composes a daily **integrity statement** (latest tree size/root, witness cosignature counts, open ingest failures = required 0, anomaly counts, coverage stats from N8), signs it with the `eel-sth` or declaration role, and appends it as a meta-domain leaf. Downstream monitors alert on *statement absence or regression*, converting silence into signal.
- **Existing hooks:** meta-events domain already routes (`tlog-ingest/src/topic_router.rs`); signing backends ready (`tlog-sequencer/src/signer.rs`); witness counts queryable from `witness_cosigs`; failure counts from `tlog-ingest/failures.rs::list_open`.
- **Build path:** cron-style task in tlog-admin or a small daemon; statement schema in `packages/event-schemas`; portal displays "last integrity statement" age prominently.

## N8. Publication coverage reconciliation certificate — IMPLEMENTED


> **Implementation:** `tlog-admin coverage --expected F --present F --signing-key …` — pure `coverage_diff` → sorted missing/unexpected, Ed25519-signed certificate, non-zero exit while anything is missing. Present-lists accumulate from whatever has arrived; certificates regenerate as connectivity returns.

- **Threat:** forms never submitted for logging — the dead-letter control catches failed appends, but a station whose results simply never enter the pipeline is invisible to it.
- **Mechanism:** reconcile the authoritative list of expected forms (stations × form types from the register/config side) against leaves actually present in the log; publish a signed daily **coverage certificate** (expected, present, missing-by-station). Missing rows block certification of downstream tallies.
- **Existing hooks:** station registry lives in ecc-api/bvi (station codes, `polling_stations` migration V13 in bvi-svc); leaf lookups trivially available in tlog-api; certificate signing reuses P3/P7 machinery.
- **Build path:** reconciliation query + pure differ function (testable against fixtures), exposed via `tlog-admin coverage --election …`; certificate event onto `eel.meta-events`.

## N9. Risk-based adaptive audit expansion (automated continuation) — IMPLEMENTED


> **Implementation:** `governance-svc/crates/rla/src/engine.rs` — `draw_sample_block` (deterministic continuation of the beacon-seeded stream: block k reproducible offline by anyone holding seed+manifest) and `required_additional_samples`; endpoint `GET /rla/audits/{run_id}/next-block` returns the pre-drawn block with rationale (current T, threshold, required winner-ballots).

- **Threat:** audits stopped too early — BRAVO continues until T crosses 1/α, but human-operated sampling invites "good enough" stopping on marginal assertions.
- **Mechanism:** after each interpretation batch, the RLA service automatically computes the additional sample size required per assertion (standard BRAVO/Gamma continuation arithmetic from current T and remaining manifest) and pre-draws the next deterministic block from the beacon-seeded stream; finalisation remains impossible until `audit_complete` regardless.
- **Existing hooks:** everything needed exists in `governance-svc/crates/rla/src/engine.rs` (`compute_t_statistic`, `draw_sample_positions` counter-mode stream supports drawing block k+1 deterministically) and the store lifecycle already locks premature finalisation.
- **Build path:** `next_sample_block(run)` engine function + endpoint returning the pre-drawn block; smoke test asserting monotone T under clean ballots and that blocks are reproducible.

## N10. Signed verifier distribution with in-log build attestation — IMPLEMENTED


> **Implementation:** `tlog-admin attest-artifact --artifact-sha256 --rekor-uuid --signer-name --signing-key-hex` (+ `attest-verify`) issuing Ed25519 attestations over canonical field bytes; receipt page embeds its bundle digest at build time and surfaces it beside results (`packages/eel-web-astro/src/pages/receipt.astro`) — comparison is local hence offline-safe.

- **Threat:** supply-chain attack on the *verification tooling itself* — citizens/parties verify proofs with a binary that lies.
- **Mechanism:** sign `verifier-rs` release artifacts (native + WASM used by the receipt page) with Sigstore/Rekor; record artifact digests and Rekor UUIDs in a transparency-log meta leaf; the receipt page pins its own WASM digest and checks it against the in-log attestation on load. A verifier that cannot prove its own integrity is refused by the page.
- **Existing hooks:** `packages/verifier-rs` already ships `wasm.rs` for browser use; `sigstore = { version = "0.10", features = ["bundle", "rekor"] }` is a workspace dependency (`Cargo.toml:94`); reproducible-build pipeline exists (`reproducibility/make-reproduce/scripts/reproduce.sh`, `multi-arch-verify.sh`).
- **Build path:** release workflow step signing artifacts; attestation envelope via the standard outbox→meta-events anchoring; `receipt.astro` fetches attestation leaf by digest prefix (reuses the wave-1 receipt resolution endpoint pattern).

---

## Honourable mentions (wave 2) — ALL IMPLEMENTED

- **Dual-control triage:** Django `_resolve_alert` enforces two distinct officers (`rfid_covers/dual_control.py::validate_dual_approval`; self-approval rejected; countersigned note persisted; 422 otherwise), and the ECC LiveView resolve flow requires a second-officer id before calling inventory-svc (`cover_tamper_live.ex`). Officers are co-located, so approval is captured offline and recorded with both ids. Pure-logic unit tests included.
- **Time-bound intake guard:** `bvi-api/src/routes/intake_guard.rs` — `EEL_POLL_WINDOW` (RFC3339,RFC3339) + skew tolerance (`EEL_INTAKE_SKEW_SECS`, default 120): Open proceeds unchanged; within-skew drift proceeds flagged (`x-eel-intake-review: drift=<secs>` header + warn); outside rejects with 403. Unset env preserves prior behaviour exactly. Geofence remains a detection concern served by the velocity checks in bvi-anomaly.
- **Fault-injection policy tests:** ingest offset decisions extracted to a pure `offset_policy` fn with DB-free tests asserting Advance ONLY on success across rate-limit / partition / dead-database injections (dedup path documented as still advancing); witness gossip parser hardened via `parse_peer_head` with malformed-shape tests (string/negative/float/null heads skipped without panic).
- **Cross-ledger reconciliation:** `bvi-anomaly::cross_ledger_delta` compares bvi-vs-rollup and abis-vs-rollup within tolerance, returning **Stale** (never a false alarm) when any ledger exceeds `max_staleness_secs` — explicitly designed for delayed sync; doc-comment explains why abis-vs-bvi is never compared directly.

## Offline-first (low-connectivity) operation model

Every wave-2 control was built to one rule: **decisions are computed from local state; networks only move data opportunistically.**

| Pattern | Applied in |
|---|---|
| Deterministic seeds/streams — blocks re-derivable offline by any party | N9 (`draw_sample_block`), beacon mixing, PMC plan |
| Capture-time crypto; ceremony later | N6 (encrypt at capture, M-of-N decrypt at count) |
| Append-first local durability; verify by local replay | N4 (JSONL FileStore verify-on-load), P2 dead-letter |
| Sequence-number semantics instead of arrival time | N5 (anchor seq continuity), N1 (late/out-of-order/duplicate commits idempotent) |
| Queued alarms drained later; staleness reported instead of false alarms | N3 canary ring + drain endpoint, HM4 `Stale` verdict, HM2 review-bucket drift header |
| Cached registries served from process memory after startup | N2 anchor views (`from_cache: true`), officer/agent key registries |
| Absence-tolerant monitoring (age thresholds, regenerable certificates) | N7 statement age alerts, N8 certificates |
| Embedded digests — client self-check needs no network | N10 receipt-page bundle digest |

Operational corollary: field devices need connectivity only for poll-open provisioning (register anchors, keys, windows) and opportunistic flushes of commitments/anchors/hit-queues; every enforcement decision is computed locally.

## Suggested priority

1. **N7 + N8** (integrity statement + coverage certificate) — cheap, closes every "silence" blind spot opened by the other controls.
2. **N4** (admin action ledger) — insiders are the hardest adversary; the chain primitive is already written.
3. **N5** (recording continuity) and **N1** (commit-reveal) — close the physical-to-digital trust boundary.
4. **N2** (register anchoring), **N3** (canaries), **N10** (signed verifiers).
5. **N6** (threshold encryption) and **N9** (adaptive audit expansion) — largest builds, strongest long-term guarantees.
