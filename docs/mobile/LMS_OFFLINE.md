# LMS Offline — Complete Reference (Training Module)

**Code:** `services/training-svc/apps/lms-offline/`
**Module spec:** Training (offline learning management)
**Functions:** Module download (5-module cap), assessment taking, sync to BFF
**API path:** `/v1/training/lms/assessments/{id}/submissions`

---

## Functions

### F1 — Module Browser
List of training modules with type (core/refresher/briefing), language. 5-module cache limit enforced. Sync to BFF.

### F2 — Assessment
Question-by-question UI with chip-based answer selection. Submit to BFF `/v1/training/lms/assessments/{id}/submissions` with UUID7 idempotency.

### F3 — Sync
Offline queue with retry. On reconnect, push queued submissions to BFF.

---

## Design Reference

`EelScaffold` (scrollable), `EelResultCard` (module list), `EelChips` (answer selection), `EelStatusChip` (status), `EelLoadingState` (loading), `EelBanner` (offline/sync messages), `EelTextStyle` typography.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
