# Acceptance Criteria — the Commission IEMS Mobile Apps (sw/ Integration)

**Integration note.** This document is adapted from `mobile/ACCEPTANCE-CRITERIA.md` (source of truth pending stitching-pass consolidation). It is a companion to `../ACCEPTANCE-CRITERIA.md` (the main sw/ acceptance criteria). The stitching pass should add a "Mobile Suite" section to `../ACCEPTANCE-CRITERIA.md` referencing this file and the per-app criteria in the M29-M34 module specs.

Per-app functional acceptance criteria live in:
- `../modules/M29-mobile-citizen-verify.md` §8
- `../modules/M30-mobile-party-agent.md` §8
- `../modules/M31-mobile-poll-clerk.md` §8
- `../modules/M32-mobile-field-support.md` §8
- `../modules/M33-mobile-personnel-dispatch.md` §8
- `../modules/M34-mobile-observer.md` §8

Passing all criteria per gate is required to proceed.

---

## 1. Common Build / QA

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MB-01 | Every app builds reproducibly | CI + verifier | Byte-identical artefacts across 3 builds |
| MB-02 | Unit test coverage >= 80% | flutter_test | Coverage report |
| MB-03 | Widget test coverage >= 60% | flutter_test | Coverage report |
| MB-04 | Integration test happy-path + edge cases per app | integration_test | All green |
| MB-05 | Contract tests vs back-end | Pact | All contracts pass |
| MB-06 | Golden tests: no unintended UI regression | golden_toolkit | Diffs reviewed |
| MB-07 | Static analysis clean (dart analyze + custom lints) | CI | Zero warnings |
| MB-08 | SBOM (SPDX) generated + signed | cosign | Verifiable |
| MB-09 | App-signing key stored in HSM or Commission-controlled escrow | Ceremony | Attested |

---

## 2. Platform Compatibility

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MC-01 | M29 runs on >= 30 real Android devices via Firebase Test Lab | Test Lab | All green |
| MC-02 | M29 runs on iOS 14+ | Real device pool | All green |
| MC-03 | Staff apps (M30, M31, M32, M34) run on Android 10+ + iOS 15+ | Real device pool | All green |
| MC-04 | PWA fallback for M29 + M33 | Browser matrix | Chrome / Safari / Firefox mobile |
| MC-05 | Battery cost <= 3% per hour foreground use | Device benchmark | Meets target |
| MC-06 | Cold start <= 3 s on mid-tier Android 8 | Bench | Meets target |
| MC-07 | Download size <= 50 MB for citizen apps (M29, M33) | APK / IPA size | Meets target |

---

## 3. Offline-First

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MO-01 | Every app functions with no network per per-app spec | Airplane mode test | Meets per-app spec |
| MO-02 | Sync queue survives app kill + phone reboot | Chaos exercise | Queue intact |
| MO-03 | Sync engine drains queue on connectivity restoration with exponential backoff | Network flap | Meets |
| MO-04 | Idempotency keys prevent duplicate writes on retry | Fault injection | No duplicates |
| MO-05 | Conflict resolution per record type per spec | Bench | Meets |

---

## 4. Security (per `SECURITY.md` in this folder)

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MS-01 | Independent mobile pentest | MobSF + manual by accredited firm | No unresolved P1 findings |
| MS-02 | Certificate pinning refuses rogue CA | MITM proxy test | Refuses connection |
| MS-03 | Root / jailbreak detection per app policy | Root device test | Meets policy |
| MS-04 | App attestation via Play Integrity / App Attest | Live test | Attestation passes |
| MS-05 | Hooking framework detection (Frida) refuses staff apps | Frida attach test | Refuses |
| MS-06 | Secure enclave key custody | Storage inspection | Keys not exportable |
| MS-07 | No PII in logs / crash reports / telemetry | Log scan | Clean |
| MS-08 | Signature verification of results snapshots on device | Live retrieval | Signature validates |
| MS-09 | Merkle-inclusion proof verifies for any polling station | Live retrieval + validation | Verifies |
| MS-10 | OTA rollback works | Chaos exercise | Rolls back cleanly |

---

## 5. Privacy

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MP-01 | Consent capture at first launch matches privacy policy | Manual | Matches |
| MP-02 | Data-safety declaration on Play Console accurate | Manual | Accurate |
| MP-03 | iOS Privacy Nutrition Label accurate | Manual | Accurate |
| MP-04 | No third-party analytics or ad SDKs | Static + dynamic analysis | Clean |
| MP-05 | Right-to-erasure in-app action completes within 30 days | Workflow test | Meets |
| MP-06 | ODPC audit passes | ODPC review | Compliant |

---

## 6. Accessibility

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MA-01 | WCAG 2.2 AA compliance on every screen | Independent audit | Compliant |
| MA-02 | TalkBack (Android) traversal completes primary flow | Manual | Complete |
| MA-03 | VoiceOver (iOS) traversal completes primary flow | Manual | Complete |
| MA-04 | Font-size scaling to 200% does not break layout | Bench | No break |
| MA-05 | High-contrast mode operational | Manual | Works |
| MA-06 | Audio-guided workflow for M29 + M33 in EN + SW | Manual | Works |
| MA-07 | Touch targets >= 48dp | Automated scan | Compliant |

---

## 7. Localisation

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| ML-01 | English + Kiswahili at launch, native-speaker signed off | Native reviewer | Signed off |
| ML-02 | Language switch persists across sessions | Bench | Persists |
| ML-03 | Date, time, number formatting per Kenyan locale | Bench | Correct |
| ML-04 | No hard-coded user-facing strings in code | Linter | Zero |
| ML-05 | International observer language pack (FR, PT) loads in M34 | Bench | Loads correctly |

---

## 8. App Store Submission

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MA-STORE-01 | Google Play submission passes review | Play Console | Approved |
| MA-STORE-02 | Apple App Store submission passes review | App Store Connect | Approved |
| MA-STORE-03 | Store screenshots accurately reflect app | Manual review | Accurate |
| MA-STORE-04 | Signed APK direct-download available on eel.example with fingerprint publication | Manual | Available |

---

## 9. MDM Distribution (private apps M30, M31, M32, M34)

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MM-01 | Google Play Managed private distribution works | Managed device test | Works |
| MM-02 | Apple Business Manager distribution works | Managed device test | Works |
| MM-03 | Per-user allowlist tied to accreditation database | Bench | Enforced |
| MM-04 | Force-update policy applied within 48h of publication | Bench | Meets |

---

## 10. Field Trial

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MF-01 | Multi-county real deployment during pilot | Field ops | Documented sign-off |
| MF-02 | Signal reception + biometric on real Kenyan devices | Field ops | Works |
| MF-03 | Sync queue behaviour on 2G / 3G / 4G / LTE | Field ops | Meets |
| MF-04 | Offline flow at rural polling station | Field ops | Meets |

---

## 11. Dress Rehearsal

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MD-01 | Full election-day simulation with all 6 apps (M29-M34) | Sandbox exercise | Complete |
| MD-02 | War-room can observe app usage in real time | Sandbox exercise | Observable |
| MD-03 | Support ticket flow via M32 (Field Support app) closes | Sandbox exercise | Closes |
| MD-04 | M33 personnel attendance app deploys at scale | Sandbox exercise | Meets |

---

## 12. Handover

| # | Criterion | Method | Pass condition |
|---|---|---|---|
| MH-01 | Source code + build scripts + docs handed to the Commission | Handover ceremony | Complete |
| MH-02 | App-signing keys transferred or escrow procedure documented | Handover ceremony | Complete |
| MH-03 | Google Play + Apple Developer accounts owned or transferable to the Commission | Handover ceremony | Complete |
| MH-04 | Training complete for the Commission ICT mobile team | Certification | Complete |
| MH-05 | MDM console configuration handed over | Handover ceremony | Complete |
