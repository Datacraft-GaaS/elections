# M29 — Mobile Citizen Verify (the Commission Verify App)

**Module ID:** M29
**Module Name:** Mobile Citizen Verify — the Commission Verify App
**Type:** Mobile Application (Flutter, Android 8.0+ / iOS 14+ / PWA fallback)
**Distribution:** Google Play, Apple App Store, signed APK from eel.example
**Users:** Registered and prospective voters (~35M reachable Kenyans)
**Owner:** Datacraft
**Status:** Specified — pending implementation
**IPR:** Custom Software vests in the Commission per GCC 15.4 (Teaming §4)

Adapted from `mobile/apps/MA01-commission-verify.md`, sw/-module renumbered as M29.

---

## 1. Purpose

Put the entire IEMS in the pocket of every Kenyan who cares to hold the election accountable. One app that lets a citizen: confirm they are on the register, find their polling station on election day, pre-register if not registered, verify the result of their polling station with a cryptographic Merkle-inclusion proof, browse candidates, read civic information, and report suspected issues.

Flagship user-visible expression of the IEMS transparency posture. Success = >= 10% of voters use it to verify results within 24h of publication.

**Relationship to M13 (Public Verifier App):** M29 is the native-mobile, Commission-branded implementation targeting the widest handset base (Android 8+ including KES 10,000 budget devices). M13 is the multi-channel superset covering short-code, SMS, WhatsApp, and PWA paths. Both modules persist as distinct specifications; the stitching pass may consolidate shared code into a common verifier library. See `sw/MOBILE-INTEGRATION-NOTES.md` §3.1.

---

## 2. Personas

- Registered voter (primary — 30M eligible)
- Prospective voter starting VPRS pre-registration
- Curious citizen, media, observer

---

## 3. Feature Set (Functional Requirements)

### 3.1 Onboarding

- **FR-M29-001** First-launch consent screen per Kenya DPA: what data collected, why, retention, right to withdraw. User must accept to proceed.
- **FR-M29-002** Language selection (English / Kiswahili at launch; extensible to Kikuyu, Dholuo, Kalenjin, Kamba, Luhya in phased release).
- **FR-M29-003** Sign-in via ID number + OTP to registered mobile. Optional biometric unlock enrolled after sign-in.
- **FR-M29-004** "Continue without sign-in" for anonymous browsing (results verification, candidate info, civic info) — most read features do not require identity.

### 3.2 Register Lookup (BVV integration)

- **FR-M29-005** Enter ID number → OTP challenge → view own register entry.
- **FR-M29-006** Register-entry screen shows: name (as registered), polling station (name, code, GPS), ward, constituency, county, status. DOB masked to year. Biometrics never shown.
- **FR-M29-007** "Not on register" screen offers VPRS pre-registration path.
- **FR-M29-008** Rate limit: 20 lookups/day per authenticated session.
- **FR-M29-009** File an objection on own record (rectification, transfer, removal) via BVV back-end.

### 3.3 Polling Station Finder

- **FR-M29-010** Show voter's registered polling station on a map (Google Maps / OpenStreetMap fallback).
- **FR-M29-011** Directions button opens native maps app.
- **FR-M29-012** Offline: last-known polling-station data cached locally, viewable without network.
- **FR-M29-013** "Wrong station?" flow to check current station if voter moved.

### 3.4 VPRS Pre-Registration (Prospective Voters)

- **FR-M29-014** Start VPRS application from within app; deep-link to VPRS portal for the full form (or embed in a webview with signed challenge).
- **FR-M29-015** IPRS verification via VPRS back-end.
- **FR-M29-016** SMS OTP for identity verification.
- **FR-M29-017** Save QR handoff code locally; show at polling centre.
- **FR-M29-018** Track application status.

### 3.5 Results Verification

- **FR-M29-019** Election picker: list of gazetted elections.
- **FR-M29-020** National → County → Constituency → Ward → Polling Station drill-down.
- **FR-M29-021** Per-polling-station page shows: results per candidate per position, signed Form 34A image (viewable + downloadable), transparency-log index, agent countersignatures list (anonymised).
- **FR-M29-022** "Verify this result" button — fetches Merkle-inclusion proof, computes root client-side using Rust FFI verifier, compares to published root fetched independently from transparency-log service, shows "Verified" or "Failed to verify".
- **FR-M29-023** Bulk verification: verify all polling stations in the user's constituency in one action.
- **FR-M29-024** Bookmark specific polling stations for follow-along notifications.
- **FR-M29-025** Share verified snapshot (with signature bundle) via native share to WhatsApp / Twitter / email — recipient can independently verify.

### 3.6 Candidate Directory

- **FR-M29-026** Browse candidates per election, per position, per area.
- **FR-M29-027** Candidate profile: portrait, party, biographic (public fields only), symbol, ballot order.
- **FR-M29-028** Search by name / party / area.

### 3.7 Civic Information

- **FR-M29-029** Curated civic information from the Commission: how to vote, what to bring, polling hours, rights.
- **FR-M29-030** Content managed by the Commission; app fetches signed content bundles.

### 3.8 Report an Issue

- **FR-M29-031** In-app form to report suspected register error, polling-day issue, incident. Attach photo / short video / location.
- **FR-M29-032** Consent to share location for the report (opt-in per report).
- **FR-M29-033** Report status tracked; anonymised for public reporting when relevant.
- **FR-M29-034** Anti-abuse: rate limit + captcha for anonymous reports; verified reports from signed-in users prioritised.

### 3.9 Notifications (opt-in)

- **FR-M29-035** Push notifications for: registration-window opens, own register entry updated, own polling-station results published, bookmarked-station results published.
- **FR-M29-036** No PII in push payloads; content fetched authenticated on receipt.

### 3.10 Accessibility and Localisation

- **FR-M29-037** WCAG 2.2 AA on every screen (upgraded from WCAG 2.1 per Teaming Annex A).
- **FR-M29-038** TalkBack / VoiceOver support.
- **FR-M29-039** High-contrast mode.
- **FR-M29-040** Font-size scaling up to 200%.
- **FR-M29-041** Audio-guided workflow in English + Kiswahili for the results-verification flow.
- **FR-M29-042** Text-only fallback for slow connections.

### 3.11 Verify Authenticity

- **FR-M29-043** In-app "Verify this app" screen shows signing-key fingerprint + version + build info; matches public fingerprints published on eel.example + multiple canonical domains.

---

## 4. Non-Functional Requirements

- **NFR-M29-001** Cold start <= 3 s on mid-tier Android 8.
- **NFR-M29-002** Results verification (fetch + client-side proof compute) <= 5 s per polling station.
- **NFR-M29-003** Download size <= 50 MB.
- **NFR-M29-004** Works offline for cached content (own polling station, downloaded results, cached candidates, civic info).
- **NFR-M29-005** Handles 5M concurrent verification requests via CDN + signed snapshot cache.
- **NFR-M29-006** Battery cost <= 3% per hour foreground.
- **NFR-M29-007** WCAG 2.2 AA independent audit pass.

---

## 5. Integrations

- BVV back-end (M07)
- VPRS back-end (M04)
- Public Results Portal (M11) for results snapshots and drill-down data
- Public Verifier App back-end (M13) — shared Merkle-proof API endpoint
- Transparency log service
- IPRS via VPRS
- SMS Gateway (I08)
- Commission-controlled telemetry endpoint

---

## 6. Security Profile

| Control | Setting |
|---|---|
| Device attestation | Required (soft-fail warn) |
| mTLS | Public endpoints + certificate pinning |
| Biometric unlock | Optional (recommended after first login) |
| Root/jailbreak detection | Warn on staff-critical features (e.g., objections) |
| Screen-capture protection | On PII screens (register lookup, objection filing) |
| Anti-tamper | Signature check on launch |
| PII in logs | None |

---

## 7. Failure Modes

| Failure | Response |
|---|---|
| Network unreachable | Cached content served; verification queued |
| OTP delivery fails | Retry / email fallback |
| Register lookup rate-limited | Explain limit; suggest official kiosk |
| Signature verification fails on result | Mark "Unverifiable"; do NOT show as verified |
| App attestation fails | Warn user; degrade to read-only |
| Store impersonation detected | User warning + link to authentic version |

---

## 8. Acceptance Criteria

| # | Criterion | Method |
|---|---|---|
| A-M29-01 | Full verification flow end-to-end on real Android device | UAT |
| A-M29-02 | Cached content works offline | Airplane mode test |
| A-M29-03 | Merkle-inclusion proof validates on device | Live retrieval |
| A-M29-04 | Signed snapshot shareable and independently verifiable by recipient | Share flow |
| A-M29-05 | WCAG 2.2 AA audit pass | Independent audit |
| A-M29-06 | Google Play + Apple App Store submission approved | Store review |
| A-M29-07 | 5M concurrent verifications sustained | Load test on back-end |
| A-M29-08 | Kiswahili native-speaker signed off | Reviewer |
| A-M29-09 | Data-safety declaration + privacy nutrition label accurate | Manual |
| A-M29-10 | Runs on Android 8 mid-tier device with <= 3 s cold start | Bench |

---

## 9. Cross-References to Other Modules

| Module | Relationship |
|---|---|
| M04 (VPRS) | Pre-registration entry point and IPRS identity check |
| M07 (BVV) | Register lookup, objection filing |
| M10 (RTS Backend) | Merkle-inclusion proof retrieval |
| M11 (Public Portal) | Results data, Form 34A images, transparency-log roots |
| M13 (Public Verifier App) | Multi-channel superset; shared Merkle-proof API; **shared library required**: extract Rust verifier crate into `libcommission-verifier` (Android JNI + iOS XCFramework) before either module ships; see MOBILE-INTEGRATION-NOTES.md §8 |
| M14 (Reproducibility Bundle) | Published roots cross-checked during client-side verification |
| M23 (Grievance) | Issue reports from citizens escalate through Grievance module |

---

## 10. Traceability to Strategy Paper

| Strategy Part / Section | Requirement driven |
|---|---|
| Part IX (Public Transparency) | FR-M29-019 through FR-M29-025 — results verification with Merkle proof |
| Part XIV.3 (Citizen Engagement) | FR-M29-001 through FR-M29-004, FR-M29-026 through FR-M29-030 |
| Part XV.1 (Accessibility) | FR-M29-037 through FR-M29-042 |
| Part XV.3 (Localisation) | FR-M29-002, FR-M29-041 |
| Part XVI.4 (Public-Facing Mobile) | Full module scope; success metric >= 10% voter take-up |

---

## 11. Source Traceability

Adapted from `mobile/apps/MA01-commission-verify.md`. References to "MA01" in the source file correspond to "M29" in this sw/-module. Source file is the canonical pre-stitching specification; this file is the sw/-integrated form.
