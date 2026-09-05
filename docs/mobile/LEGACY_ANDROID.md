# Legacy Android App — Complete Reference (M29)

**Code:** `services/biometric-edge-svc/apps/legacy-android/`
**Purpose:** Backward compatibility for deprecated Android devices
**Functions:** Deep-link to kit screens (`eel://kit/bvr/enrol`, `eel://kit/bvi/identify`), MDM install guidance, `legacy_home.dart`
**State:** `attestation_tier=legacy` (deprecated), PARITY.md updated, `url_launcher` dependency added

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
