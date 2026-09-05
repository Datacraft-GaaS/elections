# Personnel App — Complete Reference (M29)

**Code:** `services/field-mobile-svc/apps/personnel/`
**Module spec:** M29 (check-in/check-out, status ping, assignment, incident)
**Screens:** status_ping, assignment, check_in, check_out, incident
**Status:** All rewritten with `eel_design`

---

## Functions

### F1 — Status Ping (FR-001)
Real-time status report with station code, connectivity state, battery level. `EelResultCard` + `EelButton` to send.

### F2 — Assignment (FR-002)
Current station assignment with shift times. `EelStatusChip` (Active), `EelStatusChip` (shift time).

### F3 — Check-In (FR-003)
Personnel ID + station code. `EelTextField` (both fields), `EelButton` (check in).

### F4 — Check-Out (FR-004)
Confirmation with `EelBanner` reminder to submit all forms. `EelButton` + cancel option.

### F5 — Incident (FR-005)
Report with description, category, evidence. `EelTextField`, `EelButton`.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
