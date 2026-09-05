# CVR Field Officer (M35) — Completion Plan

Master plan: `docs/mobile/plan.md`. Module spec:
`docs/plans/modules/cvr-continuous-voter-register.md`. Backend:
`packages/openapi-contracts/cvr/register.yaml` (4 paths; handler scaffold
exists per status.md item 12). Est: 3 ED.

## Current state

`services/bvr-svc/apps/cvr-field-officer/` — 18 lib files, 6 test files,
real API client (SVID header, idempotency), OCR client, attestation facade
(kit-sdk), duress, geo, offline queue. Gaps: `nationalId`/`kitId` empty
strings "populated in production" (api_client.dart:29-33), no Isar (in-memory
queue "until Isar codegen lands"), no l10n, no pinning, no attestation
backend verdict.

## Target features (completion)

| # | Feature | FR |
|---|---|---|
| F1 | Real ID extraction: OCR result → nationalId; kit-sdk device id → kitId | close placeholders |
| F2 | Durable encrypted queue (Isar) replacing in-memory stub | FR-M35 offline |
| F3 | SPKI pinning + attestation verdict endpoint | security |
| F4 | 11-locale UI | parity |
| F5 | Capture quality gates + review flow hardening | spec |
| F6 | Sync engine → kit-sdk EelSyncEngine migration with conflict hook | architecture |

## Files

```
lib/src/services/api_client.dart     fill nationalId/kitId from OCR + AppIdentity; error typing problem+json
lib/src/services/offline_queue.dart  REPLACE with kit-sdk EelEncryptedIsarStore-backed queue (journal-before-act)
lib/src/services/attestation.dart    swap stub client for real client (dev flavor keeps stub)
lib/src/services/ocr_client.dart     contract-test against bvr OCR endpoint
lib/src/l10n/                        generate 11 ARBs from packages/i18n
lib/src/core/connectivity.dart       captive-portal probe + banners
test/... extend (queue durability, idempotency, pinning)
```

## Technical implementation

- Replace placeholder payload fields: `nationalId` from `record.ocrResult`
  parsed MRZ/ID fields; `kitId` from kit-sdk `EelSecureStore` device
  identity (attested at boot).
- Queue: outbox collection (Isar) with states pending/syncing/synced/failed/
  dead; UUID7 idempotency already present; kill-drill harness.
- Migrate onto eel-core `ApiClient` (pinning + retry + attest) keeping the
  role header `X-Eel-Role`.

## Algorithms

- Reuse kit-sdk sync engine conflict hook: server-wins for dedup verdicts,
  user-prompt for capture conflicts.

## Backend touchpoints

- cvr register.yaml bindings (field-notifications exists) + OCR endpoint
  contract; remove allowlist rows as bound.

## UX / UI / ergonomics

- Existing screens (home/capture/review/history/sync) are real — polish:
  capture quality guidance overlay, review diff sheet (OCR vs corrected),
  history with sync state chips; localized copy for all states; offline
  banner; biometric step-up before submit.

## Testing

- Unit: payload completeness, queue durability (kill-resume), idempotency
  keys, pinning adapter (mock socket), tier mapping.
- Widget: capture→review→sync states; 11 locales goldens.
- Integration: capture → offline → reconnect dedup; kill-drill.
- Contract: Pact for cvr paths.

## Definition of done

Master §9; empty-string placeholders eliminated; Isar queue live; scanner
green; README parity.
---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
