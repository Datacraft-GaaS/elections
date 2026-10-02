# M42 media-accreditation-portal

| field | value |
|---|---|
| M-number | M42 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5%; 99.9% during accreditation window and election day |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M42-media-accreditation-portal.md |

## Purpose

Manage accreditation of journalists, photographers, broadcasters, and live-feed operators for election-period coverage. Issue tamper-evident digital credentials, control access to results centres and restricted polling areas, enforce Media Council Act 2013 code-of-conduct obligations, and provide a revocation workflow for accreditation breaches. Badge verification is offline-capable with a 5-minute CRL refresh cycle.

Strategy reference: §XVIII (Stakeholder Engagement). Elections Act s.21A; Media Council of Kenya Act 2013 s.29; the Commission Media Accreditation Regulations.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Media Applicant Portal (WCAG 2.2 AA) | Astro | services/training-svc/astro/media-accreditation/ |
| the Commission Media Officer Back-Office | Phoenix LiveView | services/training-svc/ex/accreditation_web/ |
| Scanner App (mobile QR scanner) | Flutter | services/training-svc/apps/badge-scanner/ |
| Public Organisation List | Astro | services/training-svc/astro/media-accreditation/ |
| API | Rust (axum) | services/training-svc/crates/training-api/ |

Languages: EN + SW; applicant portal WCAG 2.2 AA.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| MediaOrganisation | id, name, organisation_type (PRINT/BROADCAST/DIGITAL/AGENCY), mck_registered (bool), live_feed_permit (bool), created_at | B | T1-consensus |
| MediaAccreditation | id, applicant_name, id_number, organisation_id, mck_membership_number, photo_url, access_zones (list of POLLING_STATION/TALLYING_CENTRE/NATIONAL_RESULTS_CENTRE/RESTRICTED_BRIEFING), status (SUBMITTED/UNDER_VETTING/APPROVED/REJECTED/BADGE_ISSUED/ACTIVE/SUSPENDED/REVOKED), badge_serial (UUID7), badge_issued_at, valid_from, valid_to, revocation_reason, revoked_at | B | T1-consensus |
| BadgeRevocationEntry | badge_serial, reason_code (BREACH/MISCONDUCT/SECURITY/EXPIRED), revoked_at, officer_id | A | T1-audit |
| AccreditationAuditEntry | id, action, actor_id, accreditation_id, timestamp, ip, purpose_code, merkle_node_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/training/media-accreditation.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/accreditations | 2 | public | yes | submit accreditation application |
| GET /v1/accreditations/{id}/status | 2 | JWT+SpiceDB (APPLICANT) | n/a | get accreditation status |
| POST /v1/accreditations/{id}/revoke | 2 | JWT+SpiceDB (MEDIA_OFFICER/SECURITY_OFFICER) | yes | revoke; second-officer confirmation for non-emergency |
| POST /v1/accreditations/bulk | 2 | JWT+SpiceDB (ORG_ADMIN) | yes | bulk crew roster CSV/XLSX upload |
| GET /v1/badges/{serial}/verify | 1 | public | n/a | badge verification; offline-capable with CRL cache; <= 300 ms |
| GET /v1/crl | 1 | public | n/a | current revocation list; refreshed every 5 minutes |
| POST /v1/live-feed-permits | 2 | JWT+SpiceDB (MEDIA_OFFICER) | yes | issue live-feed access permit linked to accreditation |
| GET /v1/public/organisations | 1 | public | n/a | accredited organisation names; no individual names |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.media_badge_issued.v1 | emits | eel.training.accreditation.v1 | operational | yes | at-least-once idempotent |
| eel.training.media_badge_revoked.v1 | emits | eel.training.accreditation.v1 | audit | yes | exactly-once |
| eel.training.media_breach_referred.v1 | emits | eel.training.accreditation.v1 | audit | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| MCK API | external | integrations/mck/ | yes (stub) | membership verification; breach referral; fallback to manual certificate upload if unavailable |
| NPS API | external | integrations/nps/ | yes (stub) | advisory clearance check; advisory only; not stored post-vetting session |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | live-feed permit network-access list push |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | audit log anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | badge digital signature |
| M43 Observer Accreditation | internal | integrations/observer-accreditation/ | yes | shared CRL infrastructure and scanner app |

## ML and AI

- None.

## Dependencies

- Modules: M16, M43
- Platform: P05, P06
- Packages: kit-sdk-flutter, eel-resilience-rs, eel-crypto-rs

## Feature checklist

Application:
1. FR-M42-001: online application: full name, NIN/Passport, employer organisation, MCK membership number, coverage scope (national/county/constituency), access zones requested (POLLING_STATION/TALLYING_CENTRE/NATIONAL_RESULTS_CENTRE/RESTRICTED_BRIEFING), ICAO-compliant photo
2. FR-M42-002: MCK membership verification via MCK API on submission; non-MCK applicants flagged for enhanced vetting; mck_verified boolean stored on record
3. FR-M42-003: background vetting checklist: NPS advisory clearance check (advisory only), the Commission internal watch-list cross-reference, prior accreditation revocation history; results visible only to the Commission Media Officer
4. FR-M42-004: application lifecycle: SUBMITTED -> UNDER_VETTING -> APPROVED/REJECTED -> BADGE_ISSUED -> ACTIVE/SUSPENDED/REVOKED; state transitions role-gated and immutable once made
5. FR-M42-005: rejection notice: written reason citing specific regulatory clause (Elections Act s.21A, Media Council Act 2013 s.29, or the Commission Media Accreditation Regulations); delivered to applicant within 48 hours of rejection
6. FR-M42-006: bulk organisation onboarding: broadcaster submits crew roster CSV/XLSX; system generates individual applications per crew member; organisation admin dashboard tracks per-person status
7. FR-M42-007: bulk roster per-row error report: invalid rows listed with field-level error codes; valid rows queued independently; valid rows not blocked by invalid peer rows
8. FR-M42-008: code-of-conduct digital acknowledgement before badge issuance; references Media Council Act 2013 s.29; timestamped and stored as AccreditationAuditEntry

Badge:
9. FR-M42-009: digital badge PDF: holder name, photo, organisation, access zones list, validity dates (valid_from, valid_to), badge serial (UUID7); QR encodes HSM-signed JWT claim; rendered using badge template service
10. FR-M42-010: badge HSM-signed via P06; signature verifiable offline using the Commission public key embedded in scanner app; badge issuance pauses if HSM unavailable and queues for signing on restore
11. FR-M42-011: live-feed access permit: separate permit layer for broadcast crews needing fibre/VSAT access; permit linked to accreditation record; auto-populates M16 Election Command Centre network-access list via outbound integration
12. FR-M42-012: accreditation window enforcement: portal accepts applications only within gazetted window; window dates configurable per gazette notice; late applications return 422 with gazette reference and statutory citation

Scanning and access:
13. FR-M42-013: QR scanner app (Flutter, shared with M43): full-screen QR scan; immediate colour-coded result (green ACTIVE / red REVOKED / amber SUSPENDED); access zone list; offline CRL cache; presence counter per results centre updated in real time
14. FR-M42-014: scanner CRL sync at each scan session start; badge verification response <= 300 ms including CRL lookup; scanner alert fires if local CRL age > 30 minutes
15. FR-M42-015: badge serial verification endpoint returns only pass/fail with zone info; no applicant name, photo, or personal data in public verification response

Revocation:
16. FR-M42-016: revocation workflow: the Commission Media Officer or Security Officer initiates with reason code (BREACH/MISCONDUCT/SECURITY/EXPIRED); second-officer confirmation required for non-emergency SECURITY and MISCONDUCT revocations; emergency SECURITY bypass logged with justification
17. FR-M42-017: badge serial added to CRL within 5 minutes of revocation confirmed; CRL published to CDN; offline scanners sync CRL at next scan session start
18. FR-M42-018: BREACH revocation triggers MCK referral package: signed PDF with badge serial, timestamp, breach description, officer details; transmitted via MCK API within 1 hour; fallback to encrypted email if MCK API unavailable
19. FR-M42-019: CRL HSM-signed by P06; scanner app rejects any CRL whose signature fails verification; CRL schema includes generated_at timestamp and revoked_serials array

Audit and public:
20. FR-M42-020: every accreditation action audit-logged with actor, timestamp, IP, and purpose code; each entry appended to P05 Merkle chain; immutable
21. FR-M42-021: RBAC: Media Officer cannot self-approve application from their own organisation; Security Officer revocation requires second-officer confirmation for non-emergency events; conflict-of-interest declaration tracked per vetting decision
22. FR-M42-022: public accredited-organisation list: organisation names only; no individual journalist names in any response field; updated daily during active window; downloadable CSV
23. FR-M42-023: real-time presence count per results centre visible to Security Officers and Results Centre Managers on the Commission back-office; presence count derived from scanner app events

Data and compliance:
24. FR-M42-024: DPA 2019 s.30(1)(b) lawful basis; biometric photo retained 7 years; NPS advisory data not stored beyond vetting session; only pass/fail advisory flag stored; DPIA registered with ODPC
25. FR-M42-025: applicant status notifications via email and SMS at each lifecycle stage (APPROVED, REJECTED, BADGE_ISSUED, SUSPENDED, REVOKED); notification delivery logged
26. FR-M42-026: application portal p99 <= 1 s under 2,000 concurrent users; auto-scaling portal; queue-based processing for bulk upload path; burst handled without dropped requests
27. FR-M42-027: system availability >= 99.9% during accreditation window and election day; >= 99.5% at other times; SLO alert fires within 5 minutes of breach
28. FR-M42-028: WCAG 2.2 AA on all applicant-facing screens and public organisation list; EN + SW language selector; verified by axe-core and manual audit
29. FR-M42-029: NPS advisory data not stored post-vetting session; advisory pass/fail flag cleared on session close; no NPS fields in persisted MediaAccreditation record
30. FR-M42-030: MCK API unavailability fallback: applicant can upload MCK membership certificate image; officer manually verifies; mck_verified flag updated when MCK API restores; manual path documented in vetting checklist
31. FR-M42-031: accreditation window dates configurable by the Commission Media Officer per gazette notice; gazette reference stored per application batch; window closure enforced via durable date gate

## Acceptance criteria

1. AC-M42-01: journalist submits online application; MCK API verifies membership within 60 seconds; vetting completes within 48 hours; badge PDF issued; QR verifies at entry point within 300 ms; verified by UAT
2. AC-M42-02: badge revoked; badge serial added to CRL within 5 minutes of revocation; offline scanner reflects revocation at next CRL sync; integration test confirms CRL timestamp and serial presence
3. AC-M42-03: BREACH revocation triggers MCK referral package transmitted via MCK API within 1 hour of revocation; MCK API stub confirms receipt; package includes badge serial, timestamp, and officer details; verified by integration test
4. AC-M42-04: bulk crew roster CSV with 500 rows processed; per-row error report generated for invalid rows; valid rows issued badges; verified by load test confirming no request failure
5. AC-M42-05: public organisation list returns organisation names only; zero individual journalist names or NIN fields appear in any response field or CSV export; verified by security test
6. AC-M42-06: application portal p99 <= 1 s under 2,000 concurrent users during accreditation window open surge; verified by k6 load test
7. AC-M42-07: badge QR verification p99 <= 300 ms including CRL lookup under normal scan load; verified by load test
8. AC-M42-08: CRL propagation from revocation event to CDN update <= 5 minutes; verified by integration test comparing revocation timestamp to CRL generated_at on CDN
9. AC-M42-09: scanner app CRL-cache-age alert fires when local CRL is > 30 minutes old; verified by unit test with mocked CRL timestamp
10. AC-M42-10: system availability >= 99.9% during 60-day accreditation window and election day; SLO breach triggers alert within 5 minutes; verified by SLO dashboard report
11. AC-M42-11: WCAG 2.2 AA on applicant portal and public organisation list; zero critical axe-core violations; EN + SW verified by language-switch smoke test
12. AC-M42-12: NPS advisory data not stored beyond vetting session; data-purge integration test confirms no NPS fields present in persisted MediaAccreditation records after session close

## Applicable edge cases

EC-DAT-01: duplicate application submission with same idempotency key and body; cached response returned; application not double-stored; MediaAccreditation record count unchanged
EC-DAT-02: duplicate submission with same idempotency key but different body; 422 `idempotency-key-reuse` returned; no partial write committed
EC-DAT-07: bulk roster partial failure; per-row error report generated for invalid rows; valid rows processed and queued for badge issuance independently
EC-DAT-09: Unicode applicant names (e.g., Arabic, Amharic script for international crews); NFC normalisation applied; grapheme-safe rendering on badge PDF and scanner display
EC-DAT-06: badge schema v1/v2 coexist during rolling deployment; both accepted by scanner app; dual-emit on badge JWT; Karapace compat check passes
EC-SEC-01: CRL tampering attempt; CRL HSM-signed by P06; scanner app rejects payload whose signature fails verification; tamper event logged
EC-SEC-03: HSM unavailable during badge signing; badge issuance pauses; applications continue to be processed to APPROVED state; badge signing queue drains on HSM restore; no data loss
EC-SEC-05: injection on all input fields (name, employer, zones, rejection reason); parameterised queries only; verified by Semgrep, sqlx compile-time checks, and ZAP
EC-SEC-08: badge serial enumeration attempt on verification endpoint; endpoint returns only pass/fail and zone info; no applicant personal data in response body
EC-SEC-09: rate-limit evasion on badge verification or application submission; keyed by subject/device/IP-slash-64; priority shedding applied; 429 returned with Retry-After header
EC-SEC-16: secrets in logs/traces; NPS advisory flag and applicant PII excluded from telemetry; log-scanner fixture in CI confirms no PII in OTEL spans
EC-IDN-02: JWT expires mid-request during vetting action; in-flight action completes; next request returns 401 with silent refresh prompt
EC-IDN-09: dual-control revocation; second-officer certificate revoked between initiating and committing revocation; both certificates re-validated at commit; revocation rejected if either invalid
EC-IDN-10: SpiceDB down during revocation attempt; fail-closed; 503 problem+json returned; revocation not committed until SpiceDB restored
EC-HUM-06: screen readers on applicant portal and public organisation list; full task completion at WCAG 2.2 AA; verified by axe-core and manual audit scripts
EC-HUM-08: EN + SW language toggle; all badge field labels render without truncation; ICU plurals on roster summary counts
EC-HUM-09: colour-blind users on scanner app result screen; result encoded as colour + symbol (checkmark/cross/warning triangle) + text; ΔE registry check passes
EC-SCL-02: surge applications at accreditation window open; auto-scaling portal handles burst; queue-based processing absorbs peak; bulk upload path reduces per-individual load
EC-SCL-04: consumer lag > 60 seconds on badge-issuance or CRL-publish queue; alert fires; backpressure applied; no OOM; CRL publish SLA re-evaluated
EC-CON-01: scanner app offline; local CRL cache used for verification; alert fires if CRL age > 30 minutes; manual block list distributed via SMS to supervisors
EC-OPS-05: MCK API unavailable during vetting; fallback to manual certificate upload activated; vetting continues without delay; mck_verified flag updated when MCK API restores
EC-OPS-07: the Commission Media Officer absent during second-officer confirmation step; confirmation re-assigned to next available officer; credential revocation exactly-once on commit
EC-LEG-02: DPA 2019 s.30(1)(b) lawful basis; NPS advisory data not stored beyond vetting session; biometric photo retained 7 years; DPIA registered; ODPC notified of any breach within 72 hours
EC-LEG-04: revocation disputed as arbitrary; compensating event records dispute; second-officer confirmation log produced as evidence; reason codes tied to specific regulation clauses
EC-SEC-17: PQ hybrid negotiation failure on scanner-to-CRL CDN endpoint; scanner falls back to classical TLS 1.3 only; never negotiates TLS 1.2; verified by testssl.sh

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | badge signature generation, CRL signing, revocation reason-code validation, window enforcement | nextest + flutter_test | M42-UNIT- |
| contract | MCK API membership check, MCK breach referral, M16 network-access push, P06 signing | Pact | M42-PACT- |
| integration | application to badge issuance; revocation to CRL update; MCK breach referral | integration_test | M42-INT- |
| e2e | journalist submits, vetting completes, badge issued, scanned at entry, revoked, CRL updated | Flutter driver | M42-E2E- |
| security | badge serial enumeration; CRL tampering; revocation bypass; self-approval by Media Officer | ZAP + red team | M42-SEC- |
| a11y | WCAG 2.2 AA on applicant portal and public list | axe-core + manual | M42-A11Y- |
| offline | scanner CRL cache; deliberate network drop; CRL age alert at > 30 min | netem | M42-OFFLN- |
| load | 2,000 concurrent applicants at window open | k6 | M42-LOAD- |

Tags: `#[trace("FR-M42-010","AC-M42-02","EC-CON-01")]`

## Runbooks and dashboards

- `docs/runbooks/training-svc-degradation.md` - extend with badge scanner CRL sync failure paths and MCK API fallback
- `docs/runbooks/emergency-revocation.md` - new runbook; Security Officer emergency revocation; CRL propagation confirmation; war-room escalation
- Grafana dashboard: `eel-iems-42` (alert ruleset: `iems-42-alerts`)
- SLO alerts: CRL propagation <= 5 min; badge verification p99 <= 300 ms; system availability >= 99.9% during accreditation window

## Open questions

- MCK API MoU: must be signed by T-270; sandbox access required for development; if delayed, manual certificate upload fallback must be production-ready from day one.
- Scanner app sharing with M43: the brief indicates shared CRL infrastructure and scanner app between M42 and M43; the Flutter app should be a shared surface built once and configured per accreditation type; confirm with M43 team before sprint planning.
