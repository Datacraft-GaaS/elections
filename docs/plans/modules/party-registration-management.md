# M45 party-registration-management

| field | value |
|---|---|
| M-number | M45 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5%; 99.5% during annual return filing window (January-March) |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M45-party-registration-management.md |

## Purpose

Automate the full lifecycle of political party registration and renewal under the Political Parties Act 2011 (PPA 2011) and the Registrar of Political Parties (RPP) mandate. Covers new party registration, annual returns, coalition registration, party symbol and colour metadata (symbol image registry deferred to M67), governance disclosure, and faction-dispute intake routed to PPDT. Integrates downstream with M12 CRMS (candidate party affiliation) and M47 (nomination fee).

Strategy reference: §XXI.A (Political Party Ecosystem). Political Parties Act 2011; Political Parties (Amendment) Act 2022; Elections Act s.22.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Party Portal (WCAG 2.2 AA) | Astro | services/crms-svc/astro/party-registration/ |
| RPP Back-Office (Registrar) | Phoenix LiveView | services/crms-svc/ex/rpp_web/ |
| Public Party Register (WCAG 2.2 AA) | Astro | services/crms-svc/astro/party-registration/ |
| API | Rust (axum) | services/crms-svc/crates/crms-api/ |

Languages: EN + SW; public register WCAG 2.2 AA.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| PoliticalParty | id, name_en, name_sw, abbreviation, ideology, status (DRAFT/SUBMITTED/DOCS_REVIEW/MEMBERSHIP_VERIFIED/REGISTRAR_REVIEW/APPROVED/CONDITIONAL/REJECTED/GAZETTED/SUSPENDED/DEREGISTERED), registration_number, gazette_notice_url, symbol_registry_id (FK to M67), colour_hex, hq_address, verified_membership_count, compliance_score (0-100), coalition_id, created_at, updated_at | B | T1-consensus |
| CoalitionRegistration | id, name, member_party_ids, agreement_doc_url, seat_allocation_formula, status (ACTIVE/DISSOLVED), dissolved_at, created_at | B | T1-consensus |
| FactionDispute | id, party_id, case_number, dispute_type, submitted_by, description, status (RECEIVED/UNDER_REVIEW/HEARING_SCHEDULED/DETERMINATION_ISSUED/WITHDRAWN), ppdt_determination_url, created_at, updated_at | B | T1-consensus |
| AnnualReturn | id, party_id, filing_year, audited_accounts_url, updated_officials_roster, membership_count_update, governance_self_declaration, foreign_origin_flag (bool), filed_at, accepted_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/crms/party-registration.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/parties | 2 | public | yes | submit new party registration application |
| GET /v1/parties | 1 | public | n/a | public party register (paginated) |
| GET /v1/parties/{id}/status | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | n/a | party registration status with detail |
| POST /v1/parties/{id}/annual-return | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | yes | file annual return |
| GET /v1/parties/{id}/membership-check | internal gRPC | RPP_SYSTEM mTLS | n/a | membership threshold verification result |
| POST /v1/coalitions | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | yes | register coalition; member parties must confirm |
| POST /v1/coalitions/{id}/confirm | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | yes | member party confirms coalition participation |
| POST /v1/coalitions/{id}/dissolve | 2 | JWT+SpiceDB (PARTY_OFFICIAL) | yes | trigger coalition dissolution |
| POST /v1/disputes | 2 | JWT+SpiceDB (PARTY_MEMBER) | yes | submit faction dispute; PPDT case number issued within 24 hr |
| GET /v1/disputes/{id} | 2 | JWT+SpiceDB (PARTY_MEMBER/PPDT_CLERK) | n/a | dispute status |
| POST /v1/disputes/{id}/determination | 2 | JWT+SpiceDB (PPDT_CLERK) | yes | PPDT uploads signed determination PDF |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.crms.party_registered.v1 | emits | eel.crms.party.v1 | operational | yes | at-least-once idempotent |
| eel.crms.party_status_changed.v1 | emits | eel.crms.party.v1 | operational | yes | exactly-once |
| eel.crms.party_deregistered.v1 | emits | eel.crms.party.v1 | audit | yes | exactly-once |
| eel.crms.coalition_dissolved.v1 | emits | eel.crms.party.v1 | operational | yes | exactly-once |
| eel.crms.faction_dispute_submitted.v1 | emits | eel.crms.party.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR Back-End | internal | integrations/bvr/ | yes | membership-list CVR cross-reference; registration-date-stamped CVR snapshot used |
| M12 CRMS | internal | integrations/crms/ | yes | party status webhook; M12 propagated within 5 minutes of status change; queue on failure |
| M47 Nomination Fee | internal | integrations/nomination-fee/ | yes | party ACTIVE status check before nomination fee accepted |
| M67 Symbol Registry | internal | integrations/symbol-registry/ | yes | symbol uniqueness perceptual-hash check; block on conflict |
| M46 Party Primaries Oversight | internal | integrations/primaries/ | yes | party ACTIVE status gate for M46 opt-in |
| PPDT | external | integrations/ppdt/ | yes (stub) | dispute referral; determination ingest; fallback to structured PDF email if API delayed |
| P05 Merkle Log | P05 | integrations/merkle/ | yes | Registrar decision audit anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes (stub) | gazette notice signing |

## ML and AI

- None (symbol uniqueness is perceptual-hash comparison; compliance score is rule-based).

## Dependencies

- Modules: M02, M12, M46, M47, M67
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs

## Feature checklist

1. FR-M45-001: new party registration: party name (EN + SW), abbreviation, ideology, founding constitution, headquarters address, regional offices, officials roster (officials + NIN), proposed symbol (metadata FK to M67), proposed colour (hex), supporting membership list (minimum 1,000 registered voters across at least 24 counties per PPA 2011 s.7)
2. FR-M45-002: membership list CVR cross-reference via M02; system returns verified count per county; must meet PPA 2011 s.7 threshold to advance; <= 30 minutes for 10,000-row upload
3. FR-M45-003: party name and symbol uniqueness check; name and abbreviation against active register; symbol image perceptual-hash against M67; block on conflict; Registrar can override with reason
4. FR-M45-004: application lifecycle: DRAFT -> SUBMITTED -> DOCS_REVIEW -> MEMBERSHIP_VERIFIED -> REGISTRAR_REVIEW -> APPROVED/CONDITIONAL/REJECTED -> GAZETTED
5. FR-M45-005: conditional approval with remediation items; party has 30 days to satisfy; application lapses if unmet
6. FR-M45-006: annual renewal by 31 March each year (audited accounts, updated officials roster, membership count, governance compliance); SUSPENDED after 30-day grace period on failure; 60-day advance reminders
7. FR-M45-007: gazette notice (PDF, IEBC standard format); Registrar HSM-signed; published to public register within 14 days of approval
8. FR-M45-008: party deregistration: statutory ground (PPA 2011 s.17); 21-day appeal window; downstream systems notified (M12, M47)
9. FR-M45-009: coalition application: coalition name, member parties (each must be ACTIVE), coalition agreement, seat-allocation formula, dissolution trigger; member parties confirm participation
10. FR-M45-010: coalition seat-allocation formula stored and surfaced in M12 CRMS nomination workflow
11. FR-M45-011: coalition dissolution by any member party; downstream systems notified
12. FR-M45-012: faction-dispute intake: any party member submits structured dispute form; case number assigned within 24 hours; routed to PPDT Clerk
13. FR-M45-013: dispute status tracker: RECEIVED -> UNDER_REVIEW -> HEARING_SCHEDULED -> DETERMINATION_ISSUED/WITHDRAWN
14. FR-M45-014: PPDT determination ingest: signed PDF uploaded; party record updated if determination mandates change; logged
15. FR-M45-015: political financing disclosure semi-annually per PP (Amendment) Act 2022 s.24; foreign-origin funds flagged for RPP review
16. FR-M45-016: governance compliance score (0-100) per party: timeliness of annual return, dispute-free record, financing disclosures; published on public register
17. Officials' NIN not exposed on public register; retained 7 years; DPA 2019 s.30(1)(b); DPIA filed
18. M12 CRMS notified of party status change within 5 minutes via eel-outbox-rs webhook

## Acceptance criteria

1. AC-M45-01: party submits 1,200-member list; CVR cross-reference completes within 30 minutes; threshold pass/fail displayed; UAT
2. AC-M45-02: Registrar issues gazette notice with HSM signature; published to public register within 14 days; integration test
3. AC-M45-03: party status change propagated to M12 within 5 minutes; integration test
4. AC-M45-04: faction dispute submitted; PPDT case number issued within 24 hours; PPDT stub test
5. AC-M45-05: coalition registration: two member parties confirm; coalition appears in M12 as valid affiliation; integration test
6. AC-M45-06: officials' NIN not exposed in public register endpoint; security test

## Applicable edge cases

**operations**
- EC-OPS-05: PPDT API integration stalled by MoU delay; fallback to structured PDF email with manual case number assignment; M45 records case number manually entered by RPP staff
- EC-OPS-06: M12 CRMS webhook delivery fails; eel-outbox-rs queues; M12 receives status on next successful delivery; M12 SLA <= 5 minutes from status change

**legal**
- EC-LEG-02: DPA 2019 s.30(1)(b); officials' NIN collected but not exposed publicly; DPIA registered; 7-year retention
- EC-LEG-04: party delays annual return; 60-day reminder sent; SUSPENDED flag propagated to M12 after 30-day grace period; nomination workflow blocks SUSPENDED parties

**scale**
- EC-SCL-01: 50 parties filing annual returns concurrently during March deadline; crms-svc auto-scales; 30-minute CVR cross-reference SLA maintained

**data**
- EC-DAT-01: CVR extract outdated relative to registration date; cross-reference uses registration-date-stamped CVR snapshot; timestamp logged in membership verification result

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | membership threshold validation, symbol perceptual-hash comparison, compliance score computation, gazette notice hash, 30-day conditional countdown | nextest | M45-UNIT- |
| contract | M02 CVR query, M12 webhook, M67 uniqueness check, PPDT referral API, P06 gazette signing | Pact | M45-PACT- |
| integration | registration to gazette; annual return filing; PPDT dispute; coalition registration; M12 propagation | integration_test | M45-INT- |
| e2e | party submits application, CVR verified, Registrar approves, gazette published, M12 notified, annual return filed | nextest e2e | M45-E2E- |
| security | officials' NIN exposure audit; party official access isolation; NIN not in public register | ZAP + red team | M45-SEC- |
| a11y | WCAG 2.2 AA on applicant portal and public register | axe-core + manual | M45-A11Y- |
| load | 50 concurrent annual returns; 10,000-row membership CSV CVR cross-reference within 30 min | k6 | M45-LOAD- |

Tags: `#[trace("FR-M45-002","AC-M45-01","EC-DAT-01")]`

## Runbooks and dashboards

- `docs/runbooks/crms-svc-degradation.md` - extend with party registration CVR cross-reference failures and M12 webhook queue drain
- `docs/runbooks/annual-return-filing-window.md` - new runbook; March deadline operations; SUSPENDED status propagation; RPP staff alerting
- Grafana dashboard: `eel-iems-45` (alert ruleset: `iems-45-alerts`)
- SLO alerts: M12 party status sync <= 5 min; CVR cross-reference <= 30 min; annual return SUSPENDED propagation SLA

## Open questions

- M67 Symbol Registry: perceptual-hash API must be ready at T-270 for symbol uniqueness checks; if M67 is delayed, symbol uniqueness falls back to exact-match only; Registrar can override; confirm M67 readiness with that team.
- PPDT API MoU: if not signed by T-270, the email-PDF fallback is the production path; RPP staff must be trained on manual case number entry.
- Political Parties Act 2011 and 2022 Amendment: must be confirmed as the operative legal framework for 2027 cycle; any amendment changes drive data model and workflow changes; IEBC Legal Director sign-off required before spec is finalised.
- Compliance score formula: §XXI.A and RPP policy define inputs but not weightings; IEBC Registrar must approve the scoring formula before implementation.
