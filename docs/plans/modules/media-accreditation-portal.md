# M42 media-accreditation-portal

| field | value |
|---|---|
| M-number | M42 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5%; 99.9% during accreditation window and election day |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M42-media-accreditation-portal.md |

## Purpose

Manage accreditation of journalists, photographers, broadcasters, and live-feed operators for election-period coverage. Issue tamper-evident digital credentials, control access to results centres and restricted polling areas, enforce Media Council Act 2013 code-of-conduct obligations, and provide a revocation workflow for accreditation breaches. Badge verification is offline-capable with a 5-minute CRL refresh cycle.

Strategy reference: §XVIII (Stakeholder Engagement). Elections Act s.21A; Media Council of Kenya Act 2013 s.29; IEBC Media Accreditation Regulations.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Media Applicant Portal (WCAG 2.2 AA) | Astro | services/training-svc/astro/media-accreditation/ |
| IEBC Media Officer Back-Office | Phoenix LiveView | services/training-svc/ex/accreditation_web/ |
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

1. FR-M42-001: online application with NIN/Passport, employer, MCK membership number, coverage scope, access zones requested, ICAO-compliant photo
2. FR-M42-002: MCK membership verification via MCK API; non-MCK applicants flagged for enhanced vetting
3. FR-M42-003: background vetting checklist: NPS advisory, IEBC internal watch-list, prior revocation history
4. FR-M42-004: application lifecycle: SUBMITTED -> UNDER_VETTING -> APPROVED/REJECTED -> BADGE_ISSUED -> ACTIVE/SUSPENDED/REVOKED
5. FR-M42-005: bulk organisation onboarding via crew roster CSV/XLSX; organisation admin tracks per-person status
6. FR-M42-006: digital badge: QR-coded PDF; holder name, photo, organisation, access zones, validity, badge serial (UUID7); HSM-signed; offline-verifiable via embedded signature
7. FR-M42-007: live-feed access permit linked to accreditation record; permit auto-populates M16 network-access list
8. FR-M42-008: code-of-conduct digital acknowledgement before badge issuance; timestamped and stored
9. FR-M42-009: badge scanning at entry points via QR scanner app; confirms accreditation, zones, and presence count per results centre in real time
10. FR-M42-010: revocation workflow: IEBC Media Officer or Security Officer initiates with reason code; badge serial added to CRL published every 5 minutes; offline scanners sync CRL at each scan session
11. FR-M42-011: BREACH revocation generates MCK referral package (signed PDF with badge serial, timestamp, breach description, officer details); transmitted via MCK API within 1 hour
12. FR-M42-012: accreditation window enforcement per gazette notice; late applications rejected with statutory reference
13. FR-M42-013: public accredited-organisation list (organisation names only, no individual journalist names); updated daily during active window
14. FR-M42-014: all accreditation actions audit-logged with actor, timestamp, IP, purpose code; appended to P05 Merkle chain
15. RBAC: Media Officer cannot self-approve own organisation; Security Officer revocation requires second-officer confirmation for non-emergency events

## Acceptance criteria

1. AC-M42-01: journalist submits application; MCK verified; badge PDF issued; QR verifies at entry point within 300 ms; UAT
2. AC-M42-02: badge revoked; CRL updated within 5 minutes; offline scanner reflects revocation at next sync; integration test
3. AC-M42-03: BREACH revocation triggers MCK referral within 1 hour; MCK API stub test
4. AC-M42-04: bulk roster CSV (500 rows) processed; per-row error report generated; load test
5. AC-M42-05: public organisation list shows org names, no individual journalist names; security test

## Applicable edge cases

**connectivity**
- EC-CON-01: scanner offline; local CRL cache used; alert if CRL age > 30 minutes; manual block list distributed via SMS to supervisors

**security**
- EC-SEC-08: badge serial enumeration attempt; verification endpoint returns only pass/fail with zone info; no personal data in response
- EC-SEC-01: CRL tampering attempt; CRL signed by P06 HSM key; scanner rejects tampered CRL

**operations**
- EC-OPS-05: MCK API unavailable during vetting; fallback to manual certificate upload; vetting continues; MCK status flag updated when API restores

**scale**
- EC-SCL-02: surge applications at window open; auto-scaling portal; queue-based processing; bulk upload path reduces individual load

**legal**
- EC-LEG-02: DPA 2019 s.30(1)(b) lawful basis; NPS advisory data not stored beyond vetting session; biometric photo retained 7 years; DPIA registered

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
