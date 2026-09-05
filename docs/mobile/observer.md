# Observer App — Complete Reference (M34)

**Code:** `services/field-mobile-svc/apps/observer/`
**Module spec:** M34 · **Plan:** [`docs/mobile/plan.md`](plan.md)
**Contract:** `packages/openapi-contracts/field/observer.yaml` (11 BFF paths)
**Design system:** `packages/design-system/flutter/lib/src/`

---

## Purpose

The Observer app is the independent verification tool for accredited observers. It performs station-level sampling, verification, incident reporting, and countersignature. Every observation is signed, timestamped, and anchored to a cryptographic proof chain.

---

## Functions (20 Features)

### F1 — Enrolment (FR-001)
Accreditation ID + biometric enrol + device attestation. Category (CHIEF, FIELD, STATION) is set at enrolment and is **immutable**.

### F2 — MFA (FR-002)
FIDO2 or platform biometric + attestation verdict + PIN fallback (5-fail lockout, 30 min).

### F3 — Category-Gated Access (FR-003)
SpiceDB drives the feature matrix. `CHIEF` sees all; `FIELD` sees visit + verify + incident; `STATION` sees checklist + visit. Countersign rights are shown only when permitted.

### F4 — Portal Read (FR-004)
Results browser: national → county → constituency → station. Each row shows published badge + proof status ring. Signed Form 34A images load from CDN with SHA-256 verification.

### F5 — Per-Station Merkle Proof (FR-005)
`EelVerifierFfi` (Rust FFI) verifies inclusion proof + form proof. Client-side verification completes in ≤ 60 s on mid-tier device.

### F6 — Root Hash Pinning + Tamper Detection (FR-006, 026)
Root pinned in `assets/config/root.json` at release. Mismatch → automatic `TAMPERING` incident (pre-filled with station, observed hash, root). Badge = `Unverifiable`, sticky until fresh verification succeeds.

### F7 — Station Bookmarks (FR-007)
Watch list + publish/update push. Bookmarks persisted in Isar; sync via push subscription.

### F8 — Bulk Constituency Download (FR-008)
Rate-limited per category/hour from policy bundle. Sequential chunk download (forms + proofs) into Isar blob store. Resumable ranges; per-item retry; storage-full guard.

### F9 — GPS Station Visit (FR-009–011)
Point-in-polygon validation against station polygon. Distance shown. Signed visit log with timestamp. Session timer on visit screen.

### F10 — Time-Boxed Session + Biometric Sign-Off (FR-010)
Session starts at enrolment. Biometric sign-off (fresh capture) required on exit.

### F11 — Structured Incidents (FR-012–013)
7 categories. Evidence: photo/video/audio. GPS consent-gated. Encrypted queue; AG mirror on backend.

### F12 — Sampling (FR-016)
Cryptographic seed = `SHA-256(protocol_version ‖ constituency ‖ election_id)`. Fisher–Yates shuffle over station list; take `n` from policy. Deterministic across reinstall (pure function).

### F13 — Verification Checklist (FR-017)
4 Boolean items + notes. Each item shows per-row match (defaults pre-ticked from OCR). Verify button disabled until confirmed or discrepancy selected.

### F14 — Countersignature (FR-018)
Category-gated. Anchored to M05. Biometric + PIN step-up; `EelSigningKey` (device-bound Ed25519); `POST .../countersign` with UUID7. Once `signed`, state is sealed (immutable union).

### F15 — End-of-Mission Report (FR-019–020)
Auto-compiled + server-rendered PDF (`GET .../end-of-mission/pdf`). Client caches + stores SHA-256.

### F16 — Offline Cache (FR-022)
Snapshots + proofs cached with stale-since display (`EelStatusChip`). Verification works offline against cached root.

### F17 — Systemic Incident Flag (FR-015, 024)
Exactly-once. SMS/USSD fallback for `CHIEF`. Deep-linkless minimal payload to reserved short code.

### F18 — Security Envelope (FR-025, 027)
mTLS pinning, `FLAG_SECURE` on incident screens, biometric per submission, secure store.

### F19 — Locales (FR-028)
EN/SW launch. FR/PT pack on-demand (signed archive, verify before load).

### F20 — WAL Integrity (FR-030)
Kill-resume preserved. Offline events queued with WAL journal; replay on reconnect.

---

## Screen-by-Screen UX

| Screen | Components |
|---|---|
| **Login** | `EelScaffold`, `EelSealStamp` (attestation icon), `EelTextField` (agent ID, PIN), biometric button, category badge |
| **Status** | `EelStatusChip` (category + role), `EelResultCard` (mission summary, session timer) |
| **Portal (Results)** | Tab nav (national/county/constituency/station), `EelResultCard` per station with proof ring |
| **Form 34A Viewer** | Image viewer (pinch zoom), hash footer, proof badge |
| **Verify** | 4-item checklist (`EelButton` toggle rows), notes (`EelTextField`), proof result (`EelStatusChip`: verified/unverifiable/invalid) |
| **Sampling** | Constituency card (`EelStatusChip`: n=... sampled), regenerate disabled, seed chip |
| **Visit** | Check-in card (`EelButton`: start session), polygon map pin, session timer (`EelStatusChip`), sign-off (biometric) |
| **Incident** | 7-category grid (`EelChips`), `EelTextField` (description), evidence (`EelButton` attachments), GPS consent toggle |
| **Countersign** | Category-gated `EelButton`, HSM-backed signature, sealed-state display |
| **Bookmarks** | Watch list (`EelResultCard` per station), publish/update button |
| **Mission Export** | Stats grid (`EelStatusChip`), PDF download (`EelButton` with download icon) |
| **Systemic Flag** | Confirm dialog (destructive style), SMS fallback (`EelBanner` info) |

---

## State Machine

`Unauthenticated → Bootstrapping → HardAttestationFailed | Authenticated → Active → OnMission → SignedOff → EomSubmitted`

Within `Active`: `Idle → CheckIn (GPS verified) → OnStation → VisitActive → VisitComplete (biometric sign-off) → CountersignPending → CountersignSigned`.

---

## Design System Reference

Every screen uses `EelScaffold` (not bare `Scaffold`), `EelButton` (primary/outlined/tertiary, fullWidth, icon), `EelResultCard` (leading icon + title/subtitle), `EelStatusChip` (ok/warning/critical/info with severity colors), `EelTextField` (label/hint/maxLines/obscureText), `EelBanner` (message + severity), `EelChips` (filter/action groups), `EelSealStamp` (trust indicator), `EelTextStyle` (displayMedium/titleMedium/bodyMedium/bodySmall), `EelColors.of(context)`.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
