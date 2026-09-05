# BVI Kit — Complete Reference (M3 Kit Hardware)

**Code:** `services/biometric-edge-svc/apps/bvi-kit/`
**Module spec:** M3 (kit hardware)
**Driver:** `AntIrisDriver` (USBControl.jar + lib_Antiris.so + libopenjpeg.so)
**SDK:** `USBControl.antIrisIrexCapture` + `IrisIso.generate`
**Functions:** Iris capture for biometric voter identification (BVI)
**Status:** Hardware-gated — returns `IrisResult.invalid()` if plugin not registered

---

## Functions

- `USBControl.antIrisIrexCapture()` — capture iris image via USB camera
- `IrisIso.generate(image)` — generate ISO 19794-6 template
- `IrisIso.match(templateA, templateB)` — 1:1 match
- `IrisIso.search(template, gallery)` — 1:N search

## State Machine

`Uninitialized → Initializing → Ready → Capturing → Captured → Generating → Generated → Matching → Matched | Unmatched | Error`

## APK

Pre-existing APK `bvi-kit` 166MB (Aug 27 build) — validates M3 stack.

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

## Dual Biometric Verification (NEW — 2-Factor)

As of M3 update, BVI requires **BOTH iris (AntIris) AND fingerprint (FAP60)**. The `DualBiometricCapture.verify()` method calls `captureFingerprint()` followed by `captureIris()`, verifies 1:1 match against enrolled templates via Rust FFI (`EelVerifierFfi`), logs full audit entry (GPS, timestamp, actor role, justification, method, proof hash, Ed25519 signature), and only returns `isValid=true` when BOTH biometrics match.
