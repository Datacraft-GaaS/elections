# Diaspora Kit — Complete Reference (M30)

**Code:** `services/biometric-edge-svc/apps/diaspora-kit/`
**Module spec:** M30 (diaspora polling station)
**Functions:** Station overview, sealed pouch verification, Form 34A
**Screens:** station_screen, pouch_screen, form34a_screen (all with `eel_design`)
**Driver:** diaspora_driver.dart

---

## Functions

### F1 — Station Overview
Station status, connection state, sync status, GPS coordinates.

### F2 — Sealed Pouch Contents
Ballot box count, Form 34A count, seal verification status, BVI backend sync.

### F3 — Form 34A
Diaspora polling station Form 34A (Presiding Officer Report). Sections A–C with vote tallies, ballot reconciliation, PO/DPO signatures.

---

## Design Reference

All screens: `EelScaffold` + `EelStatusChip` (ok/info) + `EelResultCard` (info with leading icon) + `EelButton` (primary actions) + `EelTextStyle` typography + `EelColors` tokens.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
