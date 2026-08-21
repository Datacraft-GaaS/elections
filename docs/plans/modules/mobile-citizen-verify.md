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
2. FR-M29-002: language selection (EN + SW at launch; extensible to Kikuyu, Dholuo, Kalenjin, Kamba, Luhya in phased release)
3. FR-M29-003: sign-in via ID number + OTP to registered mobile; optional biometric unlock enrolled after sign-in
4. FR-M29-004: "Continue without sign-in" for anonymous browsing of results, candidates, civic info; most read features require no identity
5. FR-M29-005: register lookup; ID number + OTP challenge + own register entry display
6. FR-M29-006: register-entry screen shows name (as registered), polling station, ward, constituency, county, status; DOB masked to year; no biometrics shown
7. FR-M29-007: "Not on register" screen offers VPRS pre-registration path
8. FR-M29-008: rate limit 20 lookups/day per authenticated session; 429 with Retry-After returned on breach
9. FR-M29-009: file objection on own record (rectification, transfer, removal) via BVV back-end
10. FR-M29-010: polling-station finder; registered polling station shown on map (Google Maps with OSM fallback)
11. FR-M29-011: directions button opens native maps app
12. FR-M29-012: offline: last-known polling-station data cached in Isar + SQLCipher; viewable without network; "last updated" timestamp shown
13. FR-M29-013: "Wrong station?" flow to check current station if voter moved
14. FR-M29-014: VPRS pre-registration start from within app; deep-link or webview with signed challenge
15. FR-M29-015: IPRS verification via VPRS back-end
16. FR-M29-016: SMS OTP for identity verification in pre-registration; email fallback on OTP delivery failure
17. FR-M29-017: save QR handoff code locally; show at polling centre
18. FR-M29-018: track pre-registration application status
19. FR-M29-019: election picker; list of gazetted elections
20. FR-M29-020: national to county to constituency to ward to polling-station drill-down
21. FR-M29-021: per-station page shows results per candidate per position; signed Form 34A image; transparency-log index; anonymised agent countersignatures
22. FR-M29-022: "Verify this result" button; Merkle-inclusion proof fetched; computed client-side using Rust FFI verifier; compared to published root from transparency-log service; shows "Verified" or "Failed to verify"
23. FR-M29-023: bulk verification; verify all polling stations in voter's constituency in one action
24. FR-M29-024: bookmark specific polling stations for follow-along notifications
25. FR-M29-025: share verified snapshot (with signature bundle) via native share to WhatsApp / Twitter / email; recipient can independently verify without the app
26. FR-M29-026: candidate directory per election, per position, per area
27. FR-M29-027: candidate profile: portrait, party, biographic (public fields only), symbol, ballot order
28. FR-M29-028: candidate search by name / party / area
29. FR-M29-029: curated civic information from the election authority; how to vote, what to bring, polling hours, rights
30. FR-M29-030: signed civic-information content bundle fetched and cached; managed by the election authority
31. FR-M29-031: in-app report form; type, description, optional photo / short video, optional location (opt-in per report)
32. FR-M29-032: location consent opt-in per report only; no background location collected
33. FR-M29-033: report status tracked; anonymised for public reporting when relevant
34. FR-M29-034: anti-abuse: rate limit + captcha for anonymous reports; verified reports from signed-in users prioritised
35. FR-M29-035: push notifications (opt-in): registration-window opens, own entry updated, own polling-station results published, bookmarked-station results published
36. FR-M29-036: no PII in push payloads; content fetched authenticated on receipt
37. FR-M29-037: WCAG 2.2 AA on every screen; axe-core audit in CI
38. FR-M29-038: TalkBack / VoiceOver support; all interactive elements reachable via accessibility tree
39. FR-M29-039: high-contrast mode
40. FR-M29-040: font-size scaling up to 200% without horizontal scroll or text truncation
41. FR-M29-041: audio-guided workflow in EN + SW for results-verification flow
42. FR-M29-042: text-only fallback for slow connections; form images deferred on 2G
43. FR-M29-043: in-app "Verify this app" screen; signing-key fingerprint + version + build info; matches public fingerprints on canonical domains
44. FR-M29-044: screen-capture protection on PII screens (register lookup, objection filing); OS-level screen-share disabled during register view; no PII in logs
45. FR-M29-045: root/jailbreak detection warns on objection-filing and register-lookup screens; degrades to read-only browse; soft-warn, not hard-fail
46. FR-M29-046: device attestation required (soft-fail warn); biometric unlock optional; attestation failure warns user; read-only browse continues
47. FR-M29-047: cold start <= 3 s on mid-tier Android 8; verification cycle (fetch + Rust FFI proof) <= 5 s per station; download size <= 50 MB; battery <= 3% per hour foreground
48. FR-M29-048: Isar + SQLCipher local store for cached polling station, candidate, and civic content; SQLCipher key derived from device keystore; no plaintext at rest
49. FR-M29-049: dio + mTLS interceptor chain for all API calls; certificate pinning on field-mobile-api; pinned certificates updated via signed OTA
50. FR-M29-050: Riverpod state management; verification state immutable after proof computed; no mutable shared state across concurrent verification tasks
51. FR-M29-051: UUID7 IDs for all CitizenReport entities; generated locally with eel-crypto-rs FFI shim; server validates UUID7 format on receipt
52. FR-M29-052: consumer lag on eel.field.citizen-report.v1 monitored; alert fires if lag > 60 s; backpressure; no OOM
53. FR-M29-053: 5M concurrent verifications served via CDN edge with signed snapshots; back-end Merkle service on separate read path; p99 verification latency <= 5 s at 5M concurrency
54. FR-M29-054: verification failure (signature mismatch or Merkle proof fail) shown as "Unverifiable" with explanation; never shown as "Verified"; failure logged client-side for debugging

## Acceptance criteria

1. AC-M29-01: full verification flow end-to-end on real Android device; measurement: UAT sign-off
2. AC-M29-02: cached polling-station content and results work in airplane mode; measurement: airplane-mode integration test
3. AC-M29-03: Merkle-inclusion proof validates on device using Rust FFI verifier against live published root; measurement: live retrieval test
4. AC-M29-04: signed snapshot shareable and independently verifiable by recipient without the app; measurement: share-flow test + independent verification
5. AC-M29-05: WCAG 2.2 AA audit pass on all screens; measurement: independent axe-core + manual audit report
6. AC-M29-06: Google Play + Apple App Store submission approved before T-60; measurement: store approval timestamp
7. AC-M29-07: 5M concurrent verifications sustained at p99 <= 5 s; measurement: k6 load test on back-end + CDN
8. AC-M29-08: Kiswahili native-speaker signed off on all screen text; measurement: human reviewer sign-off record
9. AC-M29-09: data-safety declaration + privacy nutrition label accurate on both stores; measurement: manual check against actual data collection
10. AC-M29-10: cold start <= 3 s on mid-tier Android 8 reference device; measurement: device bench on reference hardware
11. AC-M29-11: OS kill mid-verification resumes correctly; no duplicate report submissions; measurement: background-kill integration test at each step
12. AC-M29-12: duplicate CitizenReport submission same key+body returns cached response without re-execution; measurement: auto-generated idempotency test

## Applicable edge cases

**connectivity**
- EC-CON-01: offline 48 h; polling-station cached; results from cache; clear "last updated" timestamp shown
- EC-CON-02: 3G (500 kbps, 300 ms RTT); text-only fallback; form images deferred; verification SLA met
- EC-CON-03: 2G only; text-only fallback; form images deferred; USSD path suggested
- EC-CON-04: site partition primary/mirror; CDN serves cached signed snapshot; back-end Merkle on separate path
- EC-CON-05: flapping link; dio retry with backoff+jitter; no duplicate submissions
- EC-CON-06: captive portal; pinning fails closed; "network blocked" UX; no token leak
- EC-CON-07: satellite 600 ms+ RTT; transport-tuned timeouts; MTU-safe; text-only CDN path
- EC-CON-08: DNS poisoned/down; hardcoded fallback endpoints; circuit opens
- EC-CON-09: IPsec tunnel drop mid-report; idempotent re-send on reconnect
- EC-CON-10: dual-SIM failover mid-OTP; retry on SIM B; one OTP consumed
- EC-CON-11: Kafka ISR shrink on eel.field.citizen-report.v1; producer blocks; outbox accumulates; no loss

**identity**
- EC-IDN-01: kit cert revoked mid-session; not applicable (no kit in M29); API JWT refresh handles session boundary
- EC-IDN-02: JWT expires mid-request; in-flight completes; next 401 + silent refresh
- EC-IDN-03: SVID rotates during batch; no SVID in M29 public path; field-mobile-api SVID rotation transparent to client
- EC-IDN-04: OTP delivery fails; retry up to 3 times; email fallback; suggest official kiosk
- EC-IDN-05: rooted/emulator/Frida; root warn on objection-filing screen; Frida RASP terminates
- EC-IDN-06: clone app; pin mismatch; backend rejects digest; user warned and directed to authentic store listing
- EC-IDN-07: SIM swap on OTP; OTP device-bound; new-device OTP challenge required
- EC-IDN-08: voter under duress; no covert alert path in M29 public app; refer to grievance report flow
- EC-IDN-09: dual-control approver revoked; not applicable in M29 public path
- EC-IDN-10: SpiceDB down; register lookup fail-closed; 503 problem+json; cached read available
- EC-IDN-11: realm key rotated; stale JWKS; refetch on kid miss; no access interruption

**data**
- EC-DAT-01: duplicate CitizenReport submission same key+body; cached response; no re-execution
- EC-DAT-02: same report key, different body; 422 idempotency-key-reuse
- EC-DAT-10: verification fails (signature mismatch on result); mark "Unverifiable"; never show as "Verified"

**security**
- EC-SEC-05: injection on all citizen API inputs; parameterised only; Semgrep + ZAP + fuzz
- EC-SEC-09: store impersonation; user warning + link to authentic version on canonical domain
- EC-SEC-16: secrets absent from all client logs and server traces; CI regex gate + log-scanner fixture

**scale**
- EC-SCL-01: 5M concurrent result verifications via CDN; signed snapshot cache; back-end Merkle service on separate read path
- EC-SCL-02: register lookup rate-limited 20/day; explain limit; suggest official kiosk
- EC-SCL-04: consumer lag on eel.field.citizen-report.v1 > 60 s; page alert; backpressure; no OOM

**mobile**
- EC-MOB-01: OS kills mid-sync; resumable; no duplicate; WAL intact
- EC-MOB-02: storage full; graceful refusal; bounded queue; user prompted to free space
- EC-MOB-03: 1 GB RAM KES-10k device; <= 15 MB RAM; p99 <= 2 s on 3G; no OOM
- EC-MOB-04: Doze/background limits; resume on foreground; polling-station finder works without GPS
- EC-MOB-05: push token rotation / FCM down; idempotent re-register; SMS for critical alerts
- EC-MOB-06: update mid-election below floor; force floor; 7-day block; cached content survives update
- EC-MOB-07: GPS spoof check-in; not applicable (M29 has no GPS check-in); polling-station lookup from register data
- EC-MOB-08: device time wrong; server authoritative timestamp used for "last updated" display
- EC-MOB-09: biometric sensor broken; PIN fallback rate-limited; optional biometric does not block access

**human**
- EC-HUM-06: screen readers; TalkBack / VoiceOver; AA compliance on all screens
- EC-HUM-08: language selection at launch; extensible to 6+ Kenyan languages

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
- Google Play + Apple App Store account registration: the election authority must be the publisher entity; account setup must begin at T-180.
- International language packs (Kikuyu, Dholuo, Kalenjin, Kamba, Luhya): translator contracts must be in place by T-90; phased release timeline must be confirmed.
