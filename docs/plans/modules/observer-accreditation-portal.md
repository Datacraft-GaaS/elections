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
| POST /v1/mou/{org_id}/sign | 2 | JWT+SpiceDB (IEBC_LEGAL) + HSM | yes | Legal Director signs MOU for international mission |
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

1. FR-M43-001: organisation application: legal name, jurisdiction, registration documents, mandate letter, contact focal point, observer count, geographic deployment plan, prior Kenya election history
2. FR-M43-002: organisation type classification (DOMESTIC_NGO/INTERNATIONAL_MISSION/DIPLOMATIC_MISSION/RELIGIOUS_BODY/ACADEMIC); classification drives vetting depth and MOU requirement
3. FR-M43-003: MOU workflow for INTERNATIONAL_MISSION organisations: draft MOU from template; IEBC Director of Legal signs via HSM; counterpart uploads signed copy; status advances to MOU_SIGNED before individual observers can apply
4. FR-M43-004: individual observer application under approved organisation: biographic details, NIN/Passport, ICAO photo, language competencies, prior observer experience, deployment preference
5. FR-M43-005: background check: NPS advisory, IEBC watch-list, prior revocation history, criminal record self-declaration
6. FR-M43-006: application lifecycle: SUBMITTED -> DOCS_VERIFIED -> BACKGROUND_CHECK -> APPROVED/REJECTED -> CREDENTIAL_ISSUED -> DEPLOYED/SUSPENDED/REVOKED
7. FR-M43-007: digitally signed PDF credential with observer name, photo, organisation, deployment county, validity, credential serial (UUID7); signed via P06 HSM; QR verifiable offline
8. FR-M43-008: IEBC Coordinator assigns approved observers to specific constituencies; observer receives deployment letter with polling-station list; deployment data pushed to M34 Observer App
9. FR-M43-009: per-station observer quota enforcement per Elections Act s.21A; system prevents assignment beyond quota
10. FR-M43-010: pre-loaded profiles for known missions (KEOG, ELOG, EU EOM, AU, Carter Center, IFES, Commonwealth); org admin claims profile with document verification
11. FR-M43-011: revocation workflow; credential serial added to CRL; field coordinator notified; M34 app credentials for that observer invalidated
12. FR-M43-012: accreditation window enforcement per IEBC gazette notice; late applications rejected
13. FR-M43-013: public summary: observer counts per organisation and county; no individual names without consent
14. FR-M43-014: code-of-conduct digital acceptance before credential issuance
15. FR-M43-015: all mutations audit-logged; appended to P05 Merkle chain
16. Bulk roster import: CSV/XLSX up to 5,000 rows processed within 10 minutes; per-row error report

## Acceptance criteria

1. AC-M43-01: ELOG submits organisation application; approved within 5 working days; individual observer credential issued within 48 hours of approval; UAT
2. AC-M43-02: EU EOM organisation approved via MOU workflow; Legal Director HSM signature; counterpart upload advances status; UAT
3. AC-M43-03: deployment assignment pushed to M34 Observer App; observer sees assigned stations in app; integration test
4. AC-M43-04: observer credential revoked; CRL updated within 5 minutes; M34 app credential invalidated; integration test
5. AC-M43-05: 3,000-row bulk roster processed within 10 minutes; per-row error report; load test
6. AC-M43-06: per-station quota enforced; over-quota assignment returns 422; unit test

## Applicable edge cases

**connectivity**
- EC-CON-01: scanner offline; local CRL cache; CRL age alert > 30 min; same pattern as M42

**operations**
- EC-OPS-05: NPS API unavailable; advisory-only path; manual self-declaration accepted; vetting proceeds; NPS status flag updated on restore
- EC-OPS-06: international missions submit late rosters; pre-loaded org profiles and dedicated liaison officer reduce delay; conditional org approval queues individual applications pending MOU

**legal**
- EC-LEG-02: DPA 2019 s.30(1)(b) lawful basis; biometric photo retained 7 years; NPS data not retained post-vetting; DPIA registered
- EC-LEG-04: quota contention in high-interest constituencies; system enforces; overflow offered alternate constituency

**scale**
- EC-SCL-01: 5,000 concurrent roster uploads at window open; queue-based processing; 10-minute SLA maintained

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
