# M43 observer-accreditation-portal

| field | value |
|---|---|
| M-number | M43 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5%; 99.9% during accreditation window and election day |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M43-observer-accreditation-portal.md |

## Purpose

Upstream accreditation of domestic and international election observers. Distinct from M15 (IAB Dashboard, ongoing audit role) and M34 (Observer App, field data collection). This module gates entry: an observer organisation applies, is vetted, and receives credentials before any field activity. Covers domestic NGOs (KEOG, ELOG), international missions (EU EOM, AU, Carter Center, IFES, Commonwealth), and individual observers under accredited organisations.

Strategy reference: §XVIII (Stakeholder Engagement). Elections Act s.21A; IEBC Observer Accreditation Regulations 2022.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Observer Organisation Portal (WCAG 2.2 AA) | Astro | services/training-svc/astro/observer-accreditation/ |
| Individual Observer Self-Application (WCAG 2.2 AA) | Astro | services/training-svc/astro/observer-accreditation/ |
| IEBC Vetting Back-Office | Phoenix LiveView | services/training-svc/ex/accreditation_web/ |
| Deployment Board | Phoenix LiveView | services/training-svc/ex/accreditation_web/ |
| Scanner App (shared with M42) | Flutter | services/training-svc/apps/badge-scanner/ |
| Public Observer Summary | Astro | services/training-svc/astro/observer-accreditation/ |
| API | Rust (axum) | services/training-svc/crates/training-api/ |

Languages: EN + SW; French and Arabic display options for international observers (records in EN).

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ObserverOrganisation | id, name, org_type (DOMESTIC_NGO/INTERNATIONAL_MISSION/DIPLOMATIC_MISSION/RELIGIOUS_BODY/ACADEMIC), jurisdiction, mou_required (bool), mou_signed_at, approved_at, observer_quota, created_at | B | T1-consensus |
| IndividualObserver | id, organisation_id, full_name, id_number, nationality, photo_url, deployment_county_codes, assigned_constituency_codes, status (SUBMITTED/DOCS_VERIFIED/BACKGROUND_CHECK/APPROVED/REJECTED/CREDENTIAL_ISSUED/DEPLOYED/SUSPENDED/REVOKED), credential_serial (UUID7), credential_issued_at, valid_from, valid_to, revocation_reason, created_at, updated_at | B | T1-consensus |
| CredentialRevocationEntry | credential_serial, reason, revoked_at, officer_id | A | T1-audit |
| AccreditationAuditEntry | id, action, actor_id, entity_id, timestamp, ip, purpose_code, merkle_node_id | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/training/observer-accreditation.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/organisations | 2 | public | yes | submit organisation accreditation |
| GET /v1/organisations/{id}/status | 2 | JWT+SpiceDB (ORG_ADMIN) | n/a | organisation application status |
| POST /v1/organisations/{org_id}/observers | 2 | JWT+SpiceDB (ORG_ADMIN) | yes | submit individual observer application |
| POST /v1/organisations/{org_id}/observers/bulk | 3 | JWT+SpiceDB (ORG_ADMIN) | yes | bulk roster CSV/XLSX (up to 5,000 rows) |
| POST /v1/observers/{id}/deploy | 2 | JWT+SpiceDB (FIELD_COORDINATOR) | yes | assign observer to constituencies |
| GET /v1/credentials/{serial}/verify | 1 | public | n/a | credential verification; offline-capable; <= 300 ms |
| POST /v1/observers/{id}/revoke | 2 | JWT+SpiceDB (ACCREDITATION_OFFICER) | yes | revoke individual credential; CRL updated within 5 min |
| GET /v1/crl | 1 | public | n/a | revocation list; refreshed every 5 minutes; shared CDN with M42 |
| POST /v1/mou/{org_id}/sign | 2 | JWT+SpiceDB (COMMISSION_LEGAL) + HSM | yes | Legal Director signs MOU for international mission |
| GET /v1/public/summary | 1 | public | n/a | observer counts per organisation and county; no individual names without consent |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.observer_credential_issued.v1 | emits | eel.training.accreditation.v1 | operational | yes | at-least-once idempotent |
| eel.training.observer_credential_revoked.v1 | emits | eel.training.accreditation.v1 | audit | yes | exactly-once |
| eel.training.observer_deployed.v1 | emits | eel.training.accreditation.v1 | operational | yes | at-least-once idempotent |
| eel.training.mou_signed.v1 | emits | eel.training.accreditation.v1 | audit | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| NPS API | external | integrations/nps/ | yes (stub) | advisory background check; advisory only; not stored post-vetting |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | publish accredited-observer list for parallel reporting |
| M34 Observer App | internal | integrations/observer-app/ | yes | push deployment assignments and credential validity; circuit breaker |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | mutation audit log anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | credential signing and MOU signing |
| M42 Media Accreditation | internal | integrations/media-accreditation/ | yes | shared CRL infrastructure and scanner app |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | field presence dashboard; observer count per location |

## ML and AI

- None.

## Dependencies

- Modules: M15, M16, M34, M42
- Platform: P05, P06
- Packages: kit-sdk-flutter, eel-resilience-rs, eel-crypto-rs

## Feature checklist

Organisation application:
1. FR-M43-001: organisation application: legal name, jurisdiction, registration documents, mandate letter, contact focal point, total observer count, geographic deployment plan, prior Kenya election history; all documents stored in MinIO with reference FK
2. FR-M43-002: organisation type classification (DOMESTIC_NGO/INTERNATIONAL_MISSION/DIPLOMATIC_MISSION/RELIGIOUS_BODY/ACADEMIC); classification drives vetting depth and MOU requirement; immutable once approved
3. FR-M43-003: MOU workflow for INTERNATIONAL_MISSION organisations: draft MOU generated from approved template; IEBC Director of Legal signs via P06 HSM; counterpart uploads signed copy; status advances to MOU_SIGNED; individual observer applications blocked until MOU_SIGNED
4. FR-M43-004: conditional organisation approval path: DOMESTIC_NGO organisations approved conditionally pending full document verification; individual applications queued and activated on final approval
5. FR-M43-005: pre-loaded profiles for known missions (KEOG, ELOG, EU EOM, AU, Carter Center, IFES, Commonwealth); organisation admin claims profile with document verification; reduces onboarding time for returning organisations
6. FR-M43-006: accreditation window enforcement per IEBC gazette notice; organisation applications accepted only within gazetted window; late applications return 422 with gazette reference; window dates configurable per gazette notice

Individual observer application:
7. FR-M43-007: individual observer application under approved organisation: biographic details, NIN/Passport, ICAO photo, language competencies, prior observer experience, deployment preference (region/county)
8. FR-M43-008: background check: NPS advisory clearance, IEBC internal watch-list cross-reference, prior accreditation revocation history, criminal record self-declaration; advisory results visible only to IEBC Accreditation Officer; NPS data not stored post-vetting
9. FR-M43-009: application lifecycle: SUBMITTED -> DOCS_VERIFIED -> BACKGROUND_CHECK -> APPROVED/REJECTED -> CREDENTIAL_ISSUED -> DEPLOYED/SUSPENDED/REVOKED; state transitions role-gated and immutable once made
10. FR-M43-010: rejection notice: written reason citing specific statutory provision (Elections Act s.21A, IEBC Observer Accreditation Regulations 2022); delivered within 48 hours of rejection
11. FR-M43-011: bulk roster import: CSV/XLSX up to 5,000 rows; per-row error report with field-level codes; valid rows processed independently; total processing completes within 10 minutes
12. FR-M43-012: code-of-conduct digital acceptance before credential issuance; references IEBC Observer Accreditation Regulations 2022; timestamped and stored as AccreditationAuditEntry appended to P05 Merkle chain

Credential:
13. FR-M43-013: digitally signed PDF credential: observer name, photo, organisation, deployment county, validity window (valid_from, valid_to), credential serial (UUID7); signed via P06 HSM; QR verifiable offline using IEBC public key embedded in scanner app
14. FR-M43-014: credential QR verification response <= 300 ms; offline-capable with local CRL cache; scanner alert fires if local CRL age > 30 minutes
15. FR-M43-015: credential issuance pauses if P06 HSM unavailable; applications continue to APPROVED state; signing queue drains on HSM restore; no credential lost

Deployment:
16. FR-M43-016: IEBC Coordinator assigns approved observers to specific constituencies via batch assignment CSV; observer receives PDF deployment letter with polling-station list auto-generated
17. FR-M43-017: deployment data pushed to M34 Observer App within 5 minutes of assignment; observer sees assigned polling stations in app; circuit breaker handles M34 unavailability with queue-and-retry
18. FR-M43-018: per-station observer quota enforcement per Elections Act s.21A; system prevents assignment beyond quota; over-quota assignment returns 422 with alternate constituency suggestion
19. FR-M43-019: geographic coverage gap report on Deployment Board: constituencies with zero assigned observers flagged; coverage target >= 85% constituencies with at least one observer shown as dashboard metric
20. FR-M43-020: pre-election coverage report for IEBC Commissioner: accreditation rates per organisation and geographic coverage gaps; generated on demand and on schedule at T-7 and T-1

Revocation:
21. FR-M43-021: revocation workflow: IEBC Accreditation Officer initiates with reason; credential serial added to CRL within 5 minutes; field coordinator notified; M34 app credential for that observer invalidated
22. FR-M43-022: CRL published to CDN every 5 minutes; shared CDN with M42; scanner app (shared with M42) syncs at session start; CRL HSM-signed by P06; scanner rejects tampered CRL

Audit and public:
23. FR-M43-023: all mutations audit-logged with actor, timestamp, IP, and purpose code; appended to P05 Merkle chain; immutable
24. FR-M43-024: RBAC: Accreditation Officer cannot approve application where conflict-of-interest declared; dual-control for revocation required; SpiceDB enforces all role gates
25. FR-M43-025: public summary: observer counts per organisation and county; no individual observer names without consent; updated daily during active accreditation window; downloadable CSV
26. FR-M43-026: MOU audit trail: IEBC Legal Director HSM signature recorded; counterpart upload timestamped; both stored as AccreditationAuditEntry appended to P05 Merkle chain; linked to OrganisationRecord

Data and compliance:
27. FR-M43-027: DPA 2019 s.30(1)(b) lawful basis; biometric photo retained 7 years; NPS advisory data not stored post-vetting; DPIA registered with ODPC; data class B for organisation and individual records
28. FR-M43-028: WCAG 2.2 AA on organisation portal, individual self-application, and public summary; EN + SW baseline; French and Arabic display options for international observers (records in EN)
29. FR-M43-029: portal p99 <= 1 s under 5,000 concurrent users; auto-scaling; queue-based bulk import processing; burst handled without dropped requests
30. FR-M43-030: system availability >= 99.9% during accreditation window and election day; >= 99.5% at other times; SLO alert fires within 5 minutes of breach
31. FR-M43-031: observer applicant status notifications via email at each lifecycle stage; deployment letter auto-generated as PDF on constituency assignment; delivery logged

## Acceptance criteria

1. AC-M43-01: ELOG submits organisation application; approved within 5 working days; individual observer credential issued within 48 hours of approval; verified by UAT
2. AC-M43-02: EU EOM organisation approved via MOU workflow; IEBC Legal Director HSM signature applied; counterpart upload advances status to MOU_SIGNED; individual observer applications unblocked; verified by UAT
3. AC-M43-03: deployment assignment pushed to M34 Observer App; observer sees assigned polling stations in app within 5 minutes of assignment; circuit breaker test confirms retry on M34 unavailability; verified by integration test
4. AC-M43-04: observer credential revoked; credential serial added to CRL within 5 minutes; M34 app credential invalidated; offline scanner reflects revocation at next CRL sync; verified by integration test
5. AC-M43-05: 3,000-row bulk roster processed within 10 minutes; per-row error report generated for invalid rows; valid rows receive credentials; verified by load test
6. AC-M43-06: per-station observer quota enforced per Elections Act s.21A; over-quota assignment returns 422 with alternate constituency suggestion; verified by unit test covering boundary conditions
7. AC-M43-07: credential QR verification p99 <= 300 ms including CRL lookup under normal scan load; verified by load test
8. AC-M43-08: CRL propagation from revocation event to CDN update <= 5 minutes; verified by integration test comparing revocation timestamp to CRL generated_at on CDN
9. AC-M43-09: geographic coverage gap report shows constituencies with zero observers flagged; >= 85% coverage target metric visible on Deployment Board; verified by UAT with IEBC Field Deployment Coordinator
10. AC-M43-10: system availability >= 99.9% during accreditation window and election day; SLO breach triggers alert within 5 minutes; verified by SLO dashboard report
11. AC-M43-11: WCAG 2.2 AA on organisation portal, individual self-application, and public summary; EN + SW + French + Arabic display options verified by axe-core and manual audit; zero critical violations
12. AC-M43-12: NPS advisory data not stored beyond vetting session; data-purge integration test confirms no NPS fields in persisted IndividualObserver records after session close

## Applicable edge cases

EC-DAT-01: duplicate organisation application with same idempotency key and body; cached response returned; ObserverOrganisation record not double-stored
EC-DAT-02: duplicate submission with same idempotency key but different body; 422 `idempotency-key-reuse` returned; no partial write committed
EC-DAT-07: bulk roster partial failure; per-row error report generated for invalid rows; valid rows queued and processed independently; processing completes within 10-minute SLA
EC-DAT-09: Unicode observer names (Arabic, French, Amharic); NFC normalisation applied; grapheme-safe rendering on credential PDF and scanner display
EC-DAT-10: zero observers assigned to a constituency; valid state; no div-by-zero in coverage gap report; renders "0" on Deployment Board with flagged indicator
EC-SEC-01: credential serial enumeration attempt on verification endpoint; endpoint returns only pass/fail with deployment county; no applicant personal data in response body
EC-SEC-03: HSM unavailable during credential or MOU signing; operation pauses; applications continue to APPROVED state; signing queue drains on HSM restore; no credential lost
EC-SEC-05: injection on all input fields (name, mandate, deployment plan); parameterised queries only; verified by Semgrep, sqlx compile-time checks, and ZAP
EC-SEC-09: rate-limit evasion on credential verification or application submission; keyed by subject/IP-slash-64; priority shedding applied; 429 returned with Retry-After header
EC-SEC-16: secrets in logs/traces; NPS advisory data and observer PII excluded from telemetry; log-scanner fixture in CI confirms no PII in OTEL spans
EC-IDN-02: JWT expires mid-request during vetting or assignment action; in-flight action completes; next request returns 401 with silent refresh prompt
EC-IDN-09: dual-control revocation; second-officer certificate revoked between initiating and committing; both certificates re-validated at commit; revocation rejected if either invalid
EC-IDN-10: SpiceDB down; fail-closed on all writes; class-C reads only; 503 problem+json returned; no partial state transition committed
EC-HUM-06: screen readers on organisation portal and individual self-application; full task completion at WCAG 2.2 AA; verified by axe-core and manual scripts
EC-HUM-08: EN + SW + French + Arabic display options; no string truncation; ICU plurals on roster summary counts; font fallback for all scripts
EC-HUM-09: colour-blind users on scanner app result screen; result encoded as colour + symbol + text; ΔE registry check passes
EC-SCL-01: 5,000 concurrent roster uploads at window open; queue-based processing absorbs burst; 10-minute bulk-import SLA maintained under load
EC-SCL-04: consumer lag > 60 seconds on credential-issuance or CRL-publish queue; alert fires; backpressure applied; no OOM; CRL publish SLA re-evaluated
EC-CON-01: scanner app offline; local CRL cache used for verification; alert fires if CRL age > 30 minutes; manual block list distributed via SMS to field coordinators
EC-OPS-05: NPS API unavailable during vetting; advisory-only path; manual criminal-record self-declaration accepted; vetting proceeds; NPS advisory flag updated on restore
EC-OPS-06: international missions submit late rosters; pre-loaded org profiles and dedicated liaison officer reduce delay; conditional org approval queues individual applications pending MOU completion
EC-LEG-02: DPA 2019 s.30(1)(b) lawful basis; biometric photo retained 7 years; NPS advisory data not retained post-vetting; DPIA registered; ODPC notified of any breach within 72 hours
EC-LEG-04: quota contention in high-interest constituencies; Elections Act s.21A quota enforced by system; over-quota observers offered alternate constituency; no manual override without Accreditation Officer dual-sign
EC-DAT-13: soft-deleted observer (withdrawn by organisation admin); tombstone honoured in public summary counts and M34 push; purge removes record without breaking constituency quota accounting
EC-OPS-07: Field Coordinator absent during batch assignment; assignment re-assigned to next available coordinator; deployment letters re-sent; credential exactly-once on commit

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | MOU template generation, credential signature, CRL generation, quota enforcement | nextest + flutter_test | M43-UNIT- |
| contract | NPS mock, M34 push API, M15 list sync, P06 HSM signing | Pact | M43-PACT- |
| integration | organisation approval, MOU workflow, individual credential, deployment push to M34, revocation, CRL | integration_test | M43-INT- |
| e2e | ELOG organisation applies, individual approved, credential issued, deployed to M34, verified at scanner | Flutter driver | M43-E2E- |
| security | credential serial enumeration; revocation bypass; roster injection attack; over-quota assignment | ZAP + red team | M43-SEC- |
| a11y | WCAG 2.2 AA; EN + SW; French + Arabic display option smoke test | axe-core + manual | M43-A11Y- |
| load | 5,000 concurrent roster uploads; 3,000 row bulk import within 10 min | k6 | M43-LOAD- |

Tags: `#[trace("FR-M43-009","AC-M43-06","EC-LEG-04")]`

## Runbooks and dashboards

- `docs/runbooks/training-svc-degradation.md` - extend with observer accreditation CRL sync failure and MOU workflow stalls
- `docs/runbooks/observer-quota-override.md` - new runbook; quota contention resolution; IEBC Accreditation Officer manual override procedure
- Grafana dashboard: `eel-iems-43` (alert ruleset: `iems-43-alerts`)
- SLO alerts: CRL propagation <= 5 min; credential QR verification <= 300 ms; bulk import <= 10 min for 5,000 rows

## Open questions

- IEBC Observer Accreditation Regulations 2022: must be confirmed unchanged for the 2027 cycle; any regulatory changes drive model and workflow changes; IEBC Legal Director sign-off required before spec is finalised.
- Shared scanner app with M42: a single Flutter app configured per accreditation type (MEDIA vs OBSERVER) is the intended design; scanner app team ownership must be agreed in sprint planning.
- NPS API MoU timeline: if not signed by T-270, the manual self-declaration fallback becomes the primary path for all vetting; confirm probability with IEBC Security.
- Geographic coverage target: >= 85% constituencies with at least one observer; system does not auto-enforce this; IEBC Field Deployment Coordinator is responsible; gap report must be built into the Deployment Board.
