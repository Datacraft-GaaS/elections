# M31 mobile-poll-clerk

| field | value |
|---|---|
| M-number | M31 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.8 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M31-mobile-poll-clerk.md |

## Purpose

Give the Deputy Presiding Officer and polling clerks (~92,000 on polling day) a companion tool that offloads work from the KIEMS device: searchable reference materials, structured incident logging, voter-lookup fallback when the kit is single-user busy, queue management, mirrored kit dashboards, paper-backup workflow guidance, and end-of-day report generation; the KIEMS kit remains authoritative for voter identification, and this app makes the team around it faster and less error-prone.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Poll Clerk Companion App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/poll-clerk/ |
| Kiosk Operator Mode (register inspection period) | Flutter | services/field-mobile-svc/apps/poll-clerk/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM. Languages: EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ClerkCheckIn | id, clerk_id, station_code, election_id, role (DPO/CLERK/KIOSK), biometric_hash, device_attestation_id, check_in_at | B | T1-consensus |
| VoterLookupAudit | id, clerk_id, station_code, lookup_reason, id_hash (no voter PII), dpo_attestation, looked_up_at | A | T1-audit |
| ClerkIncidentReport | id, clerk_id, station_code, incident_type, description, evidence_blob_ids, escalation_level, submitted_at, signature | B | T1-consensus |
| EndOfDayReport | id, station_code, clerk_id, opening_time, closing_time, turnout_count, incidents_count, spoilt_paper_log, agent_presence_log, dpo_signature, po_signature, submitted_at | A | T1-audit |

## APIs

OpenAPI file: `packages/openapi-contracts/field/poll-clerk.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/clerk/check-in | 2 | JWT+SpiceDB (DPO/CLERK) | yes | biometric + device attestation station check-in |
| GET /v1/clerk/station/{code}/dashboard | 2 | JWT+SpiceDB (DPO/CLERK) | n/a | read-only kit dashboard mirror from BVI back-end |
| POST /v1/clerk/voter-lookup | 2 | JWT+SpiceDB (DPO) | yes | voter register lookup fallback; rate-limited; logged |
| POST /v1/clerk/incidents | 2 | JWT+SpiceDB (DPO/CLERK) | yes | submit structured incident report |
| POST /v1/clerk/queue | 2 | JWT+SpiceDB (DPO/CLERK) | yes | manage voter queue (add, next, priority) |
| GET /v1/clerk/queue/{station_code} | 2 | JWT+SpiceDB (DPO/CLERK) | n/a | queue status |
| POST /v1/clerk/end-of-day | 2 | JWT+SpiceDB (DPO) | yes | submit signed end-of-day report |
| GET /v1/clerk/kiosk/register | 2 | JWT+SpiceDB (KIOSK_OPERATOR) | n/a | BVV register lookup in kiosk mode; staff-assisted OTP |
| POST /v1/clerk/kiosk/objection | 2 | JWT+SpiceDB (KIOSK_OPERATOR) | yes | file objection on behalf of citizen with consent |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.clerk_checked_in.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |
| eel.field.clerk_incident.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |
| eel.field.end_of_day_report.v1 | emits | eel.field.clerk.v1 | operational | yes | exactly-once |
| eel.field.voter_lookup_audit.v1 | emits | eel.field.clerk.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M06 BVI Back-End | internal | integrations/bvi/ | yes | dashboard mirror; incidents submitted; circuit breaker |
| M02 BVR Back-End | internal | integrations/bvr/ | yes | voter register lookup fallback; offline unavailable |
| M07 BVV | internal | integrations/bvv/ | yes | kiosk-mode register lookup and objection filing |
| M20 Inventory Management | internal | integrations/inventory/ | yes | kit self-check on app launch; assigned kit in inventory |

## ML and AI

- None.

## Dependencies

- Modules: M02, M06, M07, M20
- Platform: none
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M31-001: enrolment via IEBC staff ID + biometric + device attestation
2. FR-M31-002: assignment bound to specific polling station per election
3. FR-M31-003: MFA sign-in: biometric + PIN + device attestation
4. FR-M31-004: polling manual (EN + SW) with searchable index; searchable in <= 1 s on-device
5. FR-M31-005: statutory forms library (Form 34A/B/C templates) for paper-backup workflow; embedded offline
6. FR-M31-006: Elections Act key clauses (s.55 emergency close, s.6A cut-off) for on-the-spot reference
7. FR-M31-007: contact card: PO, RO, CRO, county support desk, war-room hotline, emergency numbers
8. FR-M31-008: live dashboard mirroring KIEMS BVI: election, opening/closing, turnout %, voters authenticated, kit battery, connectivity, GPS fix
9. FR-M31-009: read-only from BVI back-end (M06); no writes to authentication pipeline
10. FR-M31-010: voter register lookup fallback by ID when KIEMS temporarily unavailable; requires DPO justification + biometric + audit entry
11. FR-M31-011: lookup rate-limited; every lookup logged permanently with PO + DPO biometric attestation
12. FR-M31-012: no voter authentication from this app; information lookup only; kit remains authoritative
13. FR-M31-013: queue management: register voter presence; call next per capacity; manage priority (elderly, pregnant, disabled)
14. FR-M31-014: voter takes a ticket; DPO sees position; PO calls forward
15. FR-M31-015: queue metrics: average wait time, throughput per hour
16. FR-M31-016: structured incident form: attempted double-vote, biometric fail, kit issue, physical incident, voter dispute, other; description; evidence upload
17. FR-M31-017: escalation options: to PO, to RO, to county support desk, to war room
18. FR-M31-018: incidents signed by DPO biometric; queued; sent to BVI back-end (M06)
19. FR-M31-019: step-by-step paper-backup workflow guidance for when KIEMS unavailable; paper voter authentication; paper Form 34A completion; subsequent digital reconciliation
20. FR-M31-020: paper-backup templates embedded; no external network required
21. FR-M31-021: kiosk mode during register-inspection period: BVV register lookup for citizens with staff-assisted OTP retrieval
22. FR-M31-022: kiosk objection filing on behalf of citizen with citizen consent + witness signature
23. FR-M31-023: end-of-day report generated automatically: opening/closing times, throughput, incidents, spoilt paper-ballot log, agent presence
24. FR-M31-024: end-of-day report signed by PO + DPO biometric; submitted to BVI back-end
25. FR-M31-025: reference materials fully offline
26. FR-M31-026: incident logs queued; submitted on reconnect
27. FR-M31-027: register-lookup fallback requires network; queued with informational status if offline
28. Battery cost <= 5% per hour foreground; certificate pinning + device attestation mandatory; screen-capture protection on voter-lookup and incident screens; AES-256-GCM local store

## Acceptance criteria

1. AC-M31-01: end-of-day report generated, signed, and submitted; UAT
2. AC-M31-02: paper-backup workflow followed by real clerk in field trial
3. AC-M31-03: voter-lookup fallback fully audited; audit-trace check
4. AC-M31-04: kiosk mode passes ODPC review
5. AC-M31-05: reference materials searchable offline; airplane-mode test

## Applicable edge cases

**connectivity**
- EC-CON-01: KIEMS unavailable; paper-backup workflow guides; incidents queued locally; submitted on reconnect

**identity**
- EC-IDN-05: voter presents at wrong station; lookup fallback confirms; clerk advises on correct station; no authentication from this app

**operations**
- EC-OPS-01: kit battery death; companion app still functions; paper-backup mode active; end-of-day report queued
- EC-OPS-06: DPO absent; PO alone authorises via kit; app degrades to clerk-only features

**mobile**
- EC-MOB-02: mid-tier Android 10; battery <= 5% per hour; reference materials <= 1 s search
- EC-MOB-06: mTLS + certificate pinning mandatory

**human**
- EC-HUM-03: priority queue management for elderly, pregnant, disabled voters
- EC-HUM-06: WCAG 2.2 AA on all screens

**scale**
- EC-SCL-01: 92,000 clerks checking in over 60-minute window on election morning; back-end check-in worker auto-scales

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | lookup rate-limit, queue priority logic, end-of-day arithmetic, paper-backup template rendering | nextest + flutter_test | M31-UNIT- |
| contract | BVI dashboard, BVR lookup, BVV kiosk objection | Pact | M31-PACT- |
| integration | lookup audit trail; incident submission; end-of-day report signing | integration_test | M31-INT- |
| e2e | DPO check-in, lookup, queue, incident, end-of-day | Flutter driver | M31-E2E- |
| security | voter-lookup rate-limit enforcement; kiosk mode ODPC compliance | ZAP | M31-SEC- |
| a11y | WCAG 2.2 AA; searchable reference | axe-core | M31-A11Y- |
| offline | reference materials offline; incident queue offline | netem | M31-OFFLN- |

Tags: `testWidgets('lookup fallback [FR-M31-010,AC-M31-03]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with poll-clerk offline paths and voter-lookup fallback
- Grafana dashboard: `platform/observability/field-mobile-svc/poll-clerk-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/poll-clerk-slo-alerts.yaml`

## Open questions

- ODPC review of kiosk-mode OTP lookup flow: DPA 2019 lawful basis (public task) must be documented before kiosk mode activates; IEBC DPO to lead.
- Paper-backup workflow reconciliation protocol: who owns the digital reconciliation step after paper-backup session? Must be agreed with RO back-office team before field-trial.
