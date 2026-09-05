# Accred Scanner — Complete Reference (Training Module)

**Code:** `services/training-svc/apps/accred-scanner/`
**Functions:** QR scan badge, CRL verify, status indicator
**API path:** `/v1/security/crl` (CA-verified)

---

## Functions

### F1 — Badge Scan
`mobile_scanner` captures QR. JWT parsed (3-segment base64url). `sub` claim extracted as serial.

### F2 — CRL Verification
Fetch CRL from `/v1/security/crl` (CA-verified). Check if serial is revoked. CRL staleness (30 min) tracked via `crlStaleProvider`.

### F3 — Status Display
ACTIVE / REVOKED / SUSPENDED. Status indicator with color-coded container (green/red/amber). Access zones list.

---

## Design Reference

`EelScaffold` (scrollable), `MobileScanner` (camera), `EelResultCard` (serial/checked), `EelBanner` (CRL stale warning, from-cache notice), `EelButton` (scan another), `EelTextStyle` typography, `EelColors` tokens.

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
