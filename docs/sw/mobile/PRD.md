# Mobile Applications PRD — subordinate to sw/PRD.md

**Integration note.** This document is adapted from `mobile/PRD.md` (source of truth pending stitching-pass consolidation). The sw/-module IDs for the six apps are M29 through M34. When `sw/PRD.md` v2 is complete, Section 6 of that document should include a reference to this file and to the six module IDs below.

| Source app ID | sw/ Module ID | App name |
|---|---|---|
| MA01 | M29 | the Commission Verify (Citizen) |
| MA02 | M30 | Agent (Party Agent) |
| MA03 | M31 | Poll Clerk Companion |
| MA04 | M32 | Field Support |
| MA05 | M33 | the Commission Dispatch (Personnel) |
| MA06 | M34 | Observer |

---

**Version:** 1.0
**Date:** 2026-08-18 (source); integrated 2026-08-21
**Owner:** Datacraft Limited (software)
**Consumer:** the electoral authority of Kenya
**Contract reference:** the tender specification

**Scope note.** This PRD covers standalone smartphone and tablet applications, NOT the KIEMS-device clients (BVR/BVI/RTS on the ruggedised device), which are in `../modules/`. This PRD covers six apps: the Commission Verify (citizen), Agent (party agent), Poll Clerk Companion (Deputy PO / kiosk), Field Support (Datacraft engineer), the Commission Dispatch (polling-day personnel), Observer (accredited).

---

## 1. Executive Summary

The mobile-app suite extends the IEMS to the phone in every voter's pocket, to every party agent in every polling station, to every Deputy Presiding Officer as a companion instrument, to every Datacraft field engineer supporting the fleet, to every polling clerk being dispatched, and to every accredited observer.

**Design commitment.** Every mobile app in this suite must satisfy the same non-negotiable design properties: (1) offline-first for anything that could plausibly happen without connectivity; (2) cryptographically verifiable — every read of authoritative data comes with a signature the user can validate; (3) privacy-preserving — no PII ever leaves the device unencrypted or without justification; (4) accessible — WCAG 2.2 AA on visual surfaces plus audio-guided workflows for blind or low-literacy users; (5) defensively hardened — device attestation, root/jailbreak detection, certificate pinning, secure-enclave key custody; (6) localised — English + Kiswahili at launch, extensible; (7) updatable safely — signed OTA updates with rollback; (8) auditable — every state change on device is logged and syncs to the central Merkle audit log.

**Priority app.** the Commission Verify (M29) is the flagship: a citizen who wants to check their register entry, look up their polling station on election day, and verify the result of their polling station with a cryptographic inclusion proof — all in one app in Kiswahili and English on the cheapest Android phone on the market. Success is measured by >= 10% of voters using the app to verify results within 24 hours of publication.

---

## 2. Vision and Success Criteria

### 2.1 Vision

Any Kenyan citizen with a smartphone can verify their own polling station's result with a cryptographic proof, and can trust that the app in their pocket cannot be silently tampered with.

### 2.2 Success criteria

| App | Module | Target |
|---|---|---|
| the Commission Verify | M29 | >= 3M downloads within 90 days of launch; >= 10% of voters use it to verify results within 24h of publication |
| Agent | M30 | 100% of accredited party agents onboarded on the app before election day |
| Poll Clerk Companion | M31 | Backup Deputy-PO flow tested at >= 100 polling stations during dress rehearsal |
| Field Support | M32 | 100% of field engineers using it for ticket lifecycle by election day |
| the Commission Dispatch | M33 | >= 95% of ~250K polling-day personnel dispatched + attended via app on election day |
| Observer | M34 | 100% of accredited observers onboarded; >= 90% report field observations via app |

### 2.3 Non-goals

- Ballot casting (Kenya does not conduct electronic voting; this app suite does not enable it).
- Result editing (mobile apps read only; all writes to authoritative state occur via KIEMS device or the Commission back-office).
- Voter registration completion (mobile apps can start VPRS pre-registration but biometric capture must occur on KIEMS).

---

## 3. Personas and Use Cases

| App | Module | Persona | Volume | Key use cases |
|---|---|---|---|---|
| the Commission Verify | M29 | Registered voter | 30M | Look up register entry; find polling station; verify results; report suspected issue; get civic info |
| the Commission Verify | M29 | Unregistered citizen | ~5M | Start VPRS pre-registration; find nearest registration centre |
| Agent | M30 | Party agent | ~2K per party peak | Verify Form 34A matches physical form via QR; countersign; retrieve inclusion proof; report incidents |
| Poll Clerk Companion | M31 | Deputy PO / Clerk | ~92K | Assist during polling day when KIEMS is in single-voter workflow; log incidents; reference polling manual; look up voters if kit unavailable |
| Field Support | M32 | Datacraft engineer | ~200 | Diagnose kit issues; open/track tickets; run remote diagnostics; escalate |
| the Commission Dispatch | M33 | Polling clerk | ~250K on peak day | Receive assignment; check-in via GPS; report on-site; check-out |
| Observer | M34 | Accredited observer | ~500 | Report field observations; retrieve station-level snapshots; verify inclusion proofs; submit incident forms |

---

## 4. Platform Targets

### 4.1 Supported platforms

| App | Module | Android | iOS | Web (PWA) |
|---|---|:-:|:-:|:-:|
| the Commission Verify | M29 | >= 8.0 (API 26) | >= 14.0 | PWA fallback |
| Agent | M30 | >= 10.0 (API 29) | >= 15.0 | — |
| Poll Clerk Companion | M31 | >= 10.0 (API 29) | >= 15.0 | — |
| Field Support | M32 | >= 10.0 (API 29) | >= 15.0 | — |
| the Commission Dispatch | M33 | >= 8.0 (API 26) | >= 14.0 | PWA fallback |
| Observer | M34 | >= 10.0 (API 29) | >= 15.0 | — |

Rationale: the Commission Verify + Dispatch target the widest device base (Android 8+ covers > 95% of the Kenyan market, including budget devices under KES 10,000); apps with stricter security requirements (Agent, Clerk Companion, Field Support, Observer) require Android 10+ / iOS 15+ for hardware-backed attestation.

### 4.2 Technology choice

- Flutter 3.x as the primary framework for all apps. Rationale: single codebase, Commission-teachable, mature offline-first primitives, Datacraft house standard (CrowdAfrica, MeGuard already in Flutter), strong plugin ecosystem for biometrics and secure storage, native performance for camera + scanning.
- Rust FFI (dart_ffi) for cryptography-critical modules (signature verification, HSM proxy, secure store envelope encryption).
- PWA (Progressive Web App) fallback for the Commission Verify + the Commission Dispatch — same feature set with a subset of the offline-first envelope, served from the Public Portal domain, for citizens who cannot install a native app or whose device is too old.

---

## 5. Cross-Cutting Non-Functional Requirements

Every app in this suite must exhibit these properties.

### 5.1 Offline-first

- Every user-visible feature works with no network, or gracefully explains the network requirement.
- Local encrypted store with per-record signature.
- Sync engine with retry, backoff, jitter, and conflict-resolution rules per record type.
- Queued events survive app restart, phone reboot, and OS-level killing.

### 5.2 Security

- Certificate pinning on every mTLS-protected endpoint; hard-coded fingerprints refuse any TLS chain not matching regardless of OS trust store.
- Root / jailbreak detection with graceful degradation (informed refusal for high-risk apps; warning for citizen apps).
- Secure Enclave / Android Keystore / iOS Secure Enclave for all key material.
- App integrity attestation (Play Integrity API + iOS App Attest).
- Anti-tamper: reject execution if signature check fails or code integrity is compromised.
- Runtime application self-protection (RASP): detect hooking frameworks (Frida, Xposed), detect debugger attach.
- No PII in device logs, crash reports, or telemetry.
- Auto-lock after configurable idle timeout; biometric re-auth to resume.
- Encrypted local storage with AES-256-GCM keys held in secure enclave.
- Clear data on app uninstall; explicit user consent for optional data retention.
- Reference: see `SECURITY.md` (this folder) and `../SECURITY.md` for full requirement catalogue.

### 5.3 Privacy

- Consent capture per Kenya DPA 2019 on first launch: what data is collected, why, how long retained, right to withdraw.
- Data minimisation: only fields the app genuinely needs.
- Purpose limitation enforced in code: register lookup APIs return only fields required for the specific view.
- No analytics beacons to third-party trackers. Only Commission-controlled telemetry endpoint.
- No advertising SDKs of any kind.
- Face / fingerprint templates used for on-device biometric unlock never leave the device.

### 5.4 Accessibility

- WCAG 2.2 AA compliant (upgraded from 2.1 per Teaming Annex A).
- Screen-reader friendly (TalkBack on Android, VoiceOver on iOS): every actionable widget has semantics labels; every image has meaningful alt text.
- Font size respects OS-level scale up to 200%.
- High-contrast mode.
- Voice-guided workflow for the Commission Verify (M29) and the Commission Dispatch (M33) (audio narration of screens in Kiswahili + English).
- Colour choices tested for common colour-blindness types.
- Touch targets >= 48dp.

### 5.5 Localisation

- English + Kiswahili at launch.
- Language matrix extensible via message catalogue; support Kikuyu, Dholuo, Kalenjin, Kamba, Luhya in phased release.
- International observer language pack (French, Portuguese) for M34.
- All user-facing text externalised; no hard-coded strings.
- Right-to-left support left latent for future Arabic option.
- Date-time in EAT; numbers per Kenyan locale.

### 5.6 Performance

- Cold start <= 3 seconds on Android 8.0 mid-tier device.
- Warm start <= 500 ms.
- API interaction with visible feedback within 200 ms; long operations show progress.
- App download size <= 50 MB for citizen-facing apps (M29 + M33); private apps may be larger.
- Battery cost <= 3% per hour of foreground use on a mid-tier device.

### 5.7 Resilience

- Circuit breakers on every external dependency.
- Retry with exponential backoff + jitter.
- Idempotency keys on every write.
- Graceful degradation ladder: full feature → reduced feature → read-only → offline-cache → holding page.
- Never crash on network error; always show meaningful state.

### 5.8 Observability

- Structured client-side events with correlation IDs; sent to Commission-controlled telemetry endpoint only (never a third-party analytics service).
- Crash reporting via Sentry-compatible endpoint (self-hosted).
- User feedback channel in every app.
- Only anonymised, PII-scrubbed events transmitted.

### 5.9 Distribution and Updates

- Public apps (M29, M33) — Google Play, Apple App Store, plus signed APK direct download from portal.
- Private apps (M30, M31, M32, M34) — private distribution via MDM (Google Play Managed / Apple Business Manager) plus installation profiles from Commission-controlled MDM console.
- Signed OTA updates with rollback capability.
- Software freeze 30 days before every gazetted election except critical dual-approved security patches.
- Minimum-supported-version enforcement — apps below minimum version refuse to operate; user is prompted to update.
- Firebase Cloud Messaging for push notifications, self-hosted alternative preferred where feasible.

### 5.10 Data Sovereignty

- All authoritative writes originate on-device but authoritative data resides on Commission-provided Kenya infrastructure.
- Push-notification service must not carry PII in payload.
- Telemetry endpoint is Kenya-resident.

### 5.11 Handover / IPR

- Custom Software (all Datacraft-built app code) vests in the Commission per GCC 15.4 at Operational Acceptance.
- Standard Software (Flutter framework, open-source plugins) licensed per each vendor's terms.
- App-signing keys held by the Commission (or by Datacraft in escrow with the Commission access) so the Commission can continue publishing updates post-handover.
- Source code, build scripts, and full documentation transferred at Operational Acceptance.

---

## 6. App Summaries

### 6.1 M29 — the Commission Verify (Citizen)

Flagship citizen app. Voter register lookup (BVV integration), polling-station finder with GPS map, VPRS pre-registration entry, results verification with Merkle-inclusion proof per polling station, candidate directory, civic-info reference, feedback channel. Full detail: `../modules/M29-mobile-citizen-verify.md`.

### 6.2 M30 — Agent

Party agent app. Onboarding tied to accredited-agent list per party. In-station tools: QR-scan a Form 34A to confirm it matches the transmitted digital record, view the digital signature chain, countersign, report incidents, retrieve constituency and county rollups. Full detail: `../modules/M30-mobile-party-agent.md`.

### 6.3 M31 — Poll Clerk Companion

Deputy PO / clerk companion for when the KIEMS is single-user busy. Reference materials, polling manual, incident logger, voter lookup fallback, hand-off queue management, dashboards mirrored from the kit. Full detail: `../modules/M31-mobile-poll-clerk.md`.

### 6.4 M32 — Field Support

Datacraft field-support engineer app. Ticket queue, remote-diagnostics tooling, kit-health inspection, KEBS/warranty claim initiation, escalation-matrix access, on-call rotation. Full detail: `../modules/M32-mobile-field-support.md`.

### 6.5 M33 — the Commission Dispatch

Polling-day personnel dispatch + attendance. Assignment delivery, GPS-verified check-in, on-site status pings, incident reporting, check-out, payroll trigger. Successor to Datacraft's MeGuard pattern. Full detail: `../modules/M33-mobile-personnel-dispatch.md`.

### 6.6 M34 — Observer

Accredited independent + international observer app. Read-only mirror of Public Portal + agent-level countersignature retrieval + observer-specific incident reporting per accreditation. Full detail: `../modules/M34-mobile-observer.md`.

---

## 7. Cross-App Requirements

### 7.1 Unified sign-in

- the Commission Verify (M29): OTP-based sign-in tied to voter's ID number + registered mobile.
- Agent (M30) / Poll Clerk Companion (M31) / Field Support (M32) / Observer (M34): MFA sign-in (FIDO2 or hardware-backed platform key) with device attestation.
- the Commission Dispatch (M33): biometric-first (fingerprint / face) with fallback OTP.

### 7.2 Shared shell components

- Common cryptographic verifier (Rust FFI): validates the Commission HSM signatures on any surfaced data.
- Common Merkle-inclusion-proof retriever + validator.
- Common secure-store engine (AES-256-GCM with keys in secure enclave).
- Common device-attestation module.
- Common telemetry client with PII scrubbing.
- Common accessibility framework (semantic labels, audio narration).

### 7.3 Shared visual identity

- the Commission brand palette, logo usage rules, typography (Noto Sans for latin + swahili glyphs).
- Consistent iconography.
- Standard layout patterns across apps for familiarity.

---

## 8. Testing and Acceptance

### 8.1 Testing regime

| Layer | Tool | Coverage target |
|---|---|---|
| Unit | Flutter test / Dart | >= 80% |
| Widget | flutter_test | >= 60% of visible widgets |
| Integration | Flutter integration_test | End-to-end happy path + edge cases |
| Contract | Pact | Every consumer/provider API |
| Accessibility | Google Accessibility Scanner + manual TalkBack/VoiceOver walkthrough | 100% key flows |
| Performance | Flutter DevTools + custom benchmarks | Meets §5.6 targets |
| Security | MobSF + manual pentest by independent firm | No high-severity findings |
| Localisation | Native-speaker review Kiswahili | Reviewed and signed off |
| Chaos | Simulated network loss / battery drain / device rotation | Continues per spec |
| Compatibility | Firebase Test Lab across 30+ real Android devices | Passes on all |
| Field trial | Multi-county real deployment | Documented sign-off |

Full detail in `ACCEPTANCE-CRITERIA.md` (this folder).

### 8.2 Acceptance gates

- App-store submission gate: passes Google Play + Apple App Store review with all app-store-required content (privacy policy URL, data-safety declaration, screenshots).
- MDM distribution gate: passes the Commission MDM policy review.
- Field-trial gate: multi-county real deployment with sign-off.
- Dress-rehearsal gate: full election-day simulation with all six apps in use.

---

## 9. Delivery Milestones

| Phase | Duration | Deliverables |
|---|---|---|
| Mobilisation | Month 1 | Repository setup, shared shell components, MDM enrolment |
| Alpha | Month 3 | the Commission Dispatch (M33) + Poll Clerk Companion (M31) internal beta |
| Beta | Month 4-5 | All six apps in closed beta with the Commission test users |
| Public preview (the Commission Verify) | 60 days pre-election | Google Play / App Store public listing |
| Software freeze | 30 days pre-election | No code changes except dual-approved security patches |
| Election Day | E-Day | Full support on-call |
| Post-election | E+60 | Full source transfer + signing keys handover per IPR |

---

## 10. Assumptions and Constraints

### 10.1 Assumptions

- the Commission operates or contracts an MDM console (Google Play Managed / Apple Business Manager) for private-app distribution.
- Firebase Cloud Messaging (or a self-hosted alternative) is acceptable to the Commission for push notification transport.
- Kenya-resident telemetry endpoint provisioned by Datacraft on the Commission infrastructure.
- App-signing keys held or accessible by the Commission to support continuity post-handover.

### 10.2 Constraints

- Fixed-price 3-year contract.
- All Custom Software IPR vests in the Commission.
- Kenya data-residency for authoritative data and telemetry.
- App-store review timelines: Google Play typically 1-7 days; Apple App Store 1-14 days. Plan accordingly for critical updates.

---

## 11. Glossary (abbreviated)

| Term | Definition |
|---|---|
| BVV | Biometric Voter Verification (register lookup) |
| CT | Certificate Transparency |
| DPA | Kenya Data Protection Act 2019 |
| FCM | Firebase Cloud Messaging |
| MDM | Mobile Device Management |
| MFA | Multi-Factor Authentication |
| ODPC | Office of the Data Protection Commissioner |
| OTA | Over-The-Air (updates) |
| OTP | One-Time Password |
| PII | Personally Identifiable Information |
| PWA | Progressive Web App |
| RASP | Runtime Application Self-Protection |
| VPRS | Voter Pre-Registration System |

---

## 12. Document Control

| Version | Date | Author | Change |
|---|---|---|---|
| 1.0 | 2026-08-18 | Datacraft Solutions Architecture | Initial issue with bid |
| 1.1 | 2026-08-21 | Datacraft Solutions Architecture | Integrated into sw/mobile/; MA01-MA06 renumbered M29-M34 |
