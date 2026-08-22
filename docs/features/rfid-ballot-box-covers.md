# RFID ballot box covers (RFP requirement)

Integration of tamper-evident RFID tagging on ballot box covers into the eel platform, per RFP IEBC/OIT/01/2026-2027 physical asset requirements. Owned by inventory-svc; cross-references M20 (inventory-management), M39 (ballot-printing-chain), M65 (diaspora-ballot-physical-logistics), and HARD-16 (physical seal binding).

## Contents

1. What CFR ballot box covers do (vs box body)
2. Physical layer specification
3. Data plane
4. Rust crate + reader-driver architecture
5. State machine (cover lifecycle)
6. Reader hardware types + integration
7. Human surfaces
8. Integrity + fraud posture
9. Court + audit posture
10. Honest caveats + external prerequisites

---

## 1. What cover tags do (vs box body tags)

The RFP requires RFID **on the cover specifically** so a cover-swap attack is detectable, in addition to the box body tag already covered by M39. The cover tag is bound to the box body at manufacture; any mismatch at read time raises a `cover_tamper_alert`. The tag antenna is cover-lift-detecting: any lift outside an allowed transition breaks or inverts the antenna response permanently.

## 2. Physical layer specification

| Attribute | Value | Reason |
|---|---|---|
| Air-interface | EPC Gen2v2 / ISO/IEC 18000-63 UHF (860 to 960 MHz Kenya-allocated) | Long read range (5 to 10 m through cardboard), battery-free, cheap, wide reader ecosystem |
| Chip family | NXP UCODE 9 or Impinj Monza R6-P | Anti-clone Gen2v2 crypto suite, user memory for state, kill-command for post-archive |
| Tag form factor | Tamper-evident cover-lift adhesive tag with printed loop antenna on flexible substrate; antenna breaks on cover lift | Cover-lift permanently changes tag response; detectable at next read |
| Serialisation | 96-bit EPC bound at manufacture to the box body serial; both must match at close | Prevents cover-swap |
| Cryptographic identity | HMAC-SHA256 tag authentication using per-tag secret from HSM at issuance (Gen2v2 authenticate command) | Prevents tag-cloning; reader challenges tag with nonce before accepting the read as authentic |

Optional BLE beacon backup on the cover for active tracking during transit (`bluetooth_id` field on CustodyEvent).

## 3. Data plane (migration `V051__rfid_ballot_covers.sql`)

Four new tables in inventory-svc:

- `eel_rfid_cover_tags` — one row per manufactured cover tag; bound to a box at pairing time.
- `eel_rfid_cover_reads` — every read at a reader gate, kit, or handheld; carries HMAC challenge + response + reader signature + geo.
- `eel_cover_state_transitions` — append-only lifecycle FSM transitions with per-tag `prev_hash` chain.
- `eel_cover_tamper_alerts` — any lift outside an allowed transition + severity + ECC dispatch state.

Append-only triggers on `eel_cover_state_transitions` and `eel_cover_tamper_alerts` block UPDATE + DELETE (mirroring the reviewer-actions pattern from V013).

## 4. Rust crate + reader driver

`services/inventory-svc/crates/rfid-cover-custody` — new crate:

- `src/tag.rs` — CoverTag domain + issuance + pair-to-box.
- `src/reader.rs` — Reader trait with concrete adapters (Impinj Speedway, Zebra FX9600, CipherLab RS36) + HMAC challenge/response.
- `src/read_ingest.rs` — read event ingest, HMAC verify, tamper-flag, custody-event emission.
- `src/state_machine.rs` — MANUFACTURED to STORAGE_ARCHIVE FSM with allowed transitions per reader_type + officer role.
- `src/tamper_alert.rs` — cover-lift detection, ECC dispatch, tlog anchor.
- `src/api.rs` — REST + gRPC surface: POST /v1/rfid/tags (issuance), POST /v1/rfid/tags/{id}/pair, POST /v1/rfid/reads, POST /v1/rfid/tags/{id}/transition, GET /v1/rfid/tags/{id}/history, GET /v1/rfid/tamper-alerts.

`services/inventory-svc/py/rfid-reader-driver` — Python worker exposing a stable HTTP+MQTT interface over open-source libraries (sllurp LLRP for Impinj + Zebra; bleak BLE for CipherLab RS36). No commercial SDKs required. Adapter interface + per-vendor concrete adapter + hash-based synthetic adapter for CI + tests. See ADR-0025.

`integrations/rfid-vendors/{impinj, zebra, cipherlab}/` — 3 vendor adapter directories each with adapter + double + Pact contract-tests + README.

## 5. State machine

`MANUFACTURED -> PRINTER -> WAREHOUSE -> DISPATCHED -> IN_TRANSIT -> STATION_RECEIVED -> STATION_OPENED -> VOTING_ACTIVE -> STATION_CLOSED -> SEALED_RETURN -> TALLY_ARRIVED -> COUNT_COMPLETE -> STORAGE_ARCHIVE -> DESTROYED`

Plus a QUARANTINE absorbing state reachable from any transition on tamper.

Per-transition gate:

| From | To | Required reader | Additional gates |
|---|---|---|---|
| MANUFACTURED | PRINTER | warehouse_gate at printer facility | pairing to box body recorded |
| PRINTER | WAREHOUSE | warehouse_gate at IEBC warehouse | HARD-16 seal photo bound |
| WAREHOUSE | DISPATCHED | warehouse_gate egress + officer four-eyes | dispatch manifest signed |
| DISPATCHED | IN_TRANSIT | transit_checkpoint | window: dispatch_ts + 48 h |
| IN_TRANSIT | STATION_RECEIVED | station_arrival at expected geo (M18 Geofence.Check) + within polling-day window | RO signature |
| STATION_RECEIVED | STATION_OPENED | kit_handheld + dual-officer signature + s.14A timing gate | opening seal photo bound |
| STATION_OPENED | VOTING_ACTIVE | kit_handheld after clerk EOD-opening event | none |
| VOTING_ACTIVE | STATION_CLOSED | kit_handheld + dual-officer signature | closing seal photo bound |
| STATION_CLOSED | SEALED_RETURN | kit_handheld at station egress | closing manifest signed |
| SEALED_RETURN | TALLY_ARRIVED | transit_checkpoint at tally centre | tally-centre officer signature |
| TALLY_ARRIVED | COUNT_COMPLETE | ro_desktop after 34C compile (M09) | RO session hash to tlog |
| COUNT_COMPLETE | STORAGE_ARCHIVE | archive_gate at archive facility | archive officer signature |
| any | QUARANTINE | any reader flagging tamper | ECC dispatch |
| STORAGE_ARCHIVE | DESTROYED | archive_gate after 15-year retention | Commissioner + HSM signature |

## 6. Reader hardware types + integration

- **Warehouse + archive gates** — Impinj Speedway R700 fixed reader; wired ethernet; SPIRE SVID via nodeagent.
- **Transit checkpoint gates** — same; ruggedised; solar + battery backup.
- **Kit-integrated handheld** — Impinj xArray-clip or Zebra RFD40 UHF sled paired with M09 RO Tauri desktop or the custody-mobile Flutter app via BLE / USB-OTG.
- **Handheld PDA** — dedicated Zebra MC3300xR or CipherLab RS36 for field staff.
- **Airline cargo + MFA customs** — integration via existing `integrations/airline-cargo` + `integrations/kra-eclearance` adapters (already wired).

Each reader boots via a signed firmware image, SPIRE-issues an SVID scoped to `spiffe://eel.local/reader/<serial>`, and can only submit reads to `POST /v1/rfid/reads` with its SVID header.

## 7. Human surfaces

- **`services/inventory-svc/django/inventory-admin`** — extend with RfidCoverTag + CoverRead + CoverTamperAlert ModelAdmin.
- **`services/inventory-svc/apps/custody-mobile`** — extend with an RFID scan screen (BLE-paired reader) + tamper-alert workflow.
- **`services/ecc-svc/ex/ecc_live`** — real-time "Cover tamper alerts" panel via Phoenix PubSub.
- **`services/portal-svc/astro/public`** — optional public custody-chain viewer for a specific ballot box (query by station + serial), rate-limited.

## 8. Integrity + fraud posture

- **Anti-clone** — Gen2v2 crypto-authenticated tag; HMAC-SHA256 challenge/response with per-tag secret in HSM. A cloned tag fails the challenge; the server logs `challenge_valid=false`.
- **Anti-lift** — antenna breaks on cover lift; tag fails to respond or responds with a distinctive tamper-indicator bit. Cover-lift outside an allowed transition raises a critical alert.
- **Anti-substitution** — EPC bound to box body at manufacture; both must scan at every transition; mismatch triggers a cover_tamper_alert.
- **Anti-replay** — reader challenges tag with a fresh 16-byte nonce per read; server rejects a read whose challenge appears twice for the same tag within 5 minutes.
- **Chain-of-custody** — per-tag `prev_hash` chain over `eel_cover_state_transitions`; silent rewrite breaks the chain.
- **HSM anchoring** — every state transition is HSM-signed by the responsible officer; every reader read is HSM-signed by the reader; both hash-chained.
- **Merkle anchoring** — the whole `eel_cover_state_transitions` table is periodically anchored to the P05 transparency log so any court can pull a per-box audit.
- **Cross-check with seal photo** — HARD-16 physical-seal-binding remains authoritative; RFID + seal photo + officer signature must all corroborate.

## 9. Court + audit posture

For SCoK petition evidence:

- **Chain-of-custody proof** — every ballot box has a full state-transition trail from manufacture to archive; each transition HSM-signed; each read HMAC-verified; the whole chain Merkle-anchored.
- **Tamper alert list** — every alert with detection time, geo, officer, resolution — court can query per box.
- **Independent verifiability** — the RFID chain is anchored to the transparency log; petitioner counsel can pull inclusion proofs via M40.
- **Comparison with paper** — RFID reads + seal photos + physical ballot count must all agree; any divergence is auditable and flagged.

## 10. Honest caveats + external prerequisites

- **Tag procurement** — 100k tags at ~15 to 20 KES each (~2M KES total) requires competitive tender; specification aligned with EPC Gen2v2 + tamper-lift adhesive form factor.
- **Reader procurement** — approx. 60 fixed gates (warehouses + tally centres + archive) + 250 handhelds (~30M KES capex). Vendor selection to be validated against 46,229-station scale.
- **Kenya UHF band clearance** — 865 to 868 MHz allocation by Communications Authority of Kenya; already open for EPC Gen2 per CAK 2022 band plan.
- **Reader firmware attestation** — SPIRE-issued SVID at boot requires signed firmware; vendor cooperation for a signed image needed for Impinj + Zebra + CipherLab.
- **Open RFID stack** — LLRP 1.1 (EPCglobal, royalty-free) for Zebra + Impinj fixed readers; BLE GATT + EPC Gen2 profile (open) for CipherLab RS36 handheld. No commercial SDK licences required. See ADR-0025.
- **Physical tag application** — cover tags applied at the ballot printer (M39) at time of cover manufacture; requires the printer MES to be updated to include the tag-application step + pairing to box body serial.
- **Airline + MFA cargo readers** — depends on partner cooperation (Kenya Airways cargo terminal + MFA customs terminal); interfaces already scaffolded in `integrations/airline-cargo` + `integrations/kra-eclearance`.

Related documents:
- `docs/features/anti-fraud-and-rigging-detection.md` — full 24-item hardening catalogue.
- `docs/features/SC-defensibility.md` — court-facing evidence surface.
- `services/inventory-svc/docs/CAPABILITIES.md` — M20 + M39 + M65 catalogue.
- `docs/runbooks/rfid-tamper-response.md` — ECC dispatch runbook.
- `docs/features/diagrams/rfid-ballot-box-covers.svg` — architecture diagram.

## 11. As-built status

All 12 planned deliverables are shipped across 3 commits.

### RFID-A — Rust crate + migration (commit `fd6e7f3`)

| Path | Purpose | Tests |
|---|---|---|
| `services/inventory-svc/migrations/V051__rfid_ballot_covers.sql` | 4 tables + append-only triggers on transitions + tamper_alerts + ADR-0016 data-class COMMENTs | schema |
| `services/inventory-svc/crates/rfid-cover-custody/src/domain.rs` | CoverState (15 variants) + ReaderType (8) + TamperState (5) + Severity (3) | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/tag.rs` | UUID7 + 96-bit SGTIN-96 EPC issuance + pair-to-box invariants | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/reader.rs` | Reader trait + HmacChallenger (Gen2v2 HMAC-SHA256) + ReaderRegistry SVID | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/state_machine.rs` | 13-transition table + QUARANTINE absorbing + four-eyes validator | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/hash_chain.rs` | BLAKE3 prev/this hash chain + verify_chain reconstruction | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/replay_defence.rs` | NonceStore trait + InMemoryNonceStore with 5-min window auto-prune | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/tamper_alert.rs` | classify_severity + build_alert + async ECC dispatch via reqwest | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/read_ingest.rs` | Pure pipeline: SVID check → replay defence → HMAC verify → CoverRead → alert | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/api.rs` | Axum 0.8 with 7 routes (issuance, pair, reads, transition, history, tamper-alerts triage) | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/events.rs` | 5 CloudEvent type constants + envelope builders | — |
| `services/inventory-svc/crates/rfid-cover-custody/src/metrics.rs` | 4 prometheus CounterVec (reads + tamper + transitions + hmac failures) | — |
| `services/inventory-svc/crates/rfid-cover-custody/tests/rfid_test.rs` | 47 tests | 47 |

Zero `unwrap()` in library src. Zero `iebc` identifiers. `cargo build --release` clean.

### RFID-B — Python reader driver + 3 vendor adapters (commit `3e8eed8`)

| Path | Purpose | Tests |
|---|---|---|
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/main.py` | FastAPI with 6 endpoints (register + read + batch-read + challenge + status + health) | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapter.py` | RfidReaderAdapter ABC (issue_challenge + submit_response) | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/synthetic.py` | Deterministic hash-based adapter with configurable tamper_prob (for tests + CI) | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/impinj_llrp.py` | Impinj Speedway R700 via LLRP (guarded import) | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/zebra_llrp.py` | Zebra FX9600 via LLRP 1.1 (sllurp, MIT); no commercial SDK | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/cipherlab_rs36.py` | CipherLab RS36 handheld via BLE GATT + SPP (bleak, MIT); no commercial SDK | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/adapters/llrp_common.py` | Shared LLRP-1.1 client (sllurp) with circuit breaker; used by Zebra + Impinj | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/hmac_gen2v2.py` | HMAC-SHA256 challenge/response per EPC Gen2v2 + 5-min nonce dedup via LRU OrderedDict | — |
| `services/inventory-svc/py/rfid-reader-driver/src/eel_rfid_reader/upstream.py` | Async httpx client forwarding to cover-custody REST with Idempotency-Key per UUID7 | — |
| `services/inventory-svc/py/rfid-reader-driver/tests/*` | 36 pytest (adapters + hmac + main + upstream); pyright clean | 36 |
| `integrations/rfid-vendors/impinj/{adapter,double,contract-tests,README}` | Rust adapter + tamper-injection double + Pact v2 + hardware notes (Speedway R700, 865-868 MHz, LLRP 1.1) | 5 |
| `integrations/rfid-vendors/zebra-llrp/{adapter,double,contract-tests,README}` | Rust adapter (EEL_ZEBRA_LLRP_ADAPTER_URL) + tamper-injection double + Pact v2 + hardware notes (FX9600, LLRP, sllurp) | 5 |
| `integrations/rfid-vendors/cipherlab-ble/{adapter,double,contract-tests,README}` | Rust adapter (EEL_CIPHERLAB_BLE_ADAPTER_URL) + tamper-injection double (battery_low knob) + Pact v2 + hardware notes (RS36 handheld, BLE, bleak) | 6 |

Each vendor crate is standalone (`[workspace]` opt-out) with path dep to `eel-resilience-rs` (CircuitBreaker 5 failures / 60 s window / 30 s cooldown).

### RFID-C — Human surfaces + diagram + runbook (commit `5f11d52`)

| Path | Purpose | Tests |
|---|---|---|
| `services/inventory-svc/django/inventory-admin/apps/rfid_covers/models.py` | 4 Django models mirroring V051 (managed=False) | — |
| `services/inventory-svc/django/inventory-admin/apps/rfid_covers/admin.py` | 4 ModelAdmin + ResolvedStatusFilter + mark_retired + bulk_resolve_with_note | — |
| `services/inventory-svc/django/inventory-admin/apps/rfid_covers/views.py` | HTMX views: box_chain (with prev_hash verification) + tamper_triage (inline resolve) + reader_health | — |
| `services/inventory-svc/django/inventory-admin/apps/rfid_covers/tests/*` | 23 pytest (admin + views + permission-aware queryset) | 23 |
| `services/ecc-svc/ex/ecc_live/lib/ecc_live_web/live/cover_tamper_live.ex` | LiveView on PubSub `eel:inventory:cover_tamper`; 15s auto-refresh; severity-triple-encoded badges (colour + shape + text) | — |
| `services/ecc-svc/ex/ecc_live/lib/ecc_live_web/live/cover_history_live.ex` | Per-tag state-transition timeline with in-memory chain verification | — |
| `services/ecc-svc/ex/ecc_live/test/ecc_live_web/live/{cover_tamper,cover_history}_test.exs` | 18 ExUnit tests | 18 |
| `services/inventory-svc/apps/custody-mobile/lib/src/services/rfid_reader_service.dart` | RfidReaderService trait + SyntheticReaderService + factory (Zebra + CipherLab + synthetic) | — |
| `services/inventory-svc/apps/custody-mobile/lib/src/screens/rfid_scan.dart` | BLE pairing + scan-result card + officer confirmation + state transition submit + offline queue + duress long-press | — |
| `services/inventory-svc/apps/custody-mobile/test/*` | 18 Flutter tests (widget + unit) | 18 |
| `docs/features/diagrams/rfid-ballot-box-covers.svg` | 1200x800 valid XML + colour-blind-safe + HSM lock icons at every transition + Merkle anchor + tamper → ECC path | — |
| `docs/runbooks/rfid-tamper-response.md` | 10 sections; ECC + Commissioner + Legal + ODPP escalation; Elections (Technology) Regulations 2017 Reg 17 citation | manual verify |

### Cross-language test totals

- **Rust**: 47 (cover-custody) + 15 (3 vendors) = 62 tests
- **Python**: 36
- **Django**: 23
- **Elixir**: 18
- **Flutter**: 18
- **Grand total: 157 tests across 6 languages**

### Deviations tracked

- `lot_id` FK to `eel_printing_lots` deferred: table not yet migrated (M39 in flight); UUID column stored without REFERENCES constraint with an inline COMMENT flagging the pending FK.
- `services/inventory-svc/Cargo.toml` not created: no nested workspace; root glob `services/*/crates/*` picks up the new crate automatically.
- Vendor adapter crates are standalone (`[workspace]` opt-out) to avoid touching root Cargo.toml.
- pyright mode is `standard` not `strict` to match existing codebase; TestClient's partial stubs from starlette trip strict mode.

### External prerequisites remaining

- Tag procurement (100 k tags × ~15-20 KES ≈ 2M KES) via competitive tender.
- Reader procurement (~60 fixed gates + 250 handhelds ≈ 30M KES capex).
- Kenya UHF band clearance (CAK 2022 band plan already allocates 865-868 MHz for EPC Gen2 — administrative, not blocking).
- Signed firmware images from Impinj + Zebra + CipherLab for SPIRE-attested reader boot.
- No commercial SDK licences required: LLRP (EPCglobal, royalty-free) for Zebra + Impinj; BLE GATT (open) for CipherLab. See ADR-0025.
- Printer MES update to include cover-tag application + pairing step in the M39 print run.
- Airline + MFA cargo terminal reader integration (existing adapters at `integrations/airline-cargo` + `integrations/kra-eclearance` are wired).
