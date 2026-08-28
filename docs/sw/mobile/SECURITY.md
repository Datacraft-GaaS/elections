# Mobile Security Requirements — IEBC IEMS (sw/ Integration)

**Integration note.** This document is adapted from `mobile/SECURITY.md` (source of truth pending stitching-pass consolidation). It is a companion to `../SECURITY.md` (the main sw/ security specification). The stitching pass should add a "Mobile Security" section to `../SECURITY.md` that references this file, the SR-M-* requirement namespace, and the per-app profiles in §3 below.

Mobile devices are hostile territory: they leave IEBC's physical control, they may be rooted, they may be running under an attacker's supervision. This document consolidates the mobile-specific security requirements that harden the six apps against that reality.

---

## 1. Attack Surface

- Device compromise: root / jailbreak, malware, kernel exploit.
- App tamper: repackaged APK, hooking frameworks (Frida, Xposed), debugger attach.
- Local storage exfiltration: encrypted local store extracted by root.
- Network MITM: rogue CA, IMSI catcher, corporate proxy interception.
- Push-notification abuse: spoofed FCM sender, replay of stale notifications.
- Biometric bypass: presentation attack, coerced authentication.
- Screen scraping / recording: unauthorised capture of results or PII on-screen.
- OTA update spoofing: fake update installing malicious binary.
- Store-listing impersonation: look-alike app on Play / App Store.
- Social engineering: phishing directing users to install malicious apps.
- Insider threat: Datacraft or IEBC staff with signing-key access.

---

## 2. Requirements

### 2.1 Device Integrity

- **SR-M-DEV-001** Every session establishes device integrity via Google Play Integrity API (Android) or Apple App Attest (iOS) at launch and periodically thereafter (every 4 hours or on state change).
- **SR-M-DEV-002** Attestation failure on staff apps (M30, M31, M32, M34) halts app; user must re-attest on a compliant device.
- **SR-M-DEV-003** Attestation failure on citizen apps (M29, M33) shows a prominent warning and reduces functionality (read-only, no critical actions).
- **SR-M-DEV-004** Root / jailbreak detection via multiple heuristics (su binary presence, Magisk detection, Cydia detection, mount table inspection, SELinux permissive check).
- **SR-M-DEV-005** Emulator detection: refuse on staff apps; warn on citizen apps.
- **SR-M-DEV-006** Anti-debug: refuse execution if debugger attached to production builds.

### 2.2 App Integrity

- **SR-M-APP-001** App-signature verification on every launch: refuse if signing key doesn't match expected fingerprint.
- **SR-M-APP-002** Hooking-framework detection: check for Frida (loaded modules, port scan), Xposed (framework classes), Substrate; refuse on detection for staff apps.
- **SR-M-APP-003** Code-integrity attestation: hashes of loaded Dart / native modules checked against signed manifest.
- **SR-M-APP-004** Runtime application self-protection (RASP): detect method-swizzling, class-swizzling, native library injection.
- **SR-M-APP-005** Reproducible builds so IEBC can independently verify any APK / IPA against source.

### 2.3 Cryptography

- **SR-M-CRYP-001** TLS 1.3 mandatory; refuse fallback to TLS 1.2 or below.
- **SR-M-CRYP-002** Certificate pinning via SPKI hash for every mTLS endpoint; refuse chain not matching regardless of OS trust store.
- **SR-M-CRYP-003** mTLS for all staff apps; server-authenticated TLS + pinning for citizen apps.
- **SR-M-CRYP-004** AES-256-GCM for local encryption at rest.
- **SR-M-CRYP-005** Encryption keys generated in secure enclave (Android Keystore StrongBox / iOS Secure Enclave); never exportable.
- **SR-M-CRYP-006** Perfect forward secrecy on every session.
- **SR-M-CRYP-007** No custom cryptography; all primitives from audited libraries (BoringSSL, libsodium, ring via Rust FFI, HSM proxy).
- **SR-M-CRYP-008** Signature verification for HSM-signed data (results snapshots, Merkle roots) via Rust FFI validator that ships as native binary.
- **SR-M-CRYP-009** Anti-rollback: session tokens carry monotonic counter + nonce; server rejects regression.
- **SR-M-CRYP-010** Certificate transparency check on server certificates (Google CT logs).

### 2.4 Authentication

- **SR-M-AUTH-001** Citizen apps (M29, M33): OTP-based sign-in tied to ID number + registered mobile.
- **SR-M-AUTH-002** Staff apps (M30, M31, M32, M34): MFA sign-in with FIDO2 platform authenticator or hardware key + device attestation.
- **SR-M-AUTH-003** Biometric unlock (BiometricPrompt on Android, LocalAuthentication on iOS) for staff apps and MFA re-authentication for high-value actions.
- **SR-M-AUTH-004** Biometric fail-over to strong PIN (min 6 digits, no repeat, no sequence).
- **SR-M-AUTH-005** Auto-lock after configurable idle timeout (default 5 min citizen, 2 min staff); biometric or PIN re-auth to resume.
- **SR-M-AUTH-006** Session tokens short-lived (<= 15 min citizen, <= 5 min staff); refresh requires re-attestation.
- **SR-M-AUTH-007** Account lockout after N failed attempts with exponential backoff.
- **SR-M-AUTH-008** Sign-out clears secure store; explicit consent for persistent sign-in on citizen apps.

### 2.5 Data at Rest

- **SR-M-DATA-001** All local data encrypted AES-256-GCM.
- **SR-M-DATA-002** Encryption keys held in secure enclave, wrapped by device-bound secret.
- **SR-M-DATA-003** Sensitive fields (ID number, biometric prompts, personal names) redacted from screenshots via FLAG_SECURE / privacy overlay.
- **SR-M-DATA-004** Cache eviction on sign-out; clear-on-uninstall.
- **SR-M-DATA-005** No PII in logs, crash reports, or telemetry.
- **SR-M-DATA-006** Screen recording detection: warn user (staff apps) that recording is active.
- **SR-M-DATA-007** iOS App Groups / Android ContentProvider disallowed for sensitive data.
- **SR-M-DATA-008** Keychain / Keystore items marked non-backupable + biometric-gated.

### 2.6 Data in Transit

- **SR-M-NET-001** TLS 1.3 + certificate pinning (see SR-M-CRYP-002).
- **SR-M-NET-002** mTLS for staff apps (M30, M31, M32, M34).
- **SR-M-NET-003** No HTTP, HTTPS-only enforced via App Transport Security (iOS) / cleartextTrafficPermitted=false (Android).
- **SR-M-NET-004** No traffic to third-party analytics endpoints. All telemetry to IEBC-controlled Kenya-resident endpoint.
- **SR-M-NET-005** VPN awareness: staff apps warn on active VPN because it can defeat pinning intent; user must confirm.
- **SR-M-NET-006** Proxy detection: refuse to run on staff apps if system proxy is active.

### 2.7 Storage of Secrets

- **SR-M-SEC-001** No secrets shipped in APK / IPA.
- **SR-M-SEC-002** All secrets fetched at first launch via secure attested channel and stored in secure enclave.
- **SR-M-SEC-003** Rotation on schedule; short-lived by default.
- **SR-M-SEC-004** Compromise recovery: server can invalidate device-bound secrets; app must re-provision.

### 2.8 Push Notifications

- **SR-M-PUSH-001** No PII in push payloads.
- **SR-M-PUSH-002** Signed payload envelope for high-integrity notifications; app verifies before acting.
- **SR-M-PUSH-003** Replay protection via nonce + expiry.
- **SR-M-PUSH-004** Confidentiality: sensitive content fetched over authenticated channel on receipt, not delivered in the push.

### 2.9 OTA Updates

- **SR-M-OTA-001** Signed OTA packages verified against IEBC-controlled public key baked into the app.
- **SR-M-OTA-002** Rollback capability on crash-rate regression or manual trigger.
- **SR-M-OTA-003** Minimum-supported-version enforcement: apps below minimum refuse to run.
- **SR-M-OTA-004** Software freeze 30 days before every gazetted election; only dual-approved security patches permitted.
- **SR-M-OTA-005** Full audit trail of every OTA event with actor, version, hash, cohort.

### 2.10 Biometric APIs

- **SR-M-BIO-001** Platform biometric APIs only (BiometricPrompt on Android, LocalAuthentication on iOS). No custom biometric enrolment.
- **SR-M-BIO-002** Biometric templates never leave device.
- **SR-M-BIO-003** Presentation-attack detection where platform provides it (e.g., iOS FaceID liveness).
- **SR-M-BIO-004** Fallback to strong PIN.
- **SR-M-BIO-005** Coercion resistance: for staff apps, offer duress-code fallback that appears to authenticate but silently alerts back-end.

### 2.11 Location

- **SR-M-GEO-001** Location capture only when required and with explicit consent.
- **SR-M-GEO-002** Foreground-only permission for public apps; background permission only for staff apps (M32, M33) with justification.
- **SR-M-GEO-003** IMU cross-check for GPS spoofing detection where relevant (M30, M33 check-in).
- **SR-M-GEO-004** Location data included in signed event payload; server-side integrity check.

### 2.12 Privacy

- **SR-M-PRIV-001** Kenya DPA 2019 compliant with ODPC registration.
- **SR-M-PRIV-002** Consent capture on first launch: what data collected, why, retention, right to withdraw.
- **SR-M-PRIV-003** Data minimisation: only fields the app genuinely needs.
- **SR-M-PRIV-004** No third-party trackers or advertising SDKs.
- **SR-M-PRIV-005** Right to erasure implemented as in-app action; server-side workflow completes within 30 days.
- **SR-M-PRIV-006** Data-safety declaration on Google Play + Privacy Nutrition Label on App Store accurately reflects behaviour.
- **SR-M-PRIV-007** Privacy policy hosted on iebc.or.ke; linked in-app; content matches actual behaviour.

### 2.13 Store Listing

- **SR-M-STORE-001** Only IEBC-owned Google Play + Apple Developer accounts publish official apps.
- **SR-M-STORE-002** Look-alike-app monitoring service subscribed; take-down process pre-agreed.
- **SR-M-STORE-003** Public-key fingerprint of legitimate app published on iebc.or.ke + multiple canonical domains so citizens can verify.
- **SR-M-STORE-004** In-app "verify authenticity" screen shows signing-key fingerprint + version + build info.

### 2.14 Screen Content Protection

- **SR-M-SCR-001** FLAG_SECURE on Android + isScreenCaptureEnabled=false behaviour on sensitive screens (biometric, MFA, register lookup).
- **SR-M-SCR-002** iOS: screenshot detection triggers warning; screen-record detection triggers overlay.
- **SR-M-SCR-003** App-switcher preview hidden for staff apps (M30, M31, M32, M34).
- **SR-M-SCR-004** Screenshots of results are permitted on citizen apps (public data); PII-bearing screens excluded.

### 2.15 Insider / Supply-Chain

- **SR-M-SUP-001** Signed commits (Sigstore or GPG) on every merge.
- **SR-M-SUP-002** Two-person review on protected branches.
- **SR-M-SUP-003** SBOM (SPDX) published per release; signed.
- **SR-M-SUP-004** Reproducible builds; IEBC + independent auditor can verify any published APK / IPA against source.
- **SR-M-SUP-005** Play Console + App Store Connect access limited by role + MFA + audit.
- **SR-M-SUP-006** App-signing key managed by IEBC or held in escrow with IEBC-controlled unlock ceremony.

### 2.16 Incident Response

- **SR-M-IR-001** Kill-switch: server-side capability to disable a version or a specific user session.
- **SR-M-IR-002** Force-update pathway on discovered vulnerability.
- **SR-M-IR-003** Breach-notification workflow to ODPC within 72 h per DPA.
- **SR-M-IR-004** Post-incident public disclosure (redacted) within 30 days.

---

## 3. Per-App Security Profile

| App | Module | Device attestation | mTLS | Biometric unlock | Duress code | Screen-capture protection |
|---|---|:-:|:-:|:-:|:-:|:-:|
| IEBC Verify | M29 | Required (soft-fail warn) | Public + pinning | Optional | — | On PII screens |
| Agent | M30 | Required (hard-fail) | Yes | Required | Yes | On all screens |
| Poll Clerk Companion | M31 | Required (hard-fail) | Yes | Required | Yes | On voter-lookup screens |
| Field Support | M32 | Required (hard-fail) | Yes | Required | — | On kit-diagnostic screens |
| IEBC Dispatch | M33 | Required (soft-fail warn) | Yes | Required | — | On assignment + PII screens |
| Observer | M34 | Required (hard-fail) | Yes | Required | — | On incident-report screens |

---

## 4. Cross-Reference

| Threat model section | Mobile requirements |
|---|---|
| §3 Software supply chain | SR-M-SUP-* |
| §4 Physical device (KIEMS) — analogous for phones | SR-M-DEV-*, SR-M-APP-* |
| §5 Cryptographic keys | SR-M-CRYP-*, SR-M-SEC-* |
| §6 Network | SR-M-NET-* |
| §8 Application | SR-M-APP-*, SR-M-AUTH-*, SR-M-DATA-*, SR-M-SCR-* |
| §9 Data at rest | SR-M-DATA-* |
| §10 Insider | SR-M-SUP-*, SR-M-IR-* |
| §11 Social engineering | SR-M-STORE-*, SR-M-PUSH-* |

---

## 5. Testing

Every SR-M-* requirement is verified in mobile security testing (MobSF + independent penetration test) and mapped in `ACCEPTANCE-CRITERIA.md` (this folder).
