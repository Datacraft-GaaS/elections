# Custody Mobile (M65) — Implementation Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/diaspora-ballot-physical-logistics.md` (packing app
surface). Backend: `packages/openapi-contracts/custody/openapi.yaml` has 9
unbound paths (allowlist) — bind in inventory-svc. Est: 3 ED + backend
binding.

## Current state

`services/inventory-svc/apps/custody-mobile/` — 3 lib files. Real:
transport-driven RFID reader services (Zebra RFD40/CipherLab RS36 via
platform channels + synthetic for CI), scan screen, HMAC challenge concept.
Missing: packing workflow (FR-M65-001…009), photo/GPS/seal capture, signing
WAL, locales (EN/FR/SW/AR required by FR-M65-010), backend binding.

## Target features

| # | Feature | FR (M65) |
|---|---|---|
| F1 | Packing workflow initiation per post | 001 |
| F2 | Envelope scan + count per type; manifest totals | 002 |
| F3 | ISO 17712 H-class seal UID record (from M20 inventory) | 003 |
| F4 | Sealed-container photo + SHA-256 in event | 004 |
| F5 | GPS of packing location | 005 |
| F6 | Container UID format KE27-DIA-{POST}-{SEQ} | 006 |
| F7 | Dual signature: Embassy Election Officer + IAB observer (remote signed confirm fallback) | 007 |
| F8 | TPM-signed, device-encrypted WAL; sync on reconnect; idempotent double-sync | 008–009 |
| F9 | Locales EN/FR/SW/AR with ICU plurals | 010 |
| F10 | RFID cover scan verification (existing) wired to container events | inventory |

## Files

```
lib/src/core/*                    NEW: eel-core composition (db/outbox/config/tamper)
lib/src/features/packing/         packing_screen (wizard), envelope_scan_step, seal_step, photo_step, gps_step, signature_step
lib/src/features/manifest/        manifest_screen, manifest_model.dart
lib/src/features/containers/      container_registry_screen, uid_generator.dart
lib/src/features/rfid/            wire existing scan → container attach flow
lib/src/i18n/                     fr + ar ARBs; RTL
lib/src/services/signing.dart     kit-sdk signing + TPM-backed key via platform (device-gated fallback software key)
test/packing_flow_test.dart, uid_test.dart, manifest_test.dart, ...
```

## Technical implementation

- **Packing wizard** (`EelStepFlow`): 6 steps (post → envelopes → seals →
  photo → gps → signatures); each step journals before proceeding; packing
  event assembled as canonical JSON + Ed25519 (TPM key where hardware
  present — platform channel `eel.custody.tpm/sign`; fallback software key
  with `attestation_tier=software` marked, mirroring legacy-android
  convention).
- **Seal UID**: fetch from M20 inventory (BFF) by scanned barcode; verify
  not-consumed; consume on attach (server-side atomic).
- **Remote IAB witness**: if no local observer, app generates witness
  request (QR/deep-link for observer app or web confirm page); confirm
  signature attached later — event has `witness_pending` until then
  (server-side state, not local mutation of signed record).
- **RFID attach**: existing reader service scan → container-cover tag EPC
  bound to container UID; tamper-bit reads recorded in event stream.

## Algorithms

- UID generator with per-post sequence from server allocation (offline:
  pre-allocated ranges synced with MDM config).
- Photo hashing pipeline: EXIF-stripped re-encode → SHA-256 → store hash +
  blob; blob uploaded via presigned MinIO (BFF).
- Idempotent packing sync: batch id = UUID7 of manifest; re-send → cached
  receipt (EC-DAT-01).

## Backend touchpoints

- Bind custody `openapi.yaml` 9 paths in inventory-svc: packing events,
  container registry, seal consume, witness confirm; event schemas under
  `event-schemas/custody/`.

## UX / UI / ergonomics

- Wizard with big scan targets; count stepper per envelope type; seal
  entry with barcode scan + manual override + verification prompt; photo
  step auto-captures + shows hash; signature step = two-panel (local
  officer, IAB) with remote-witness fallback path clearly labeled; manifest
  review with edit-before-sign; after sign — immutable receipt + QR.
- FR/SW/AR/EN complete; AR RTL; ICU plurals in counts ("3 envelopes").
- Offline-first: full flow no network; sync screen with batch state.

## Testing

- Unit: UID allocation, manifest signing, idempotent sync, hash pipeline.
- Widget: wizard states incl. witness_pending; manifest review; RTL goldens.
- Integration: pack → seal → sign → offline → sync dedup; kill-drill.
- Contract: Pact for custody paths; idempotency + DAST on bindings.
- A11y: traversal + pseudo; 4 locales complete (release blocker: en/sw/fr/ar).

## Definition of done

Master §9; 9 custody paths bound (allowlist cleared); FRs tested; device
RFID paths self-checked; README updated.
---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
