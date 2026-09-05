# Hello Flutter — Complete Reference (M0 Baseline)

**Code:** `services/hello-svc/apps/hello-flutter/`
**Purpose:** Build verification / baseline
**Functions:** Basic Flutter scaffold, `build.gradle.kts` (AGP 8.9.1 + Kotlin 2.0.21), AndroidManifest (v2 embedding), resources
**Verified:** APK 143MB (new build from this session), Java 17 + Gradle 8.11.1 + AGP 8.9.1 stack

---
# Audit Reference (5 axes: where / when / who / why / how)
Every action now logs via `AuditService` (encrypted Isar + outbox replay):
- **where**: GPS (LocationService) or station-bound coordinate
- **when**: ISO 8601 UTC timestamp (per entry UUID7)
- **who**: Actor identity + SpiceDB role (actorId, actorRole)
- **why**: Justification / reason / dispute reason (reason field)
- **how**: Method (biometric_attestation_fingerprint / iris / HSM_Ed25519 / OTP / PIN) + SHA-256 proof hash + Ed25519 signature
Offline events queued via outbox; replayed on reconnect with full 5-axis verification.
