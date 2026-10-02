# RTS Returning Officer (M09)

Module M09 (`rts-ro`) owned by `rts-svc`. Returning Officer desktop application (Tauri) for compiling Form 34B (constituency tally) and Form 34C (county tally), editing tallies with mandatory reason codes, handling dispute cases, requiring agent countersign, and hashing session recordings to the transparency log.

## Contents

1. What M09 does
2. Data plane
3. Ingest sources
4. Pipeline
5. API surface
6. Human surfaces
7. Integrity + anti-tamper
8. Kenya statute alignment
9. Honest caveats + external prerequisites
10. Test coverage + operational runbooks

---

## 1. What M09 does

M09 is the tool used by Returning Officers (RO) at constituency and county tallying centres. After receiving Form 34A transmissions from polling stations (M10), the RO uses the Tauri desktop app to review station tallies, compile the Form 34B, resolve discrepancies, accept or reject agent challenges, and sign the final declared result. The signed Form 34B is countersigned by accredited party agents before submission to M10.

M09 is distinct from M08 (kit-side transmission) and M10 (server-side event-sourced tally). M09 is the human decision-making surface that sits between them.

## 2. Data plane

Crates: `services/rts-svc/crates/rts-ro-api/`, `rts-domain/src/aggregates/form34b.rs`, `rts-domain/src/aggregates/form34c.rs`, `declaration/`.

| Aggregate | Purpose | Data class |
|---|---|---|
| **Form34B** | form_id, constituency_code, station_tallies[], ro_edit_log[], dispute_cases[], ro_sig, agent_sigs[], status (DRAFT/DECLARED/DISPUTED) | A |
| **Form34C** | form_id, county_code, constituency_totals[], county_ro_sig, status | A |
| **RoEditEntry** | edit_id, form_id, field, old_value, new_value, reason_code, ro_identity, ts_hlc | A |
| **DisputeCase** | case_id, form_id, raised_by (agent), reason, status, resolution_note, resolved_by, ts | B |
| **AgentCountersign** | sig_id, form_id, agent_identity, sig_bytes, ts_hlc | A |
| **SessionRecordingHash** | recording_id, sha256_hash, tlog_leaf_id, ts | C |

## 3. Ingest sources

| Source | Notes |
|---|---|
| M10 rts-backend | Form34A aggregates per station available via `GET /rts/v1/ro/stations` |
| Returning Officer (biometric) | Fingerprint login + per-edit biometric confirmation |
| Party agents (countersign) | Agent JWT + biometric; countersign required before Form 34B submission |
| Physical Form 34A scans | Displayed alongside transmitted tally for manual cross-check |

## 4. Pipeline

Crates: `services/rts-svc/crates/rts-ro-api/`, `declaration/src/`, `rts-projections/src/ro_view.rs`.

```
RO biometric login (fingerprint) -> Tauri desktop app
      -> load constituency station tallies from M10
             (rts-projections/src/ro_view.rs: ro_view materialised projection)
      -> display Form 34B compilation view:
             station-by-station tally with ICR confidence flags
             stations with missing / disputed transmissions highlighted
      -> RO edits (if needed):
             each edit: mandatory reason_code + RO biometric confirm
             RoEditEntry appended (immutable edit log)
      -> dispute case handling:
             agent raises dispute -> DisputeCase opened
             RO reviews -> resolves with reason note
             DisputeCase.status = RESOLVED / UPHELD
      -> RO signs Form 34B (Ed25519, RO identity + form hash + HLC)
      -> agent countersign round:
             each accredited agent biometric-auth + signs
             minimum agent countersign count configurable (default: 1 per party)
      -> session recording hash:
             Tauri captures tallying session recording
             SHA-256 of recording submitted to transparency log (P05)
             tlog_leaf_id stored in SessionRecordingHash
      -> submit Form 34B to M10:
             POST /rts/v1/ro/form34b
             M10 emits eel.rts.form34b_received.v1
      -> county RO compiles Form 34C from constituency Form 34Bs
             same edit + countersign + tlog pattern
```

## 5. API surface

Contract: `packages/openapi-contracts/rts/openapi.yaml` (shared with M08/M10).

| Method + path | Persona | Purpose |
|---|---|---|
| `GET /rts/v1/ro/stations` | RO (JWT + SpiceDB `RETURNING_OFFICER`) | Fetch constituency station tally summary |
| `GET /rts/v1/ro/form34a/{station_code}` | RO (JWT + SpiceDB) | Fetch individual Form 34A detail |
| `POST /rts/v1/ro/form34b` | RO (JWT + SpiceDB) | Submit compiled + signed Form 34B |
| `PATCH /rts/v1/ro/form34b/{id}/edit` | RO (JWT + SpiceDB) | Append RoEditEntry (biometric-gated) |
| `POST /rts/v1/ro/form34b/{id}/disputes` | Agent (JWT + SpiceDB) | Raise dispute case |
| `PATCH /rts/v1/ro/form34b/{id}/disputes/{cid}` | RO (JWT + SpiceDB) | Resolve dispute |
| `POST /rts/v1/ro/form34b/{id}/countersign` | Agent (JWT + SpiceDB) | Agent countersign |
| `POST /rts/v1/ro/form34c` | County RO (JWT + SpiceDB) | Submit Form 34C |
| `POST /rts/v1/ro/session-recording-hash` | RO (JWT + SpiceDB) | Submit SHA-256 of recording to tlog |

## 6. Human surfaces

| Surface | Path | Purpose |
|---|---|---|
| RO Tauri desktop app | `services/rts-svc/apps/` (Tauri + web frontend) | Form 34B/34C compilation, edit log, dispute, agent countersign |
| Agent countersign panel | Same Tauri app (agent role) | Party agent review and countersign |
| RO view projection | `services/rts-svc/crates/rts-projections/src/ro_view.rs` | Read model feeding Tauri app |

Languages: English + Kiswahili. WCAG 2.1 AA. Offline-capable: Tauri app bundles RO view snapshot; full submission requires connectivity.

## 7. Integrity + anti-tamper

- **RO Ed25519 signature** on Form 34B binding form hash + RO identity + HLC; tampered forms detected by M10.
- **Immutable edit log** -- every RoEditEntry appended with RO biometric confirmation; no in-place edit; full audit trail reconstructable.
- **Agent countersign** -- minimum agent signature count enforced before submission; each agent biometric-auth; agent identity + sig stored in AgentCountersign.
- **Session recording hash** -- SHA-256 of tallying session video submitted to transparency log (P05) via `services/transparency-svc/crates/tlog-api/`; leaf ID stored for later verification.
- **Dispute case lifecycle** -- all disputes resolved before Form 34B submission; unresolved disputes block submission.
- **Replay protection** -- `eel_idempotency_key` per form submission; M10 returns cached 200 on duplicate.
- **HSM countersignature on M10** -- final Form 34B countersigned by HSM after M10 accepts; RO sees confirmation.
- **TLA+ Tally spec** (`services/rts-svc/crates/declaration/src/proptest_rules.rs`) -- tally invariants verified by proptest; no subtraction of votes; no constituency total exceeding station sum.

## 8. Kenya statute alignment

- **Elections Act s.39 (tally declaration)** -- Form 34B is the legal constituency tally declaration; M09 is the tool for its digital compilation; paper form remains the canonical legal document.
- **Elections Act s.41 (agent access)** -- agent countersign path enforces that registered party agents participate in tally review; dispute mechanism is the statutory challenge path.
- **Elections (General) Regulations Reg. 87 (forms and tallies)** -- edit log with mandatory reason codes documents every deviation from ICR-extracted tallies.
- **Elections (Technology) Regulations 2017** -- signed tally + session recording hash provides the certifiable electronic evidence chain.

## 9. Honest caveats + external prerequisites

- **Tauri desktop app** requires Windows / macOS / Linux distribution to RO offices; installer packaging and code-signing is an ops prerequisite.
- **Session recording** capability requires camera/screen-capture permissions on RO laptops; privacy notice and consent required per KDPA.
- **Agent countersign count** -- minimum count is configurable; value must be agreed with the Commission legal before G3.
- **Form 34C** (county tally) depends on all constituency Form 34Bs being received; any incomplete Form 34B delays county declaration.
- RO view projection (`rts-projections/src/ro_view.rs`) is a read model; correctness depends on M10 event-sourced stream being consistent; any M10 outage during tallying must be resolved before RO can compile Form 34B.

## 10. Test coverage + operational runbooks

| Layer | Harness | ID prefix |
|---|---|---|
| Unit (edit log append, agent countersign validation, dispute FSM, tally sum invariants) | nextest | M09-UNIT- |
| Property (tally invariants: no subtraction, no exceed, edit log immutable) | proptest (`declaration/src/proptest_rules.rs`) | M09-PROP- |
| Contract (M10 form34b ingest; tlog session recording hash) | Pact | M09-PACT- |
| Integration (RO view projection; M10 double; tlog double) | nextest + testcontainers | M09-INT- |
| E2E (full RO tally flow in Tauri staging; agent countersign; dispute resolution) | Playwright + Tauri test | M09-E2E- |
| Security (biometric-gate bypass; edit-log mutation attempt; agent impersonation) | ZAP + manual | M09-SEC- |

Runbook (to create): `docs/runbooks/rts-ro-degradation.md` (Form 34B submission failure; missing station tally; agent countersign timeout).

Full spec: `docs/plans/modules/rts-ro.md`. Crates: `services/rts-svc/crates/rts-ro-api/`, `declaration/`, `rts-projections/`.
