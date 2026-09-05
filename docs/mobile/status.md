# Mobile apps completeness audit

Updated 2026-09-03. Scope: all 18 Flutter apps under `services/*/apps/` plus
`packages/kit-sdk-flutter`. Findings measured against the repo's own
completeness procedure (`docs/plans/11-completeness-audit.md`, scanner
`tools/completeness`) and the module specs in `docs/plans/modules/`.

## Verdict

6 production-grade, 3 partial, 3 scaffolds posing as apps, plus the SDK. The 3
field-mobile persona apps (poll-clerk, observer, party-agent) are the weakest —
they pass the repo's own scanner but are mostly placeholder UIs, and one stub
violates the repo's own security ADR.

## Production-grade (real logic, real backends, tests)

| App | State |
|---|---|
| `packages/kit-sdk-flutter` | Core SDK: Ed25519 signing, attestation tiers, telemetry (hash chain, redaction, sampling), offline queues. 15 test files. Stub attestation client is documented as "wired at kit runtime". |
| `bvr-svc/apps/cvr-field-officer` | Real API client (SVID header, idempotency keys, `/v1/cvr/field-notifications` matches backend), OCR, geo, duress, 6 test files. Real FAP60 fingerprint driver. |
| `field-mobile-svc/apps/personnel` | Real `/v1/dispatch/*` integration matching actual dispatch-api routes, workforce duty, notifications, signing, GPS. Placeholder biometric hash is documented deferral D2 (blocked on kit pairing). |
| `inventory-svc/apps/custody-mobile` | Real transport-driven RFID (Zebra RFD40 / CipherLab RS36 via platform channels), synthetic adapter for CI. |
| `rts-svc/apps/rts-kit` | Real Ed25519 signing, mTLS HTTP transmit, fails closed without anchors; SMS fallback stub is documented. Queue provider properly overridden in `main.dart`. |
| `training-svc/apps/lms-offline`, `accred-scanner` | Real sync engines (connectivity-aware, idempotent), SQLite repos with codegen. |

Kits with partial simulation:

- `bvi-kit`: GNSS/time-fence logic real; NTS time, GNSS readings and capture
  simulated (`identification_workflow.dart`).
- `diaspora-kit`: offline queue with deferred transmission real, but
  `hsmSignature: 'stub-kit-signature-...'` is fabricated
  (`form34a_screen.dart:56`).
- `bvr-kit`: fingerprint real; the MRZ → IPRS → sign → upload path in
  `enrolment_workflow.dart:103-109` is entirely `Future.delayed` stubs.

## Critical finding: attestation bypass in 3 persona apps

`poll-clerk`, `observer`, and `party-agent` each ship an identical
`AttestationService` that always returns `passed: true, token: 'stub-token'` —
directly contradicting the ADR-0021 comment above it ("hard-fail MDM
attestation before any auth"). Worse, `auth_provider.dart:27` in all three logs
in after a 300 ms delay with no backend call at all. For an election platform,
device attestation silently bypassed on polling-day devices is the single most
serious gap. Related: their `LocalDbService` is a no-op (no Isar/SQLCipher, so
"offline-first" is not real), and `ApiService` (Dio → `bff.eel.internal`) is
defined but referenced by zero screens.

## Literal stub screens (UI is `Text('... stub')`, no logic)

- **poll-clerk**: 4 of 5 screens — incident, kiosk, EOD report, station dashboard.
- **observer**: all 4 screens — incident, station visit, mission export, tally verify.
- **party-agent**: all 4 screens — QR verify, countersign, check-in, dispute
  (+ empty duress service body).
- **field-support**: all 4 screens are static text; `TicketService` exists and
  its backend routes (`/v1/support/*`) exist, but the service never constructs
  a baseUrl and no screen calls it.

## Structural gaps

1. **No deployable artifacts**: only `bvr-kit` and `bvi-kit` have `android/`
   scaffolds; 14 of 18 apps cannot build an APK. Audit step 9.2 ("assert
   deployable artefact exists") fails for them.
2. **Missing contract**: module spec M31 references
   `packages/openapi-contracts/field/poll-clerk.yaml` — the directory does not
   exist. The 287 unbound-path deferral (status.md item 14) includes
   field-mobile-bff's 18 paths, so poll-clerk/observer/party-agent have no
   bound API to integrate against even if their UIs were real.
3. **Scanner blind spot**: `tools/completeness` catches `TODO` /
   `UnimplementedError` in Dart but not `Text('Incident submission stub')`
   UIs — which is why these apps pass the CI gate. 22 committed vendor-Java
   TODOs in `libUvcFinger` also are not allowlisted.
4. **Test depth**: persona apps have 1–2 tests each, both just "app renders
   without crash". No test asserts any feature behavior.
5. **`legacy-android`** is a documented shim (PARITY.md is honest about it),
   but its two buttons are `onPressed: () {}` — dead.
6. CI runs `flutter test` per app (good) but not `flutter analyze`
   (currently clean, ungated).

## Recommended priority

1. Replace the three `AttestationService` stubs with real Play Integrity /
   App Attest or fail closed (security).
2. Wire the BFF / attestation / local-db trio in poll-clerk, observer,
   party-agent, or strip the apps to honest skeleton status in the module
   inventory.
3. Implement the `poll-clerk.yaml` contract and bind the 18 field-mobile-bff
   paths.
4. Add `flutter analyze` to the CI matrix; extend the scanner to flag
   `'... stub'` UI strings.
---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
