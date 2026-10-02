# M45 party-registration-management

| field | value |
|---|---|
| M-number | M45 |
| Home service | crms-svc (ADR-0020) |
| Owning work package | WP-4.11 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5%; 99.5% during annual return filing window (January-March) |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M45-party-registration-management.md |

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

Party registration:
1. FR-M45-001: new party registration wizard: party name (EN + SW), abbreviation, ideology, founding constitution PDF, headquarters address, regional offices, officials roster with NIN, proposed symbol (metadata FK to M67), proposed colour hex, supporting membership list (minimum 1,000 registered voters across at least 24 counties per PPA 2011 s.7)
2. FR-M45-002: membership list CVR cross-reference via M02; verified count returned per county; must meet PPA 2011 s.7 threshold to advance; completion <= 30 minutes for 10,000-row upload
3. FR-M45-003: party name and abbreviation uniqueness check against active register; block on conflict; Registrar may override with recorded reason
4. FR-M45-004: symbol perceptual-hash uniqueness check via M67; block on conflict above configurable threshold; Registrar override with recorded reason
5. FR-M45-005: application lifecycle FSM: DRAFT -> SUBMITTED -> DOCS_REVIEW -> MEMBERSHIP_VERIFIED -> REGISTRAR_REVIEW -> APPROVED / CONDITIONAL / REJECTED -> GAZETTED; invalid transitions rejected with reason
6. FR-M45-006: conditional approval: Registrar specifies remediation items; 30-day countdown notice sent to party; application lapses automatically if conditions unmet at day 30
7. FR-M45-007: RPP Deputy Registrar document checklist vetting: document type, PDF format, max 20 MB, virus scan; per-item error report on failure
8. FR-M45-008: 60-day advance reminder to party for annual return (31 March deadline); second reminder at 30 days; final notice at 7 days; all reminders logged
9. FR-M45-009: annual return filing: audited accounts upload, updated officials roster, membership count update, governance compliance self-declaration; filing receipt issued
10. FR-M45-010: SUSPENDED status applied automatically 30 days after annual return grace period expires; propagated to M12 and M47 via eel-outbox-rs within 5 minutes
11. FR-M45-011: gazette notice PDF generated (the Commission standard format); Registrar HSM-signed via P06; published to public register within 14 days of approval
12. FR-M45-012: party deregistration: Registrar initiates with statutory ground (PPA 2011 s.17); party notified; 21-day appeal window; DEREGISTERED status confirmed after window; M12 and M47 notified via eel-outbox-rs
13. FR-M45-013: deregistration appeal: party files appeal within 21-day window; Registrar reviews; appeal outcome recorded; all events audit-logged

Coalition registration:
14. FR-M45-014: coalition application: coalition name, member parties (each ACTIVE), agreement document, seat-allocation formula, dissolution trigger; minimum two member parties
15. FR-M45-015: coalition confirmation: each member party official confirms participation via authenticated portal; coalition ACTIVE only after all confirmations received
16. FR-M45-016: coalition seat-allocation formula stored and surfaced in M12 CRMS nomination workflow
17. FR-M45-017: coalition dissolution: any member party may trigger dissolution; coalition status DISSOLVED; M12 notified within 5 minutes via eel-outbox-rs

Faction dispute management:
18. FR-M45-018: faction-dispute intake: any party member submits structured dispute form (rival leadership claim, expelled-member grievance, branch election dispute); case number issued within 24 hours; routed to PPDT Clerk
19. FR-M45-019: dispute status tracker: RECEIVED -> UNDER_REVIEW -> HEARING_SCHEDULED -> DETERMINATION_ISSUED / WITHDRAWN; party and disputant can track via portal
20. FR-M45-020: PPDT determination ingest: PPDT Clerk uploads signed PDF; party record updated if determination mandates change; change logged in audit trail
21. FR-M45-021: PPDT fallback: if PPDT API unavailable, structured PDF email pathway; RPP staff enter case number manually; system accepts manual entry; audit logs fallback path

Governance and compliance:
22. FR-M45-022: political financing semi-annual disclosure per PP (Amendment) Act 2022 s.24; foreign-origin funds flagged for RPP review automatically
23. FR-M45-023: governance compliance score (0-100) per party: timeliness of annual return, dispute-free record, financing disclosures; score published on public register
24. FR-M45-024: compliance score formula version-controlled; score recalculated on each relevant event; formula requires Registrar approval before activation

Data protection and access control:
25. FR-M45-025: officials' NIN collected for identity verification; not exposed on any public endpoint; retained 7 years per DPA 2019 s.30(1)(b); DPIA filed
26. FR-M45-026: RPP back-office: application queue with status filter; membership-threshold meter; symbol uniqueness result panel; approve/conditional/reject modal with mandatory reason field
27. FR-M45-027: public party register: searchable by name, abbreviation, status, compliance score; gazette notice download; no PII exposed; WCAG 2.2 AA; EN + SW
28. FR-M45-028: all Registrar decisions audit-logged and appended to P05 Merkle chain within 60 seconds of decision event

Integrations and notifications:
29. FR-M45-029: M12 CRMS notified of every party status change within 5 minutes via eel-outbox-rs webhook; queue on failure; delivery retry with exponential backoff; M12 SLA <= 5 minutes
30. FR-M45-030: M47 Nomination Fee receives party ACTIVE / SUSPENDED / DEREGISTERED status change; nomination fee acceptance blocked for non-ACTIVE parties
31. FR-M45-031: M46 Party Primaries Oversight performs ACTIVE gate check on each opt-in request; non-ACTIVE parties rejected at M45 API with reason code

## Acceptance criteria

1. AC-M45-01: portal p99 <= 1 s under 500 concurrent party users; k6 load test with crms-svc passes at routine 99.5% availability class
2. AC-M45-02: membership CVR cross-reference for 10,000 rows completes within 30 minutes; load test with M02 stub passes
3. AC-M45-03: party status change (APPROVED, SUSPENDED, DEREGISTERED) propagated to M12 within 5 minutes via eel-outbox-rs; integration test with webhook receiver confirms delivery timestamp
4. AC-M45-04: system availability >= 99.5% during January-March annual-return filing window; SLO burn-rate alert configured and tested in Alertmanager
5. AC-M45-05: gazette notice published to public register within 14 days of approval; P06 HSM signature verified by public portal; integration test
6. AC-M45-06: officials' NIN absent from GET /v1/parties and GET /v1/parties/{id}/status responses; automated OpenAPI response scanner and ZAP security scan pass
7. AC-M45-07: faction dispute submitted; PPDT case number issued within 24 hours; PPDT stub integration test confirms case number in response
8. AC-M45-08: conditional approval 30-day countdown fires at day 30; application lapses automatically; unit test with mocked clock at T+30 midnight EAT passes
9. AC-M45-09: SUSPENDED status applied automatically 30 days after annual-return grace period; M12 webhook received within 5 minutes; integration test
10. AC-M45-10: coalition registration: two member parties confirm; coalition appears in M12 as valid nomination affiliation; integration test
11. AC-M45-11: WCAG 2.2 AA axe-core audit passes on party applicant portal and public party register; EN + SW UI strings verified; manual review sign-off
12. AC-M45-12: all Registrar decisions appended to P05 Merkle chain within 60 seconds; integration test confirms leaf insertion with valid anchor timestamp
13. AC-M45-13: duplicate party name or abbreviation submission returns 409 Conflict with reason; uniqueness constraint and integration test pass

## Applicable edge cases

data:
- EC-DAT-01: CVR extract outdated relative to registration date; cross-reference uses registration-date-stamped CVR snapshot; timestamp logged in membership verification result
- EC-DAT-02: same application idempotency-key with different party name body; 422 idempotency-key-reuse returned; no partial application persisted
- EC-DAT-06: party submits membership CSV v1 format while system expects v2; schema shim upgrades envelope; both versions accepted; compat check in CI passes
- EC-DAT-07: partial batch failure in membership CSV upload; per-row error report returned; valid rows committed; invalid rows listed for resubmission
- EC-DAT-09: Unicode party names (Swahili diacritics); NFC normalisation applied before uniqueness check; correct ICU collation on public register
- EC-DAT-10: compliance score denominators zero for new party with no history; no divide-by-zero; score initialised to baseline defined by Registrar formula

security:
- EC-SEC-05: SQL injection or script injection in party name, ideology, or dispute grounds fields; parameterised queries only; Semgrep and ZAP tests pass
- EC-SEC-06: SSRF via gazette_notice_url or constitution document URL; allowlist enforced; private-range redirect blocked; Nuclei test passes
- EC-SEC-07: malicious PDF in document upload (embedded scripts, zip-slip); strict PDF validation; no execution; fuzz corpus test passes
- EC-SEC-12: public register page cache poisoned; cache key includes gazette signature version; client verifies inclusion proof; Varnish injection test passes
- EC-SEC-16: officials' NIN must not appear in logs, traces, or OpenTelemetry spans; log-scanner CI fixture passes on every build

identity:
- EC-IDN-02: party official JWT expires mid-application wizard; in-flight step completes; next request returns 401 and silent refresh; no application data lost
- EC-IDN-09: Registrar approval requires dual-control (Registrar + Deputy); approver revoked mid-flow; re-validation at commit; approval blocked if approver revoked; compensating event logged
- EC-IDN-10: SpiceDB unavailable when Registrar attempts status change; fail closed; 503 problem+json returned; no status change persisted

scale:
- EC-SCL-01: 50 parties file annual returns concurrently during March deadline; crms-svc horizontally scales; CVR cross-reference 30-minute SLA maintained
- EC-SCL-04: eel-outbox-rs Kafka consumer lag > 60 s during March filing peak; backpressure applied; M12 webhook eventually delivered; no message dropped; SRE alerted

legal:
- EC-LEG-02: court-ordered injunction halting gazette publication; Registrar places party in LEGAL_HOLD status; M12 notified; status visible on public register
- EC-LEG-04: party delays annual return past 31 March; 60-day and 30-day reminders sent; SUSPENDED applied after 30-day grace; M12 propagation within 5 minutes; nomination workflow blocks SUSPENDED party

operations:
- EC-OPS-05: PPDT API MoU not signed; fallback to structured PDF email; RPP staff enter case number manually; system accepts manual entry; fallback path audit-logged
- EC-OPS-06: M12 CRMS webhook delivery fails repeatedly; eel-outbox-rs queues; SRE alerted at queue depth > 100; M12 SLA degraded but message not lost

connectivity:
- EC-CON-05: eel-outbox-rs webhook to M12 flaps on network instability; exponential backoff with retry budget; no duplicate status change delivered; server dedupe count 0

human:
- EC-HUM-08: Swahili party names and wizard step labels with long strings; no truncation; ICU plurals; 150% pseudo-locale test passes
- EC-HUM-09: party symbol colour hex validation; colour-blind accessibility warning displayed if proposed colour fails contrast threshold; ΔE check in public register

time:
- EC-TIM-04: annual return deadline 31 March computed in EAT; UTC storage; UTC-to-EAT conversion in reminder scheduler; boundary test passes
- EC-TIM-08: conditional approval 30-day lapse deadline computed inclusively in EAT; boundary test +/- 1 second passes; application lapses at midnight EAT day 30

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
- Political Parties Act 2011 and 2022 Amendment: must be confirmed as the operative legal framework for 2027 cycle; any amendment changes drive data model and workflow changes; the Commission Legal Director sign-off required before spec is finalised.
- Compliance score formula: §XXI.A and RPP policy define inputs but not weightings; the Commission Registrar must approve the scoring formula before implementation.
