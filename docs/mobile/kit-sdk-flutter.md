# kit-sdk-flutter — Hardening Plan

Master plan: `docs/mobile/plan.md`. Package: `packages/kit-sdk-flutter`
(+ `example/`). Est: 6 ED (subset counted in Wave M0).

## Current state

Production-grade core: Ed25519 `EelSigningKey`, `EelSecureStore`,
`offline_queue`, `sync_engine`, telemetry stack (hash chain, redactor,
adaptive sampling, drop detector, session recorder), HLC, time fence,
`verifier_ffi`, `abis_kit_client`, fingerprint/iris scanner abstractions,
SPKI pinning noted but implemented where? — `http_client.dart` is 91 lines
with NO pinning despite pubspec comment claiming a custom adapter. Att:
`EelAttestationClientStub` only.

## Work items

| # | Item | Details |
|---|---|---|
| K1 | SPKI pinning adapter | Custom `HttpClientAdapter` for dio: fetch pin-set (config), validate chain SPKIs, dual-pin rotation window, fail-closed; test with mock `HttpClientAdapter` + real socket integration test vs self-signed roca CA fixture |
| K2 | Real attestation clients | `PlayIntegrityAttestationClient` (integrity API classic + verdict endpoint call), `AppAttestAttestationClient` (DeviceCheck keygen/attest/assertion); both behind `EelAttestationClient`; platform channels `eel.attest/android` + `/ios`; stub only in dev flavor |
| K3 | `EelEncryptedIsarStore` | Isar open with AES key from secure store; schema versioning + N-1 migration helper; WAL-journal `withJournal<T>()` transaction wrapper used by all apps |
| K4 | Push abstraction | `EelPushTransport` (FCM/APNs neutral): token lifecycle, no-PII payload validator (schema-driven), tap routing |
| K5 | Geo utils | `pointInPolygon` (ray casting), `haversineMeters`, tolerance rings — shared by personnel/observer/party-agent |
| K6 | `EelProblem` | RFC 9457 parser + `ApiProblem` types + localized message keys |
| K7 | Canonical JSON | RFC 8785 serializer (pure Dart) + test vectors (existing eel-crypto fixtures) |
| K8 | Hardening | `EelMinHitTarget` docs; RASP check library (`rasp.dart` — root/emulator/frida/debugger/resign heuristics) shared entrypoint; tamper event builder matching `device_tamper_detected/v1.schema.json` |
| K9 | Example app | exercise every module (attest, sign, queue, telemetry, pin toggle, verifier) as living doc + CI canary |

## Files

```
lib/src/http_client.dart            extend (pinning adapter + problem+json + retry policy object)
lib/src/attestation.dart            add real clients + verdict client wiring
lib/src/attestation/{play_integrity,app_attest,verdict}.dart
lib/src/encrypted_isar_store.dart
lib/src/push/{push_transport,validators}.dart
lib/src/geo/{polygon,distance}.dart
lib/src/problem.dart
lib/src/canonical_json.dart
lib/src/rasp.dart
lib/src/tamper_event.dart
example/lib/* (expand)
test/... per module (pinning socket tests, PI client with fake transport, Isar migration, geo fixtures)
```

## Testing

- Golden: canonical JSON vectors; geo property tests (point-in-polygon
  invariants); backoff distribution stats; pinning fail-closed both directions
  (bad pin blocks, good pin passes, rotated set accepts old pin in window).
- `flutter analyze --fatal-infos` green; 100 % public API doc comments.

## Definition of done

Master §9 applied to a package: pinning live, real attestation clients
compile + unit-tested with fake transports, encrypted store used by at least
2 apps, example canary in CI green.
---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
