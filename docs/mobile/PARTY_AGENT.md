# Party Agent App — Complete Reference (M30)

**Code:** `services/field-mobile-svc/apps/party-agent/`
**Module spec:** M30 · **Plan:** [`docs/mobile/plan.md`](plan.md)
**Contract:** `packages/openapi-contracts/field/party-agent.yaml` (10 BFF paths)
**Design system:** `packages/design-system/flutter/lib/src/`

---

## Purpose

The Party Agent app is the field tool for accredited party agents. It verifies Form 34A, countersigns results, files disputes, and reports incidents. Every action is biometrically attested and cryptographically signed with a device-bound HSM key.

---

## Functions (13 Features)

### F1 — Enrolment (FR-001)
Accreditation ID + party token + biometric enrol. Station assignment set at check-in.

### F2 — Station-Bound MFA (FR-002–004)
Hard attestation required. Platform biometric + PIN (5-fail lockout, 30 min). Station-bound: agent can only operate at their assigned station.

### F3 — GPS Check-In (FR-005–007)
Haversine ≤ 200 m from station coordinates. If outside → rejected with distance display. Override requires supervisor (backend). PO notified via `agent_checked_in` event.

### F4 — Station Dashboard (FR-008–009)
Aggregates only (turnout, auth count). Configurable delay. Strictly read-only.

### F5 — Form 34A Verify Cycle (FR-010–016, 026, 028, 029)
The core workflow:
1. **QR Scan** (`mobile_scanner`) → parse form ID + station
2. **Fetch**: `GET /v1/agent/forms/34a/{id}` → signed record + signature chain + Merkle proof
3. **Verify**: `EelVerifierFfi` (Rust FFI via JNI) → proof + chain verification
4. **Compare**: digital values vs physical form side-by-side; per-row match toggles
5. **Countersign**: biometric + PIN step-up → `EelSigningKey` (TEE-backed Ed25519) → `POST .../countersign` with UUID7

**Offline path**: cached record verification works offline; countersign outboxed and replayed on reconnect.

**Once signed**: `CountersignState.signed` is a sealed union — controller rejects any mutation (FR-035).

### F6 — Dispute Form (FR-016)
Mandatory reason textarea + category chips + evidence. Exactly-once: client UUID7 + server idempotency store. Confirmation sheet states escalation chain. Immutable receipt after submit.

### F7 — Chief-Agent Rollups (FR-017–019)
Fetch 34B (constituency) / 34C (national). Client recomputes aggregate hash from contributing form hashes. Compares to published Merkle root via FFI. Mismatch → dispute sheet pre-filled.

### F8 — Incident Reporting (FR-020–023)
7 categories. Evidence: photo/video/audio. Consent-gated GPS. Escalation to tally + war room.

### F9 — Push Notifications (FR-024–025)
Station events, rollup publications, broadcasts. No PII payloads. Subscribe via BFF token endpoint.

### F10 — Duress Alert (FR-030)
Alternate PIN (set at enrolment) opens normal-looking session with `duress=true` flag. All actions proceed. Silent alert outboxed with `type=DURESS`. Exactly-once (server dedup). Long-press on Verify for 3 s (haptic only) triggers same path.

### F11 — Security Envelope (FR-031–034)
RASP, screen-capture protection (`FLAG_SECURE` everywhere), secure store (Isar encrypted), UUID7 per operation.

### F12 — Immutable Countersignature State (FR-035)
`CountersignState` sealed union in Riverpod. Once `CountersignState.signed`, all mutations rejected. State survives app restart (Isar persistence).

### F13 — MDM Force-Update (FR-036)
Floor version check at bootstrap. App blocked below minimum version.

---

## Screen-by-Screen UX

| Screen | Components |
|---|---|
| **Login** | `EelScaffold` (centered), `EelSealStamp`, `EelTextField` (agent ID, PIN), biometric button |
| **Dashboard** | `EelScaffold`, `EelStatusChip` (station/party), `EelChips` (agent status), `EelResultCard` (aggregates) |
| **Check-In** | `EelTextField` (station code), `EelButton` (GPS verify), distance display, `EelBanner` (outside range) |
| **Forms** | `EelScaffold`, `EelResultCard` (form list), `EelButton` (view details) |
| **QR Verify** | `EelScaffold`, `EelButton` (start scan), alignment frame, 3-step progress rail |
| **Verify Result** | `EelResultCard` (proof status: VERIFIED/FAILED), signature chain list with checkmarks |
| **Countersign** | `EelButton` (sign now), sealed-state display, HSM badge |
| **Dispute** | `EelTextField` (reason, mandatory), `EelChips` (category), `EelBanner` (escalation), `EelButton` (submit) |
| **Rollup** | `EelStatusChip` (progress: 137/180), `EelResultCard` per position, verify/dispute buttons |
| **Incidents** | 7-category grid (`EelChips`), evidence attachments, GPS consent |
| **Duress Alert** | `EelBanner` (silent mode), `EelButton` (confirm silent) |
| **QR Verify Result** | `EelResultCard` (status badge), signature chain, compare rows |

---

## State Machine

`Unauthenticated → Bootstrapping → HardAttestationFailed | Authenticated → Active → OnShift → OnStation → VerifyPending → CountersignPending → Countersigned → DisputePending → Disputed`

Duress path: any `Active` state with `duress=true` flag (visually identical to normal path).

---

## Algorithms

- **Haversine** for GPS validation (≤ 200 m threshold)
- **Canonical JSON** (RFC 8785) before signing/verification
- **Merkle inclusion proof** via Rust FFI; pure-Dart fallback for CI parity
- **Fisher-Yates shuffle** with SHA-256 seeded RNG (for sampling if extended)
- **Exactly-once**: client UUID7 + server idempotency; re-send returns cached; body mismatch → 422 conflict

---

## Design System Reference

Every screen uses `EelScaffold` (not bare `Scaffold`), `EelButton` (primary/outlined/tertiary, fullWidth, loading, icon), `EelResultCard` (leading icon + title + subtitle, elevation), `EelStatusChip` (ok/warning/critical/info severity), `EelTextField` (label/hint/maxLines/obscureText), `EelBanner` (message/severity), `EelChips` (filter/action groups), `EelSealStamp` (HSM/trust indicator), `EelTextStyle` (displayMedium/titleMedium/bodyMedium/bodySmall), `EelColors.of(context)`.
