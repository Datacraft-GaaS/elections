# Party Agent App (M30) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/mobile-party-agent.md` (36 FRs). Contract to author:
`packages/openapi-contracts/field/party-agent.yaml` (10 paths per module
table). Est: 10 ED.

## Current state

`services/field-mobile-svc/apps/party-agent/` — 28 lib files, all 4 feature
screens are `Text('... stub')`; `DuressAlertService.sendSilentAlert` empty;
`LocalDbService` no-op; auth fake; attestation stub. 23 locales present
(en/sw + 9 Kenyan + pseudo).

## Target features

| # | Feature | FR |
|---|---|---|
| F1 | Enrolment (accreditation ID + party token + biometric enrol) | 001 |
| F2 | Station-bound assignment + attestation hard-fail + MFA | 002–004 |
| F3 | GPS check-in (≤ 200 m validation) + PO notification + party badge | 005–007 |
| F4 | Station dashboard (aggregates only, configurable delay) | 008–009 |
| F5 | Form 34A verify cycle: QR scan → fetch signed record + signature chain → compare → Merkle proof via Rust FFI → countersign (HSM-backed, biometric+PIN) | 010–016, 026, 028, 029 |
| F6 | Dispute form (mandatory reason, exactly-once event, escalation to tally + war room) | 016 |
| F7 | Chief-agent rollups 34B/34C + verify + dispute | 017–019 |
| F8 | Incident reporting (obstruction/tampering/intimidation/kit/campaigning/other; photo/video/audio; consent-gated GPS) | 020–023 |
| F9 | Push notifications (station events, rollup publications, broadcasts; no PII payloads) | 024–025 |
| F10 | Duress code → silent war-room alert exactly-once, UI unchanged | 030 |
| F11 | RASP, screen-capture protection everywhere, secure store, UUID7 | 031–034 |
| F12 | Immutable countersignature state after HSM sign (Riverpod) | 035 |
| F13 | MDM force-update block below floor version | 036 |

## Files

```
lib/main.dart                        rewrite bootstrap
lib/src/core/*                       eel-core composition (bootstrap, session, api, db, outbox, tamper, config, connectivity)
lib/src/features/auth/               login, session_controller, lockout
lib/src/features/check_in/           check_in_screen, gps_validator, party_badge_screen
lib/src/features/dashboard/          station_dashboard_screen (aggregates)
lib/src/features/verify/             qr_scan_screen, form34a_view, signature_chain_widget, verify_flow_controller
lib/src/features/countersign/        countersign_screen, hsm_client.dart
lib/src/features/dispute/            dispute_screen (form + rollup variants)
lib/src/features/rollup/             rollup_screen, rollup_verify_controller
lib/src/features/incident/           incident_screen, evidence (photo/video/audio)
lib/src/features/duress/             duress_service (real: kit-sdk hook + outbox), duress_pin_gate
lib/src/features/messages/           status + broadcast inbox (push-driven fetch)
lib/src/services/local_db_service.dart  DELETE
lib/src/services/attestation_service.dart DELETE
test/..., integration_test/..., test/golden/...
```

## Technical implementation

- **F5 verify cycle** (the heart): 
  1. `mobile_scanner` QR → parse form id + station.
  2. `GET /v1/agent/forms/34a/{id}` (BFF) → signed record incl. signature
     chain [PO sig, kit sig, backend countersignature] and Merkle proof.
  3. Client verification via kit-sdk `EelVerifierFfi` (libeel-verifier; Android
     JNI + iOS XCFramework from packages/verifier-rs) — proof + chain.
  4. Comparison screen: digital values vs physical form side-by-side.
  5. Countersign: biometric+PIN step-up → kit-sdk `EelSigningKey` (device-bound
     Ed25519; TEE-backed where available) → `POST .../countersign` with
     UUID7. State: `CountersignState` is a sealed union — once `signed`,
     controller rejects any mutation (FR-035). Offline path: cached record
     verification works offline (FR-026); countersign outboxed (FR-027).
- **F3 GPS validation**: haversine ≤ 200 m against station coords from
  assignment; if outside → rejected with distance + "move to station" copy;
  override only via supervisor (backend). Notification to PO is a BFF
  side-effect (event `agent_checked_in`).
- **F7 rollups**: fetch 34B per constituency / 34C; client recomputes
  aggregate hash from contributing form hashes (list in payload) and compares
  to published Merkle root via FFI; mismatch → dispute sheet auto-filled.
- **F10 duress**: alternate PIN (set at enrolment) opens a normal-looking
  session with `duress=true` flag; all actions proceed normally; a one-time
  encrypted alert is outboxed with type DURESS exactly-once (server dedups);
  long-press alternative triggers same path (kit-sdk `EelDuressHook`).
- **F5 QR from kit screen vs paper**: capture both; from-kit mode dims
  brightness tip; low-light guidance overlay.

## Algorithms

- Haversine + polygon tolerance for check-in (reuse eel-core geo utils).
- Canonical JSON (RFC 8785) serialization before signing/verification;
  signature chain walk (leaf → intermediates → root pin) with explicit
  per-hop error codes displayed in UI.
- Merkle inclusion proof verification (FFI); fallback pure-Dart verifier for
  web/CI parity tests (kit-sdk has basis).
- Exactly-once dispute: client UUID7 + server idempotency store; re-send
  returns cached response (EC-DAT-01); body mismatch → 422 conflict sheet.

## Backend touchpoints

- `party-agent.yaml` (10 paths incl. rollups) bound in mobile-bff + rts-svc
  pass-through; push tokens; `form_disputed` exactly-once producer+consumer
  tests (event-schema exists under `event-schemas`).

## UX / UI / ergonomics

- **Tab shell**: Verify · Rollups · Incidents · Status; badge screen from
  check-in persists as app-bar avatar (party color band + name + station).
- **Verify flow**: camera view with alignment frame; after scan, 3-step rail
  (Fetched → Proof OK → Countersigned); signature chain as vertical list with
  checkmarks and failure reasons; compare screen uses side-by-side
  value/field pairs with per-row match toggles (defaults pre-ticked from OCR
  where legible, never silently); Countersign button disabled until all rows
  confirmed or discrepancy chosen.
- **Dispute**: reason textarea mandatory + category chips + evidence
  attachment; confirmation sheet states exactly what is submitted and that it
  escalates to war room; after submit — immutable receipt.
- **Rollups**: per-position cards with progress ("34B: 137/180 stations
  published"); verify button per card with result ring; dispute per rollup.
- **Duress**: zero visual difference; long-press on Verify for 3 s (haptic
  only) documented in the security runbook, not in user docs.
- **Offline**: verify tab shows cached records with "offline — proof verified
  against cached root" caption; outbox chip with pending count.
- States matrix per master §5.3; goldens per state; a11y traversal of the
  verify cycle is the primary a11y test.

## Testing

- Unit: GPS validation boundary (199/200/201 m), rollup recompute, duress
  exactly-once, countersign immutability (state machine), UUID7 format.
- Widget: verify cycle states incl. `proof-failed` (auto TAMPERING incident
  path FR-026 analog: FR-M30-014 context), dispute sheet, offline badges.
- Integration: full scan→verify→countersign ≤ 30 s (FR-028) on LTE profile
  (toxiproxy), offline countersign → reconnect dedup, kill-drill.
- Contract: Pact all 10 paths. Security: rooted AVD + Frida block (FR-031),
  screenshot block test, no-PII push payload test.
- A11y: traversal of verify + dispute; 48 dp targets; 200 % text scale.

## Definition of done

Master §9; module FRs 001–036 with FR-tagged tests; AC rows in module spec
ticked; contract bound; duress tested in CI with synthetic sink; MDM staging
install verified.