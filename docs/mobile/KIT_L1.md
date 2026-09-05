# Kit L1 — Complete Reference (M4 AI Platform)

**Code:** `services/ai-platform-svc/apps/kit-l1/`
**Functions:** `KitL1Pipeline` wrapping `AnomalyDetector`; integration with biometric service
**Status:** Integrates with `AnomalyDetector`; uses `eel_design` (EelScaffold + EelButton)

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
