# M25 special-voter-workflows

| field | value |
|---|---|
| M-number | M25 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.5 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M25-special-voter-workflows.md |

## Purpose

Provide legally compliant, auditable registration and voting workflows for five special voter categories (remand prisoners, hospital/infirmary voters, KDF/GSU/Prison Service personnel, refugees conditional on judicial expansion, and IDPs) that cannot use standard polling-station procedures; each category has a dedicated kit mode, signed-witness-chain protocol, and post-event IAB audit sample.

## Surfaces and languages

| surface | technology | path under services/support-admin-svc/ |
|---|---|---|
| Special-mode kit UI | Flutter | apps/special-kit/ |
| Special Voter Operations Dashboard | Phoenix LiveView | ex/special_web/ |
| IDP Re-assignment Centre Console | Django | django/idp_admin/ |
| Public special voter counts page | Astro | astro/special-public-site/ |
| API | Rust (axum) | crates/support-admin-api/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| SpecialVoterFacility | id, category (REMAND_PRISON/HOSPITAL/BARRACKS/REFUGEE_CAMP/IDP_CAMP), county_code, kit_ids, presiding_officer_id, languages, voting_date, published_voter_count, status | B | T1-consensus |
| SpecialVoterEvent | id, facility_id, category, event_type, voter_id (encrypted), officer_id, witness_ids, reason_code, event_at, merkle_node_id | A | T1-audit |
| SpecialForm34 | id, facility_id, form_variant (34A-R/34A-H/34A-B/34A-ID), tallies, signatures, transmitted_at, merkle_node_id | A | T1-audit |
| IDPReassignment | id, voter_id, original_station_code, idp_facility_id, reassigned_at, displacement_source, effective_date | B | T1-voter |

## APIs

OpenAPI file: `packages/openapi-contracts/support/special-voter.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/special/facilities | 2 | JWT+SpiceDB | n/a | list all special voter facilities |
| GET /v1/special/facilities/{id} | 2 | JWT+SpiceDB | n/a | facility detail and status |
| POST /v1/special/facilities/{id}/events | internal gRPC | SVID mTLS | yes | submit special voter event from kit |
| GET /v1/special/facilities/{id}/events | 2 | JWT+SpiceDB | n/a | event log for a facility |
| POST /v1/special/forms/34 | internal gRPC | SVID mTLS | yes | submit special Form 34 |
| GET /v1/special/forms/34 | 1 | public | n/a | list published special forms |
| GET /v1/special/forms/34/{id} | 1 | public | n/a | single form with Merkle proof |
| POST /v1/special/idp/reassignments | 2 | JWT+SpiceDB | yes | record IDP voter reassignment |
| GET /v1/special/dashboard | 2 | JWT+SpiceDB | n/a | ops coordinator live status |
| GET /v1/special/audit/sample/{category} | 2 | JWT+SpiceDB (IAB_AUDITOR) | n/a | IAB random-sample authentication logs |
| GET /v1/special/public/counts | 1 | public | n/a | voter counts per facility (no PII) |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.support.special_facility_opened.v1 | emits | eel.support.special-facility.v1 | operational | yes | at-least-once idempotent |
| eel.support.special_voter_event.v1 | emits | eel.support.special-voter.v1 | operational | yes | at-least-once idempotent |
| eel.support.special_form34_transmitted.v1 | emits | eel.support.form34-special.v1 | event-source | yes | at-least-once idempotent |
| eel.support.secrecy_violation_detected.v1 | emits | eel.support.violation.v1 | operational | yes | exactly-once |
| eel.support.idp_reassignment_recorded.v1 | emits | eel.support.idp.v1 | operational | yes | at-least-once idempotent |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M01 BVR / M02 BVR Back-End | internal | integrations/bvr-special/ | yes | circuit breaker; kit queues |
| M05 BVI / M06 BVI Back-End | internal | integrations/bvi-special/ | yes | circuit breaker; offline mode |
| M03 ABIS | internal | integrations/abis/ | yes | delayed retry |
| M10 RTS Back-End | internal | integrations/rts/ | yes | circuit breaker; form queued |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | queue; anchor on reconnect |
| P06 HSM | P06 | integrations/hsm/ | yes | signing queue |
| M11 Public Portal | internal | integrations/portal/ | yes | fallback CDN |
| Kenya Prisons Service | external (MoU) | integrations/prisons-service/ | yes (stub) | fallback manual roll upload |
| MoH / Hospital Authorities | external (MoU) | integrations/hospital/ | yes (stub) | manual scheduling |
| KDF/GSU | external (MoU) | integrations/kdf/ | yes (stub) | manual officer list |
| UNHCR Kenya | external (MoU, conditional) | integrations/unhcr/ | yes (stub) | feature-flagged off by default |
| NDOC / Red Cross | external | integrations/ndoc/ | yes (stub) | static dataset fallback |
| I08 SMS/USSD | I08 | integrations/sms-gateway/ | yes | SMS fallback |

## ML and AI

- None directly; on-device liveness / quality models from M28 packaged in special-kit firmware.

## Dependencies

- Modules: M01, M02, M03, M05, M06, M10, M11
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, kit-sdk-flutter

## Feature checklist

1. FR-M25-001: REMAND_PRISON kit mode; voter roll scoped to registered prison's remand population from Prisons Service MoU
2. FR-M25-002: dual biometric sign-in of Prison Governor + IEBC deputy Presiding Officer required to open kit
3. FR-M25-003: party-agent complement enforced per IEBC regulation; minimum one agent per leading presidential candidate
4. FR-M25-004: fingerprint match via ABIS; fallback iris + face with Presiding Officer signed authorisation
5. FR-M25-005: RFID-tagged sealed kit arrives at prison gate; custody log and party-agent custody certificate
6. FR-M25-006: voting schedule published per facility in Kenya Gazette >= 14 days before election
7. FR-M25-007: Form 34A-R published within 30 min of close via standard RTS pipeline
8. FR-M25-008: IAB random 5% sample of remand-prisoner authentication logs reviewed at T+7
9. FR-M25-009: HOSPITAL mode: tablet + portable fingerprint + iris scanner + battery pack
10. FR-M25-010: hospital voter roll cross-referenced from standard register by county returning officer
11. FR-M25-011: Elections Act s.72 consent and witness protocol; attending clinician as impartial witness
12. FR-M25-012: s.44A complementary mechanism for voters who cannot present biometrics normally; dual party-agent witness
13. FR-M25-013: assisted voting: patient appoints assistant; assistant identity and patient consent logged; assistant restrictions enforced
14. FR-M25-014: hospital kit operates offline; Form 34A-H synced on return to base
15. FR-M25-015: hospital voter list published anonymised (count only) in Kenya Gazette at T-14
16. FR-M25-016: BARRACKS mode; voter roll scoped to service personnel; Elections Act s.5 special enrolment
17. FR-M25-017: civilian control enforced; IEBC-seconded Returning Officer biometric must match kit open; no military credential
18. FR-M25-018: ballot secrecy enforcement; physical booth positioned away from superior officers; proximity alert logged
19. FR-M25-019: special enrolment window for service personnel; roll compiled by IEBC only
20. FR-M25-020: Form 34A-B published within 30 min of close
21. FR-M25-021: KDF ballot-secrecy violation events reviewed at 100% by IAB
22. FR-M25-022: REFUGEE_VOTING_ENABLED feature flag (default false); activation via IEBC-authorised config change
23. FR-M25-023: refugee voter roll from UNHCR Kenya + Dept of Refugee Services when flag activated
24. FR-M25-024: UNHCR ProGres v4 biometric match API; ABIS does not store refugee biometrics independently
25. FR-M25-025: refugee voting at designated refugee station when flag is activated
26. FR-M25-026: IDP register compiled from UNHCR/NDOC/Red Cross cross-referenced with standard voter register
27. FR-M25-027: pre-election IDP verification window 15 days before election; roll re-assignment recorded
28. FR-M25-028: IDP_CAMP mode; re-assigned voters; standard BVI workflow
29. FR-M25-029: Form 34A-I variants included in correct constituency aggregation based on original registration
30. FR-M25-030: all special-category events anchored in Merkle chain within 60 min (or on sync if offline)
31. FR-M25-031: special-category kit dispatch tracked with same RFID chain-of-custody as standard kits
32. FR-M25-032: special-category voter count per facility published on public portal at T-14
33. Violation event: auto-trigger on secrecy-violation; officer must acknowledge within 5 min or auto-escalate to IEBC Chair
34. IDP re-assignment: day-of s.44A complementary mechanism as fallback if voter turned away
35. Overcounting guard: voter cannot authenticate at both original and IDP station
36. DPA 2019: health status never stored; prison charge data never stored; refugee biometrics not in ABIS
37. Multi-language: EN + SW + Somali and Oromo for North Eastern and Rift Valley refugee/IDP facilities
38. Offline operation: >= 8 hours for all special modes; hospital kit >= 6 hours battery

## Acceptance criteria

1. AC-M25-01: 100% of eligible remand facilities offered the vote per schedule; measured by facility log vs gazette
2. AC-M25-02: >= 95% hospital voter sessions completed per schedule; session completion log
3. AC-M25-03: 100% KDF ballot-secrecy violation events reviewed within 24 hours; violation event log
4. AC-M25-04: >= 98% of special Form 34 published within 30 min of close; timestamp delta
5. AC-M25-05: 100% of IAB 5% audit sample reviewed at T+7; audit completion log
6. AC-M25-06: 100% IDP re-assignments reflected in correct constituency tally; aggregation cross-check
7. AC-M25-07: 100% special-category events in Merkle chain within 60 min; Merkle reconciliation
8. AC-M25-08: dual biometric sign-in for REMAND/BARRACKS kit enforced; unit test + UAT
9. AC-M25-09: REFUGEE_VOTING_ENABLED flag defaults false; activation requires IEBC_CHAIR HSM
10. AC-M25-10: WCAG 2.2 AA audit pass on voter-facing and public screens

## Applicable edge cases

**connectivity**
- EC-CON-01: hospital kit may be fully offline during patient rounds; 8-hour queue
- EC-CON-03: remote remand prison may have 2G only; kit uses signed SMS fallback

**identity**
- EC-IDN-08: duress finger during BVI; covert alert, UI identical
- EC-LEG-03: s.44A invoked at hospital or remand for medical inability

**data**
- EC-DAT-11: spoilt > valid in special facility; block with reason

**security**
- EC-SEC-01: signature mismatch on special Form 34; never applied
- EC-SEC-02: kit tamper in transit to facility; quarantine stream

**legal**
- EC-LEG-03: s.44A complementary mechanism for hospital patients, amputees, sedated voters

**operations**
- EC-OPS-01: kit battery death mid-session in hospital; journal-before-act; exact resume
- EC-OPS-06: facility relocation (flood/curfew); HSM-signed geofence update required

**human**
- EC-HUM-01: worn fingerprints common in manual labourers at barracks; multi-finger retry
- EC-HUM-02: amputee voter at hospital; face/ID path
- EC-HUM-05: assisted voting at hospital; consent logged; no secrecy leak
- EC-HUM-06: screen readers; AA compliance
- EC-HUM-08: 11 languages; Somali and Oromo for northeast IDP facilities

**kit**
- EC-KIT-01: GNSS spoof at remote prison; multi-constellation + IMU check

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | category-mode config, reason-code catalogue, IDP overcounting guard, dual-sign-in enforcement | nextest | M25-UNIT- |
| property | special event Merkle anchor, spoilt count reconciliation | proptest | M25-PROP- |
| contract | kit-to-backend special Form 34, RTS aggregation | Pact | M25-PACT- |
| integration | end-to-end remand-prison session with synthetic roll; hospital mobile kit sync; barracks result transmission | testcontainers | M25-INT- |
| e2e | simulated multi-category election day with all five categories active | Playwright + kit-simulator | M25-E2E- |
| security | barracks kit refuses military credential; hospital record never exposes health status; remand roll has no charge data | ZAP + cargo-fuzz | M25-SEC- |
| a11y | audio-guidance for hospital voter; large-text mode | axe-core + manual | M25-A11Y- |
| offline | 8-hour offline BVI at remand prison; sync integrity on reconnect | netem | M25-OFFLN- |
| chaos | hospital kit battery failure mid-session; remand prison network failure | Chaos Mesh | M25-CHAOS- |
| audit | IAB 5% sample selection reproducibility from Merkle chain | nextest | M25-AUDIT- |

Tags: `#[trace("FR-M25-002","AC-M25-08","EC-LEG-03")]`

## Runbooks and dashboards

- `docs/runbooks/support-admin-svc-degradation.md` - extend with special-mode offline paths
- `docs/runbooks/special-voter-violation-response.md` - new runbook for KDF secrecy violation alerts
- Grafana dashboard: `platform/observability/support-admin-svc/special-voter-dashboard.json`
- SLO alert: `platform/observability/support-admin-svc/special-voter-slo-alerts.yaml` (99.9% form publication)

## Open questions

- Kenya Prisons Service MoU negotiation timing: may delay remand roll data availability; build ingestion pipeline on synthetic data until MoU is signed.
- Refugee eligibility court expansion: feature-flagged but activation process and timeline unknown; post-award monitoring required.
- UNHCR ProGres v4 API availability and authentication schema must be confirmed before integration sprint.
- Hospital voter roll cross-reference methodology requires MoH sign-off; timing risk if MoH MoU is delayed.
