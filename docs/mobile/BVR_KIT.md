# BVR Kit — Complete Reference (M3 Kit Hardware)

**Code:** `services/biometric-edge-svc/apps/bvr-kit/`
**Module spec:** M3 (kit hardware)
**Driver:** `Fap60Driver` (MethodChannel `eel.bvr/fingerprint`)
**SDK:** `FUFingerSDK.LIVESCAN_Init` + `FUFingerSDK.captureFinger` (Android Java) + `FPAPI` (libFingerAPI.so)
**Functions:** Fingerprint capture for biometric voter registration (BVR)
**Status:** Hardware-gated — returns `FingerprintResult.invalid()` if plugin not registered (never crashes)

---

## Functions

- `LIVESCAN_Init(context)` — initialize SDK
- `captureFinger()` — capture fingerprint image
- `FPAPI.match(templateA, templateB)` — 1:1 match
- `FPAPI.search(template, gallery)` — 1:N search

## State Machine

`Uninitialized → Initializing → Ready → Capturing → Captured → Matching → Matched | Unmatched | Error`

When plugin not present (e.g. emulator), `M3FeatureGate` returns `false` and `Fap60Driver` returns `FingerprintResult.invalid()`.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.

---

## Dual Biometric Enrollment (NEW — 2-Factor)

As of M3 update, BVR requires **BOTH fingerprint (FAP60) AND iris (AntIris)** for enrollment and verification. The unified `DualBiometricCapture` flow (see `dual_biometric_capture.dart`) captures fingerprint first (`LIVESCAN_Init` + `captureFinger`), then iris (`USBControl.antIrisIrexCapture` + `IrisIso.generate`), logs both to `AuditService` with 5-axis audit entry, and produces a combined `proofHash` (SHA-256 of canonical fingerprint + iris payload) signed with `EelSigningKey` (device-bound Ed25519, TEE-backed).
