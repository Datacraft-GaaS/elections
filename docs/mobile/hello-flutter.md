# hello-flutter (CI validator) — Maintenance Plan

Purpose: `hello-svc` reference scaffold; validates kit-sdk wiring in CI.
Intentionally minimal — do NOT grow features.

## Work items

1. Consume eel-core bootstrap (proves the foundation package in the hello
   harness — required by Wave M0 exit).
2. Exercise: attest (stub, dev flavor), outbox round-trip against hello-api,
   telemetry session to telemetry-ingest dev double.
3. Keep one widget test + one integration smoke; used as first target of
   `just mobile-ci` per master §11 Wave M0 exit.

## Definition of done

Serves as canary: `just mobile-ci` runs it first; any kit-sdk/eel-core
regression breaks here first, not in production apps.Updating docs/mobile/hello-flutter.md -> HELLO-FLUTTER (expanded reference with functions, capabilities, design-system usage, testing)

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
