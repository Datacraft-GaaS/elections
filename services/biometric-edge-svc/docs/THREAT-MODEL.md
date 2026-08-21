# biometric-edge-svc Threat Model

## Trust Boundaries

- All edge-api endpoints require SPIFFE/SPIRE SVID mTLS. No public internet access.
- Kit devices authenticate via per-device Ed25519 key provisioned at manufacturing.
- Attestation tokens are verified in kit-attest before any enrolment is forwarded to bvr-svc.

## Threats and Controls

| Threat | Control |
|---|---|
| Rogue kit device submits fabricated enrolments | attestation verification (kit-attest); revocation list check on each request |
| GNSS spoofing to misplace polling station | multi-constellation cross-check (EC-KIT-01); IMU sanity envelope |
| Officer coercion (distress scenario) | distress finger -> silent alert to ECC (EC-IDN-08); UI proceeds normally |
| Kit clock manipulation to bypass time fence | NTS time acquisition required before session open; skew > 5 min refuses session (EC-TIM-01) |
| Replay attack on biometric upload | Idempotency-Key checked at edge-api and bvr-svc; device_sig over capture payload |
| MitM on kit-to-edge channel | SVID mTLS; SPKI pinning in EelHttpClient |
| OTA firmware tampering | Ed25519 manifest signature verified before kit applies update |
| Insider threat: operator accesses biometric data | Data-class A encryption at rest; SVID audit log on every access |
| Legacy device with no hardware attestation | attestation_tier=legacy flagged in backend; enhanced physical security perimeter required |

## Data Classification

- Biometric captures: class A (encrypted in transit and at rest; access requires elevated SVID).
- Kit telemetry (heartbeat): class D (non-sensitive; GPS is station location, not voter location).
- Distress alerts: class C (sensitive operational; ECC-only decryption key).
