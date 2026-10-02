# ADR 0021: Mobile security posture

Status: accepted
Date: 2026-08-21

## Context

Voter-facing mobile apps (M13 verifier, M29 citizen verify) and accredited-user apps (M30 party agent, M31 poll clerk, M32 field support, M33 dispatch, M34 observer) run on citizen and staff devices outside the physical trust boundary. `sw/MOBILE-INTEGRATION-NOTES.md` §5 and `sw/mobile/SECURITY.md` mandate device attestation, in-app RASP, and hardened storage, extensions to ADR-0007 that do not fit inside a portal/backend ADR.

## Decision

- **Device attestation on every session**:
  - Android: **Google Play Integrity API**, verifies device integrity (Meets Basic + Device + Strong Integrity levels; policy per app tier).
  - iOS: **Apple App Attest + DeviceCheck**, verifies device authenticity.
  - Attestation token attached to every authenticated request via `X-Eel-Attest` header; backend rejects sessions without a fresh valid attestation.
- **RASP (Runtime Application Self-Protection)**:
  - Frida detection, Xposed detection, Magisk detection, root/jailbreak detection.
  - Debugger detection, emulator detection.
  - App integrity check (signature verification against pinned signing certificate).
  - Actions on tamper: session terminate + local wipe + telemetry event to Kafka `eel.mobile.tamper.v1` + user-facing security warning (non-technical).
- **Local storage**:
  - **Isar + SQLCipher** for structured data (per MOBILE-INTEGRATION-NOTES).
  - **flutter_secure_storage** (Android Keystore / iOS Keychain) for secrets.
  - No PII in shared preferences, no PII in logs.
- **Network**:
  - **dio + mTLS + certificate pinning** (pin to the Commission intermediate CA fingerprint, not leaf).
  - HSTS enforced.
  - Only TLS 1.3 (with PQ hybrid per ADR-0005).
- **Session**:
  - Short-lived JWTs (5-15min), refresh via silent auth.
  - Refresh tokens stored only in secure storage.
  - Session terminates on backgrounding for accredited apps (M30-M34); public apps (M13/M29) permit backgrounding but re-authenticate on foreground after 5 min.
- **Distribution**:
  - **Google Play Managed** (Android Enterprise) for staff apps (M30-M34).
  - **Apple Business Manager** + custom app distribution for iOS staff apps.
  - Public apps (M13/M29) via Play Store and App Store.
  - Signing: keys sealed in HSM; per-release cosign attestation (ADR-0006).
- **Update policy**:
  - Force-update floor per app; refuse to run below version floor.
  - Silent update within 24h of security release; hard block after 7d.
- **Accessibility** (extends ADR-0008): **WCAG 2.2 AA** across all mobile apps; Kenyan Sign Language interpretation surfaced in M13 (spec-mandated).
- **Language**: at least English + Swahili for voter-facing apps; additional Kenyan languages per spec.

## Consequences

Positive: attestation removes rooted/tampered devices from valid session pool; RASP raises attacker cost on device-side manipulation; secure-storage discipline defends stolen-device scenarios; distribution channels enforce provenance.

Negative: Play Integrity / App Attest depend on Google / Apple services, offline citizens using non-standard Android forks may fail attestation. Fallback: SMS/USSD verification path (M13 multi-channel) covers this population without a mobile app.

## Alternatives rejected

- No attestation: allows rooted-device credential-stuffing at scale.
- SafetyNet: deprecated by Google in favour of Play Integrity.
- Custom in-app attestation without Play/App: does not meet audit standards; reinvents wheel.
- Ad-hoc distribution (APK sideload) for staff apps: no provenance trail.

## Related

ADR-0002 (Flutter for mobile + kits), ADR-0007 (session tokens + BFF), ADR-0008 (WCAG 2.2 AA), ADR-0011 (public API surface accepts attestation header), ADR-0015 (`eel.mobile.tamper.v1` topic).
