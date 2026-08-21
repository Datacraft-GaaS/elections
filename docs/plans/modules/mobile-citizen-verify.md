# M29 mobile-citizen-verify

| field | value |
|---|---|
| M-number | M29 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.7 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M29-mobile-citizen-verify.md |

## Purpose

Put the IEMS in the pocket of every Kenyan who cares to hold the election accountable: a single Flutter app (Android 8+ including KES 10,000 budget devices, iOS 14+, PWA fallback) that lets a citizen confirm register status, find their polling station, start VPRS pre-registration, cryptographically verify polling-station results via Merkle-inclusion proof computed client-side using a Rust FFI verifier, browse candidates, read civic information, and report suspected issues; success metric is >= 10% of voters using it to verify results within 24 hours of publication.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Citizen Verify App (Android / iOS / PWA) | Flutter | services/field-mobile-svc/apps/citizen-verify/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Languages: English + Kiswahili at launch; extensible to Kikuyu, Dholuo, Kalenjin, Kamba, Luhya in phased release. Audio guide in EN + SW for results-verification flow.

## Domain model

Citizen Verify is primarily a read-heavy app with no server-side PII state beyond what BVR, VPRS, and RTS back-ends already hold. Module-local entities:

| entity | key fields or notes | data class | store |
|---|---|---|---|
| CitizenReport | id, report_type, description, photo_blob_id, location_lat, location_lon, location_consent (bool), reporter_session_id (anonymous), status, created_at, report_signature | C | T2 |
| VerificationBookmark | id, user_session_hash (no PII), station_code, bookmarked_at | D | Valkey |

## APIs

OpenAPI file: `packages/openapi-contracts/field/citizen-verify.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/citizen/register | 2 | JWT (OTP-verified) | n/a | own register entry lookup; rate 20/day/session |
| POST /v1/citizen/register/objection | 2 | JWT (OTP-verified) | yes | file rectification / transfer / removal objection via BVR |
| GET /v1/citizen/polling-station | 1 | public | n/a | voter's polling station details (cached) |
| GET /v1/citizen/results/{station_code} | 1 | public | n/a | per-station results with signed Form 34A image |
| GET /v1/citizen/results/{station_code}/proof | 1 | public | n/a | Merkle-inclusion proof for client-side verification |
| GET /v1/citizen/results/bulk/{constituency_code} | 4 | public | n/a | all stations in constituency (rate-limited bulk) |
| GET /v1/citizen/candidates | 1 | public | n/a | candidate directory per election/position/area |
| GET /v1/citizen/civic | 1 | public | n/a | signed civic-information content bundle |
| POST /v1/citizen/reports | 2 | JWT or public + captcha | yes | submit citizen report with optional evidence |
| GET /v1/citizen/reports/{id} | 2 | JWT (own reports) | n/a | report status |
| POST /v1/citizen/notifications/subscribe | 2 | JWT | yes | subscribe to push notifications (no PII in payloads) |
| GET /v1/citizen/app/verify | 1 | public | n/a | app signing-key fingerprint and build info |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.citizen_report_submitted.v1 | emits | eel.field.citizen-report.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_published.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M07 BVV | internal | integrations/bvv/ | yes | register lookup unavailable; show last-cached; no PII shown |
| M04 VPRS Back-End | internal | integrations/vprs/ | yes | deep-link to VPRS portal or embed webview |
| M10 RTS Back-End | internal | integrations/rts/ | yes | CDN cached snapshots; Merkle-proof from RTS |
| M11 Public Portal | internal | integrations/portal/ | yes | results drill-down data; Form 34A images |
| M13 Public Verifier App | internal | integrations/verifier-lib/ | yes | shared Rust FFI Merkle-verifier crate |
| M23 Grievance Channels | internal | integrations/grievance/ | yes | citizen reports escalate through Grievance |
| M14 Reproducibility Bundle | internal | integrations/reproducibility/ | yes | published roots cross-checked during client-side verification |
| I08 SMS / USSD gateway | I08 | integrations/sms/ | yes | OTP delivery; email fallback |
| Google Maps / OpenStreetMap | external | integrations/maps/ | yes | OSM fallback if Google Maps unavailable |

## ML and AI

- None; shared Rust FFI Merkle verifier is deterministic cryptographic code, not ML.

## Dependencies

- Modules: M04, M07, M10, M11, M13, M14, M23
- Platform: none (public CDN; no HSM)
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M29-001: first-launch DPA consent screen; what data collected, why, retention, right to withdraw; must accept to proceed
2. FR-M29-002: language selection (EN + SW at launch; extensible)
3. FR-M29-003: sign-in via ID number + OTP to registered mobile; optional biometric unlock enrolled after sign-in
4. FR-M29-004: "Continue without sign-in" for anonymous browsing of results, candidates, civic info; most read features require no identity
5. FR-M29-005: register lookup; ID number + OTP challenge + own register entry display
6. FR-M29-006: register-entry screen shows name (as registered), polling station, ward, constituency, county, status; DOB masked to year; no biometrics shown
7. FR-M29-007: "Not on register" screen offers VPRS pre-registration path
8. FR-M29-008: rate limit 20 lookups/day per authenticated session
9. FR-M29-009: file objection on own record (rectification, transfer, removal) via BVV back-end
10. FR-M29-010: polling-station finder; registered polling station shown on map (Google Maps with OSM fallback)
11. FR-M29-011: directions button opens native maps app
12. FR-M29-012: offline: last-known polling-station data cached; viewable without network
13. FR-M29-013: "Wrong station?" flow to check current station if voter moved
14. FR-M29-014: VPRS pre-registration start from within app; deep-link or webview with signed challenge
15. FR-M29-015: IPRS verification via VPRS back-end
16. FR-M29-016: SMS OTP for identity verification in pre-registration
17. FR-M29-017: save QR handoff code locally; show at polling centre
18. FR-M29-018: track pre-registration application status
19. FR-M29-019: election picker; list of gazetted elections
20. FR-M29-020: national to county to constituency to ward to polling-station drill-down
21. FR-M29-021: per-station page shows results per candidate per position; signed Form 34A image; transparency-log index; anonymised agent countersignatures
22. FR-M29-022: "Verify this result" button; Merkle-inclusion proof fetched; computed client-side using Rust FFI verifier; compared to published root from transparency-log service; shows "Verified" or "Failed to verify"
23. FR-M29-023: bulk verification; verify all polling stations in voter's constituency in one action
24. FR-M29-024: bookmark specific polling stations for follow-along notifications
25. FR-M29-025: share verified snapshot (with signature bundle) via native share to WhatsApp / Twitter / email; recipient can independently verify
26. FR-M29-026: candidate directory per election, per position, per area
27. FR-M29-027: candidate profile: portrait, party, biographic (public fields only), symbol, ballot order
28. FR-M29-028: candidate search by name / party / area
29. FR-M29-029: curated civic information from IEBC; how to vote, what to bring, polling hours, rights
30. FR-M29-030: signed civic-information content bundle fetched from IEBC; managed by IEBC
31. FR-M29-031: in-app report form; type, description, optional photo / short video, optional location (opt-in per report)
32. FR-M29-032: location consent opt-in per report only
33. FR-M29-033: report status tracked; anonymised for public reporting when relevant
34. FR-M29-034: anti-abuse: rate limit + captcha for anonymous reports; verified reports from signed-in users prioritised
35. FR-M29-035: push notifications (opt-in): registration-window opens, own entry updated, own polling-station results published, bookmarked-station results published
36. FR-M29-036: no PII in push payloads; content fetched authenticated on receipt
37. FR-M29-037: WCAG 2.2 AA on every screen
38. FR-M29-038: TalkBack / VoiceOver support
39. FR-M29-039: high-contrast mode
40. FR-M29-040: font-size scaling up to 200%
41. FR-M29-041: audio-guided workflow in EN + SW for results-verification flow
42. FR-M29-042: text-only fallback for slow connections
43. FR-M29-043: in-app "Verify this app" screen; signing-key fingerprint + version + build info; matches public fingerprints on eel.or.ke and canonical domains
44. Device attestation: required (soft-fail warn); biometric unlock optional; root / jailbreak warn on objection-filing screens; screen-capture protection on PII screens; no PII in logs
45. Cold start <= 3 s on mid-tier Android 8; verification cycle <= 5 s per station; download size <= 50 MB; battery <= 3% per hour foreground; 5M concurrent verifications via CDN

## Acceptance criteria

1. AC-M29-01: full verification flow end-to-end on real Android device; UAT
2. AC-M29-02: cached content works offline; airplane-mode test
3. AC-M29-03: Merkle-inclusion proof validates on device; live retrieval
4. AC-M29-04: signed snapshot shareable and independently verifiable by recipient; share flow
5. AC-M29-05: WCAG 2.2 AA audit pass; independent audit
6. AC-M29-06: Google Play + Apple App Store submission approved; store review
7. AC-M29-07: 5M concurrent verifications sustained; load test on back-end
8. AC-M29-08: Kiswahili native-speaker signed off; human reviewer
9. AC-M29-09: data-safety declaration + privacy nutrition label accurate; manual check
10. AC-M29-10: runs on Android 8 mid-tier device with <= 3 s cold start; device bench

## Applicable edge cases

**connectivity**
- EC-CON-01: no network; polling-station cached; results verification queued or shown from cache; clear "last updated" timestamp shown
- EC-CON-03: 2G network; text-only fallback; form images deferred

**identity**
- EC-IDN-04: OTP delivery fails; retry up to 3 times; email fallback; suggest official kiosk

**data**
- EC-DAT-10: verification fails (signature mismatch on result); mark "Unverifiable"; never show as "Verified"

**security**
- EC-SEC-02: app attestation fails; warn user; degrade to read-only; no objection filing
- EC-SEC-09: store impersonation; user warning + link to authentic version on eel.or.ke

**human**
- EC-HUM-06: screen readers; TalkBack / VoiceOver; AA compliance on all screens
- EC-HUM-08: language selection at launch; extensible to 6+ Kenyan languages

**scale**
- EC-SCL-01: 5M concurrent result verifications via CDN; signed snapshot cache; back-end Merkle service on separate read path
- EC-SCL-02: register lookup rate-limited 20/day; explain limit; suggest official kiosk

**mobile**
- EC-MOB-01: battery < 20% on election day; text-only mode; reduced animation; polling-station finder still works
- EC-MOB-02: Android 8 mid-tier; cold start <= 3 s; download <= 50 MB
- EC-MOB-04: GPS unavailable; polling-station lookup from register data (no GPS required)
- EC-MOB-07: 5M concurrent verifications; CDN edge + signed-snapshot cache absorbs peak

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Merkle-inclusion verifier (Rust FFI), rate-limit guard, report anti-abuse, DPA consent gate | nextest + flutter_test | M29-UNIT- |
| property | Merkle proof always false on tampered data | proptest | M29-PROP- |
| contract | BVV register lookup, RTS form retrieval, Merkle-proof API | Pact | M29-PACT- |
| integration | full verification flow (fetch + proof + FFI compute); objection filing; VPRS deep-link | integration_test | M29-INT- |
| e2e | voter journey: sign-in, register lookup, results drill-down, verify, share snapshot | Playwright + Flutter driver | M29-E2E- |
| security | attestation fail degrades correctly; screen-capture blocked on PII screens; no PII in logs | ZAP + manual | M29-SEC- |
| a11y | TalkBack / VoiceOver; high-contrast; font scaling 200%; audio guide | axe-core + manual | M29-A11Y- |
| offline | polling-station data cached; verification from cached proof | netem | M29-OFFLN- |
| load | 5M concurrent verifications; CDN cache validation; back-end Merkle service | k6 | M29-LOAD- |
| i18n | EN + SW correctness; native-speaker sign-off | manual | M29-I18N- |

Tags: `testWidgets('verify result [FR-M29-022,AC-M29-03,EC-DAT-10]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - create for citizen-verify and shared mobile back-end paths
- Grafana dashboard: `platform/observability/field-mobile-svc/citizen-verify-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/citizen-verify-slo-alerts.yaml` (verification latency < 5 s, CDN availability 99.5%)

## Open questions

- Shared Rust FFI Merkle-verifier library: must be extracted into `libeel-verifier` (Android JNI + iOS XCFramework) before M13 or M29 ships; dependency on M13 team; resolve ownership before integration sprint per MOBILE-INTEGRATION-NOTES.md §8.
- Google Play + Apple App Store account registration: IEBC must be the publisher entity; account setup must begin at T-180.
- International language packs (Kikuyu, Dholuo, Kalenjin, Kamba, Luhya): translator contracts must be in place by T-90; phased release timeline must be confirmed.
