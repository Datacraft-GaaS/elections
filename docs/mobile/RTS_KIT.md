# RTS Kit — Complete Reference (M28/M08)

**Code:** `services/rts-svc/apps/rts-kit/`
**Module spec:** M28 + M08 SMS
**Functions:** RFID scan/transmit, AT command lifecycle (`AT+CMGF`, `AT+CMGS`), GsmModem lifecycle, SMS PDU encoder, USB serial transport, session/bootstrap
**Screens:** scan_screen (`EelScaffold` + `EelButton` for RFID), scan_result_screen (`EelResultCard` + `EelButton`)
**Drivers:** `rts_driver.dart` (RFID/sign/transmit workflows)
**SDK calls:** Real SDK signatures documented in [`docs/mobile/M3_HARDWARE_INTEGRATION.md`](M3_HARDWARE_INTEGRATION.md)

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
