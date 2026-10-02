# Mobile Architecture — the Commission IEMS (sw/ Integration)

**Integration note.** This document is adapted from `mobile/ARCHITECTURE.md` (source of truth pending stitching-pass consolidation). It is a companion to `../ARCHITECTURE.md` (the main sw/ system architecture). The stitching pass should add a "Mobile Subsystem" section to `../ARCHITECTURE.md` that references this file and the M29-M34 module specs.

---

## 1. Principles

1. Offline-first. Every user-visible feature works with no network or gracefully explains the requirement.
2. Signed data only. Every read of authoritative data (results, register lookup, inclusion proof) carries an HSM signature validated on device.
3. Secure-enclave for keys. No key material touches disk unencrypted.
4. Device attestation. Every session establishes device integrity via Play Integrity / App Attest.
5. Shared shell. Common cryptographic verifier, secure store, telemetry, accessibility framework — one codebase, six apps.
6. Kenya sovereignty. Telemetry endpoint, push-notification transport, and API back-ends are Kenya-resident.
7. Testable. Every screen and every API interaction covered by tests and reproducible in CI.
8. Portable. Flutter single codebase for Android + iOS; PWA fallback for the widest reach.
9. Safely updatable. Signed OTA + rollback + minimum-supported-version enforcement.
10. No lock-in. Post-handover, the Commission can build, sign, and distribute updates without vendor cooperation.

---

## 2. Platform Stack

### 2.1 Framework

- Flutter 3.x primary UI framework.
- Dart 3.x application code.
- Rust FFI (dart_ffi) for security-critical modules: signature verification, envelope encryption, HSM proxy, Merkle-proof validation.
- Native platform code (Kotlin / Swift) only where Flutter plugins are insufficient: hardware-backed keystore, biometric APIs, camera / scanner integration, MDM handshake.

### 2.2 App architecture

- Riverpod for state management (compile-time-safe, testable, Datacraft standard).
- freezed + json_serializable for immutable data classes.
- isar / drift for local encrypted persistence (SQLite under the hood with SQLCipher).
- flutter_secure_storage wrapping platform secure storage.
- dio for HTTP with mTLS + interceptor chain (retry, backoff, telemetry).
- flutter_bloc / GoRouter for navigation.
- injectable / get_it for dependency injection.

### 2.3 Testing stack

- flutter_test for unit + widget tests.
- integration_test for end-to-end.
- mocktail for mocks.
- golden_toolkit for UI regression.
- patrol for device-farm testing.

---

## 3. High-Level Diagram

```
+------------------------------------------------------------------+
|                         MOBILE APP (Flutter)                     |
|  +----------------------------------------------------------+   |
|  |  PRESENTATION                                            |   |
|  |  Screens | Widgets | Accessibility framework             |   |
|  |  Localisation (EN/SW) | Theme                            |   |
|  +---------------------------+------------------------------+   |
|                              |                                   |
|  +---------------------------v------------------------------+   |
|  |  DOMAIN                                                  |   |
|  |  Use-cases | Entities | State-management (Riverpod)      |   |
|  +---------------------------+------------------------------+   |
|                              |                                   |
|  +---------------------------v------------------------------+   |
|  |  DATA                                                    |   |
|  |  +----------+  +----------+  +------------------------+  |   |
|  |  | Local    |  | Remote   |  | Sync engine            |  |   |
|  |  | (Isar +  |  | (dio +   |  | (queue, retry, resolve)|  |   |
|  |  | SQLCipher)|  | mTLS)    |  |                        |  |   |
|  |  +----+-----+  +----+-----+  +------+-----------------+  |   |
|  +-------+-------------+---------------+-----------------+   |   |
|          |             |               |                      |
|  +-------v-------------v---------------v------------------+   |
|  |  SHARED SHELL (all six apps — M29 through M34)         |   |
|  |  - Cryptographic verifier (Rust FFI)                   |   |
|  |  - Merkle-proof validator (Rust FFI)                   |   |
|  |  - Secure-store engine (Rust FFI + platform keystore)  |   |
|  |  - Device-attestation module                           |   |
|  |  - Telemetry client (PII-scrubbed)                     |   |
|  |  - Push-notification handler                           |   |
|  +---------------------------+------------------------------+   |
|                              |                                   |
|  +---------------------------v------------------------------+   |
|  |  PLATFORM (Android / iOS)                               |   |
|  |  - Secure Enclave / Android Keystore / TEE              |   |
|  |  - Biometric APIs (BiometricPrompt / LocalAuthentication)|  |
|  |  - Camera + Scanner + QR                                |   |
|  |  - GPS + IMU                                            |   |
|  |  - MDM handshake                                        |   |
|  +----------------------------------------------------------+   |
+------------------------------------------------------------------+
                    |  mTLS + certificate pinning
                    |
                    v
      +--------------------------------------+
      |  the Commission EDGE (Kenya-resident)          |
      |  - WAF + rate limit                  |
      |  - Public endpoints (CDN-cached)     |
      |  - Kit-gateway equivalent for apps   |
      +---------------+----------------------+
                      |
                      v
      +--------------------------------------+
      |  IEMS BACK-ENDS (see ../modules/)    |
      |  - BVV / VPRS / RTS / Portal / ...  |
      |  - HSM | Merkle transparency log     |
      +--------------------------------------+
```

---

## 4. Offline-First Strategy

### 4.1 Local store

- Isar + SQLCipher for structured local data, encrypted at rest with AES-256.
- Master encryption key derived from a device-bound secret in the secure enclave.
- Local schema versioned; migrations tested per release.
- Retention policy per record: user-cleared data purged immediately; system-cached data cleared on TTL.

### 4.2 Sync engine

- Records marked `pending`, `syncing`, `synced`, `failed`, `conflict`.
- Push queue drained on connectivity restoration; exponential backoff + jitter.
- Idempotency keys on every write.
- Conflict resolution rules per record type: last-writer-wins for user preferences; server-wins for authoritative data; user-adjudicated for incident reports.
- Sync progress visible to user (spinner or notification).
- App restart, phone reboot, and OS-level killing preserve the queue.

### 4.3 Signed-content cache

- Every cryptographically signed artefact (results snapshot, Merkle root, form image) cached locally with the signature.
- Verification runs on retrieval, not on cache-hit — mistrust the cache.
- Cache eviction by TTL + LRU; signed artefacts pinnable per user request.

---

## 5. Security Architecture (summary)

Detail in `SECURITY.md` (this folder). Key architectural elements:

- Certificate pinning on every mTLS endpoint using SPKI hashes; refuse any chain not matching regardless of OS trust store.
- Device attestation via Google Play Integrity API and Apple App Attest at session start.
- Root / jailbreak detection with graceful degradation for citizen apps and refusal for staff apps.
- Runtime application self-protection (RASP): detect hooking frameworks (Frida, Xposed), detect debugger attach.
- App integrity attestation: check on launch that the running binary matches the signed manifest.
- Secure-store engine stores encryption keys in the platform secure enclave; never on disk unencrypted.
- Biometric unlock for staff apps and MFA re-authentication for high-value actions.
- No secrets in APK / IPA: all secrets fetched at first-launch via secure attested channel.
- Signed OTA updates with rollback capability.

---

## 6. Push Notifications

- Firebase Cloud Messaging (FCM) for Android and APNs for iOS as delivery transports.
- No PII in payload. Payload contains only an opaque event ID; app retrieves the full event via authenticated API on receipt.
- Signed payload envelope for high-integrity notifications (e.g., agent countersignature request).
- Self-hosted alternative evaluated for post-handover independence (matrix / apns-only + web push).

---

## 7. MDM (Mobile Device Management)

- Google Play Managed for private Android app distribution.
- Apple Business Manager for private iOS app distribution.
- App-configuration policy for private apps: per-user allowlist, force-update policy, per-org branding.
- Public apps (M29, M33) distributed via public app stores; also as signed APK / IPA from the Commission portal.
- MDM console operated by the Commission ICT with Datacraft supporting during contract.

---

## 8. Sync with Central IEMS

Each app consumes a subset of the IEMS APIs (see `../API-CATALOG.md`). Key patterns:

| Interaction | Pattern | Offline behaviour |
|---|---|---|
| BVV register lookup | REST + rate-limit + captcha (public) or mTLS (staff) | Cached last N lookups locally |
| VPRS application submit | REST + OTP + IPRS check | Draft queued locally; submitted on connect |
| Results snapshot retrieval | REST + signature verification | Cached signed snapshots; verified on retrieval |
| Merkle-inclusion proof | REST + client-side verification | Cached per polling station |
| Agent countersignature | mTLS + biometric | Queued locally; submitted on connect |
| Kit-diagnostic query (M32) | mTLS to MDM | Read-only cache; live only when online |
| Personnel attendance check-in (M33) | mTLS + biometric + GPS | Queued locally; submitted on connect |
| Observer incident report (M34) | mTLS + attachments | Queued locally with encrypted attachments; submitted on connect |

---

## 9. Distribution and Update Model

### 9.1 Public apps (M29, M33)

- Google Play + Apple App Store.
- Signed APK direct-download from `verify.eel.example` for citizens without Play access.
- OTA updates staged: 5% canary → 25% → 100%; automated rollback on crash-rate regression.
- Minimum-supported-version enforcement (< min = refuses to run, prompts update).

### 9.2 Private apps (M30, M31, M32, M34)

- Managed distribution via Commission MDM.
- Force-update within 48 hours of publication.
- Per-user allowlist tied to accreditation database.

### 9.3 Software freeze

- 30 days before every gazetted election.
- Only dual-approved critical security patches permitted during freeze.
- Freeze policy encoded in CI/CD; changes require CCB approval.

---

## 10. Telemetry and Observability

- Structured client events (JSON): app.launched, screen.viewed, action.triggered, error.thrown, sync.completed, sync.failed, network.state.
- Correlation IDs propagate across client → API → back-end → audit log.
- No PII in events: user IDs are opaque tokens; free text scrubbed.
- Kenya-resident telemetry endpoint (self-hosted).
- Crash reporting: self-hosted Sentry-compatible collector.
- User feedback channel in-app: submits attached with device + app version + optional email.

---

## 11. Environment Strategy

| Environment | Purpose | Data |
|---|---|---|
| Production | Live public + private apps | Real production data (register lookup, results) |
| Staging | Pre-production validation | Sanitised subset (synthetic data) |
| Sandbox | the Commission training + drills | Fully synthetic |
| Development | Datacraft dev | Fully synthetic |
| CI | Test execution | Synthetic + mocked |

Apps built with build-flavour system: same codebase, different endpoints and signing keys per flavour.

---

## 12. Handover Architecture

- Full Flutter source code + Rust FFI modules under GCC 15.4 IPR transfer.
- Build scripts + CI/CD pipeline definitions.
- Signed OTA update pipeline handed to the Commission (or Datacraft holds signing keys in escrow per SCC).
- App-store publisher accounts (Google Play Developer + Apple Developer) either transferred to the Commission or Commission-owned from day one.
- Documentation: this file, `PRD.md`, `SECURITY.md`, per-app module specs M29-M34, runbooks, ADRs.
- MDM console configuration transferred with training.

Post-handover, the Commission ICT can continue to build, sign, and publish updates independently.
