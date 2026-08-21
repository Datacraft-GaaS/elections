# M64 voter-list-public-inspection

| field | value |
|---|---|
| M-number | M64 |
| Home service | portal-svc (ADR-0020) |
| Owning work package | WP-5.7 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M64-voter-list-public-inspection.md |

## Purpose

Publishes verifiable roll snapshots for the statutory public inspection window, accepts voter objections and correction claims with biometric claimant verification, and auto-refers disputed claims to eCourt at the T-60 deadline.

## Surfaces and languages

| surface | technology | path under services/portal-svc/ |
|---|---|---|
| Public voter inspection portal | Astro | astro/inspection/ |
| Admin inspection management | Django | django/portal-admin/ |
| Internal API | Rust (axum) | crates/inspection/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| RollSnapshot | id, election_id, snapshot_date, voter_count, merkle_root, content_url, signed_at | C | T1-consensus |
| Objection | id, snapshot_id, objector_id_hash, subject_voter_id_hash, grounds, status (pending/upheld/dismissed), filed_at | B | T1-consensus |
| CorrectionClaim | id, snapshot_id, claimant_id_hash, biometric_verify_ref, field_to_correct, proposed_value, status (pending/verified/applied/rejected), filed_at | B | T1-consensus |
| AutoReferralPacket | id, claim_id, ecourt_ref, referred_at, idempotent_key | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/inspection/snapshots | 1 signed-static | public | n/a | list roll snapshots |
| GET /v1/inspection/snapshots/{id} | 1 signed-static | public | n/a | snapshot metadata and Merkle root |
| GET /v1/inspection/snapshots/{id}/verify | 2 live-per-user | public | n/a | voter self-check (anti-enumeration guards) |
| POST /v1/inspection/objections | 2 live-per-user | JWT | yes | file objection |
| GET /v1/inspection/objections/{id} | 2 live-per-user | JWT | n/a | objection status |
| POST /v1/inspection/corrections | 2 live-per-user | JWT | yes | file correction claim |
| GET /v1/inspection/corrections/{id} | 2 live-per-user | JWT | n/a | correction status |
| PUT /v1/inspection/objections/{id}/decide | 3 accredited | JWT+SpiceDB | yes | adjudicate objection |
| PUT /v1/inspection/corrections/{id}/apply | 3 accredited | JWT+SpiceDB | yes | apply correction to register |
| POST /v1/inspection/corrections/{id}/refer | 3 accredited | JWT+SpiceDB | yes | manual eCourt referral |

OpenAPI: packages/openapi-contracts/portal/voter-list-public-inspection.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.portal.objection_filed.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.portal.correction_applied.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.portal.auto_referral_sent.v1 | emits | eel.portal.inspection.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M04 VPRS (voter self-service link) | internal | crates/inspection/ | yes | advisory |
| M01 BVR device (biometric claimant verify) | internal | crates/inspection/ | yes | advisory; correction claim deferred if BVR unavailable |
| M10 rts-backend (snapshot source) | internal | crates/inspection/ | yes | advisory |
| eCourt (auto-referral) | ecourt | integrations/ecourt/ | yes | circuit breaker; referral queued; alert if queue grows |

## ML and AI

none

## Dependencies

- Modules: M04 (voter link), M01 (biometric verify), M10 (snapshot source)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-astro, eel-web-py

## Feature checklist

Roll snapshot generation

1. FR-M64-001: Generate roll snapshot for each polling station containing voter sequence number, full name, station code, ward, constituency, and county at T-90 window open. (FR-64.01)
2. FR-M64-002: HSM-sign each station roll; publish SHA-256 digest, generation timestamp, signer key fingerprint, and record count alongside the roll. (FR-64.01, FR-64.09)
3. FR-M64-003: Make signed station rolls available for download within 24 hours of T-90 trigger. (FR-64.01)
4. FR-M64-004: Suppress ID number, date of birth, and biometric reference from all public roll outputs per DPA 2019 §26. (FR-64.01, NFR-64.03)
5. FR-M64-005: Publish roll version number and updated SHA-256 to M11 public portal and IAB domain mirror on every version change. (FR-64.09)
6. FR-M64-006: Trigger P0 alert within 30 seconds when independently computed hash mismatches published SHA-256. (FR-64.09)
7. FR-M64-007: Station roll files served from CDN; origin serves only the signed manifest; CDN cache invalidated within 5 minutes of new version. (NFR-64.05, EC-SCL-01)
8. FR-M64-008: ETag-based 304 responses on snapshot metadata endpoint. (EC-SCL-02)

Public voter self-check

9. FR-M64-009: Unauthenticated voter self-check by name and station returns confirmation of registration, station name, and ward; no ID number in response. (FR-64.02)
10. FR-M64-010: Authenticated self-check tier accepts NIIMS reference for exact-match lookup. (FR-64.02)
11. FR-M64-011: Constant-time response shape for found/not-found to prevent voter enumeration; p95 delta under 5 ms. (EC-SEC-10)
12. FR-M64-012: Rate-limit unauthenticated search to 10 requests per minute per IP and per /64 prefix. (NFR-64.04)
13. FR-M64-013: CAPTCHA step-up triggered at bot-pattern threshold; block after threshold breach. (NFR-64.04)
14. FR-M64-014: USSD self-check fallback path in at most 5 numeric steps for low-literacy users. (EC-HUM-10)
15. FR-M64-015: Roll download endpoint limited to 5 downloads per hour per IP. (NFR-64.04)

Objection intake

16. FR-M64-016: Accept objection from any registered voter within T-90 to T-60 window; verify objector via M04 VPRS API. (FR-64.03)
17. FR-M64-017: Validate objection grounds: not-citizen, under-age, deceased, duplicate, false-address, or other. (FR-64.03)
18. FR-M64-018: Accept document upload as supporting evidence (PDF, max 5 MB per document). (FR-64.03)
19. FR-M64-019: Assign UUID7 case number on objection submission; display on confirmation screen. (FR-64.03)
20. FR-M64-020: Send acknowledgement SMS and email within 5 minutes of objection filing. (FR-64.03, NFR-64.06)
21. FR-M64-021: Block new objection filings after T-60 23:59 EAT hard close; OPA policy enforced at transaction time. (FR-64.06, EC-TIM-09)

Correction claims

22. FR-M64-022: Accept correction claim for own record (name, address, station transfer) with biometric verification via M01 or M13. (FR-64.04)
23. FR-M64-023: Route correction claims through a workflow distinct from objections. (FR-64.04)
24. FR-M64-024: Pre-fill current value from roll snapshot in correction claim form. (FR-64.04)
25. FR-M64-025: Reject correction claim if biometric verification is unavailable; defer and notify claimant. (FR-64.04)

Objection routing and determination

26. FR-M64-026: Route objection to Registration Officer assigned to the subject voter's station. (FR-64.05)
27. FR-M64-027: Allow officer to request additional evidence with a 72-hour claimant response window. (FR-64.05)
28. FR-M64-028: Record determination (upheld, dismissed, or referred-to-court) with reason text and officer UUID. (FR-64.05)
29. FR-M64-029: CRO counter-signs batch determinations for assigned county; dual-officer approval required before any roll commit. (FR-64.05, FR-64.07)
30. FR-M64-030: SpiceDB RBAC: officer write scope limited to assigned stations; CRO scope limited to own county. (FR-64.05)

Deadline enforcement and auto-referral

31. FR-M64-031: Auto-refer any objection not determined by T-60 23:59 EAT to Elections Court via structured referral packet. (FR-64.06)
32. FR-M64-032: Idempotent auto-referral scheduler; double-fire at T-60 produces exactly one eCourt packet. (EC-LEG-09)
33. FR-M64-033: Notify objector, subject voter's CRO, and IEBC Legal on auto-referral. (FR-64.06)
34. FR-M64-034: Deliver auto-referral packet to Elections Court within 1 hour of T-60 breach; circuit breaker queues if eCourt unavailable. (NFR-64.06)

Roll update after determination

35. FR-M64-035: Upheld objections propose deletion or correction in M04 VPRS staging table; no direct write to live roll. (FR-64.07)
36. FR-M64-036: Generate and publish new signed roll version within 2 hours of committed dual-officer-approved change. (FR-64.07)

Compliance dashboard

37. FR-M64-037: Internal dashboard: days remaining in window, objection counts by county (filed, pending, determined, lapsed), correction claim counts, on-time determination rate. (FR-64.08)
38. FR-M64-038: Auto-alert to IEBC Chair when any county's lapsed-objection rate exceeds 5%. (FR-64.08)
39. FR-M64-039: IAB read access to compliance dashboard; daily lapsed-objection count per county emitted. (FR-64.08)

Security and data protection

40. FR-M64-040: Store objector and subject voter identifiers as hashed references in all log and event payloads; no plain ID in telemetry. (NFR-64.03, EC-SEC-16)
41. FR-M64-041: Parameterised queries on all search and intake endpoints; Semgrep CI gate enforces no string interpolation in SQL. (EC-SEC-05)
42. FR-M64-042: SpiceDB-down behaviour: fail closed on writes; class-C reads only; return 503 problem+json. (EC-IDN-10)
43. FR-M64-043: JWT silent refresh on expiry during in-flight request; next unauthenticated call returns 401. (EC-IDN-02)
44. FR-M64-044: Access logs retained 7 years per Elections Act §44. (NFR-64.03)

Accessibility and internationalisation

45. FR-M64-045: WCAG 2.2 AA on all voter-facing screens; screen-reader compatible markup on search and objection forms. (NFR-64.01)
46. FR-M64-046: English and Swahili throughout; support all 11 gazetted languages on public portal. (NFR-64.01, EC-HUM-08)
47. FR-M64-047: Mobile-first layout; functional on 1 GB RAM devices on 3G. (NFR-64.01, EC-CON-02)

## Acceptance criteria

1. AC-M64-01: Voter enumeration prevented: timing oracle test shows p95 delta under 5 ms between found and not-found responses on the self-check endpoint. (EC-SEC-10)
2. AC-M64-02: Auto-referral at T-60 idempotent: double-fire test produces exactly one eCourt packet; second call returns cached 200; no duplicate referral record. (EC-LEG-09)
3. AC-M64-03: Merkle root verifiable against snapshot content: inclusion proof accepted by SDK verifier for every record in the snapshot; property test.
4. AC-M64-04: Objection and correction filing blocked after T-60 23:59 EAT: OPA hard-close test at boundary plus and minus 1 second. (EC-TIM-09)
5. AC-M64-05: Voter ID hashes absent in plain from all logs and traces: log-scanner CI fixture returns zero matches on Tier A and B field patterns. (EC-SEC-16)
6. AC-M64-06: Biometric claimant verification required before correction claim progresses: integration test with M01 stub returns deferred status when BVR unavailable.
7. AC-M64-07: Public search portal p99 under 500 ms at 5 M requests per hour with at least 95% served from CDN edge; k6 load test. (EC-SCL-01, portal SLA)
8. AC-M64-08: Acknowledgement SMS and email sent within 5 minutes of objection filing: integration test with stub gateway; p99 measured.
9. AC-M64-09: Auto-referral packet delivered to Elections Court within 1 hour of T-60 breach: integration test with eCourt double.
10. AC-M64-10: New signed roll version published within 2 hours of committed dual-officer-approved change: integration test.
11. AC-M64-11: Commission Chair alert fires when any county lapsed-objection rate exceeds 5%: unit test with planted lapsed count.
12. AC-M64-12: SpiceDB down triggers fail-closed on writes and class-C reads only: PodChaos test; 503 problem+json returned to callers.
13. AC-M64-13: JWT expiry mid-request: in-flight objection filing completes; next call returns 401; integration test with 5-second TTL realm.
14. AC-M64-14: CDN cache invalidated within 5 minutes of roll version update: integration test verifies ETag changes before 5-minute mark.
15. AC-M64-15: WCAG 2.2 AA audit pass: axe-core CI run returns zero critical violations on search and objection intake forms.

## Applicable edge cases

Security

EC-SEC-05: SQL or NoSQL injection on name, station, or objection-grounds fields; parameterised queries enforced; Semgrep CI gate; ZAP scan on intake endpoints.
EC-SEC-10: Voter enumeration via self-check; constant-time response shape; step-up CAPTCHA on volume threshold; timing oracle p95 delta under 5 ms.
EC-SEC-12: Cache poisoning of roll snapshot CDN entries; cache key includes roll version and signature fingerprint; client verifies SHA-256 before use.
EC-SEC-15: Portal admin session hijack on objection adjudication step; CSRF token plus session binding; re-auth required on determination write.
EC-SEC-16: Voter and objector ID hashes not in plain in logs or traces; telemetry scrubber; log-scanner CI fixture enforces zero matches.

Identity

EC-IDN-02: JWT expires mid-objection-filing POST; in-flight request completes with original token; next call triggers silent refresh or returns 401.
EC-IDN-09: CRO counter-sign approver revoked between dual-control steps; both roles re-validated at final commit; partial approval rejected.
EC-IDN-10: SpiceDB down during officer determination write; fail closed; 503 problem+json returned; class-C reads served from replica.
EC-IDN-11: Realm key rotated while portal is live; JWKS refetched on kid-miss; no portal outage; token validation resumes automatically.

Data

EC-DAT-01: Duplicate objection submission with same idempotency key and body; cached response returned; no second record created; no re-execution.
EC-DAT-02: Same idempotency key with different objection body; 422 idempotency-key-reuse returned; original record unchanged.
EC-DAT-09: Unicode names in voter search and objection grounds; NFC normalisation applied; ICU collation; grapheme-safe display in all 11 locales.
EC-DAT-10: Zero objections filed in a county; valid result; lapsed-rate denominator guarded against division-by-zero in dashboard.
EC-DAT-13: Soft-deleted voter in snapshot search; tombstone honoured; record not returned as registered; purge job respects retention schedule.

Scale

EC-SCL-01: 5 M visitors per hour on public search portal; at least 95% served from CDN edge; origin p99 under 500 ms; k6 plus Gatling 5-region test.
EC-SCL-02: Thundering herd on snapshot STH endpoint at T-90 open; ETag 304 responses; SDK jitter on retries; hit ratio monitored.
EC-SCL-04: Consumer lag exceeds 60 seconds on portal inspection event topic; backpressure applied; page fired; no OOM on consumer pod.
EC-SCL-08: Valkey eviction removes objection idempotency key; PG-durable idempotency store falls back; no duplicate record created.

Human

EC-HUM-06: Screen reader on objection intake form; full task completion with zero axe critical violations; semantics tested manually.
EC-HUM-08: 11 languages and long objection ground strings on confirmation page; no truncation; ICU plurals; font fallback for all scripts.
EC-HUM-09: Colour-blind view of compliance dashboard heat map; symbol plus text plus pattern encoding; ΔE contrast registry checked.
EC-HUM-10: Low-literacy USSD self-check fallback; numeric menu in at most 5 steps; IVR alternative available.

Legal and time

EC-LEG-09: T-60 auto-referral scheduler double-fire; idempotent scheduler produces exactly one eCourt packet; duplicate suppressed.
EC-TIM-08: Inspection window petition close at T-60 23:59 EAT; boundary test at plus and minus 1 second; EAT UTC offset applied correctly.
EC-TIM-09: Register freeze race at T-60/T-30; OPA hard-close at transaction time; all-or-nothing; concurrent writers tested under load.

Operations

EC-OPS-08: Disk full on evidence-document attachment storage; 507 response to uploader; alert fired; existing snapshot files not corrupted.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Inspection window FSM, anti-enumeration guard, auto-referral idempotency | nextest | M64-UNIT- |
| property | Merkle root verification; objection idempotency; T-60 race | proptest | M64-PROP- |
| contract | eCourt referral double; M02 correction apply; M01 biometric verify | Pact | M64-PACT- |
| integration | Full inspection lifecycle; auto-referral; window close | nextest + testcontainers | M64-INT- |
| e2e | Public self-check; objection and correction filing; admin adjudicate | Playwright | M64-E2E- |
| load | 5M/hr public portal | k6 + Gatling | M64-LOAD- |
| security | Timing oracle ≤5 ms delta; ZAP; log scanner | ZAP + Semgrep | M64-SEC- |
| i18n | 11 locales | custom | M64-I18N- |
| a11y | axe on public surfaces | axe-core | M64-A11Y- |

Tags: `#[trace("FR-M64-008","AC-M64-02","EC-LEG-09")]`

## Runbooks and dashboards

- docs/runbooks/portal-svc-degradation.md (extend with inspection section)
- docs/runbooks/ecourt-referral-queue.md (new; trigger when queue exceeds threshold)
- platform/observability/portal-svc/inspection-dashboard.json
- platform/observability/portal-svc/slo-alerts.yaml

## Open questions

- eCourt API specification not yet available; double gates development.
- Whether biometric verification for correction claims requires the claimant to be physically present at an IEBC office or can be remote; affects M01 integration scope.
