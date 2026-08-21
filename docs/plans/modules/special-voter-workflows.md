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

Category A: Remand Prisoners:

1. FR-M25-001: REMAND_PRISON kit mode: voter roll scoped to registered prison's remand population (pre-trial detainees only; convicted prisoners excluded unless court order extends right); roll sourced from Kenya Prisons Service CRS cross-reference under MoU.
2. FR-M25-002: Dual biometric sign-in of Prison Governor and IEBC-assigned deputy Presiding Officer required to open kit; kit cannot activate without both biometrics matched.
3. FR-M25-003: Party-agent complement enforced: minimum one agent per two leading presidential candidates; additional agents accommodated where facility permits.
4. FR-M25-004: Prisoner identity verification via ABIS fingerprint match at standard threshold; fallback to iris + face combination with Presiding Officer signed authorisation on documented fingerprint degradation.
5. FR-M25-005: Kit arrives sealed with RFID tag; custody log records prison gate entry; party agents sign custody certificate at gate; RFID tracked through same chain as standard kits (M10).
6. FR-M25-006: Voting schedule published per facility in Kenya Gazette >= 14 days before election day.
7. FR-M25-007: Form 34A-R (remand variant) published within 30 minutes of close via standard RTS pipeline; 100% of remand facilities.
8. FR-M25-008: IAB random 5% sample of remand-prisoner authentication logs reviewed within T+7; audit sample selection reproducible from Merkle chain.

Category B: Hospital/Infirmary Voters:

9. FR-M25-009: HOSPITAL mode: tablet + portable fingerprint scanner + portable iris scanner + battery pack; same OS and firmware as standard kit; provisioned in HOSPITAL mode.
10. FR-M25-010: Hospital voter roll derived from standard voter roll by cross-referencing patients confirmed as registered voters by hospital administration; approved by county returning officer.
11. FR-M25-011: Consent and witness protocol per Elections Act s.72: patient gives informed consent; attending clinician (or hospital administrator) signs as impartial witness; party agent of patient's choice may be present.
12. FR-M25-012: Physical mobility accommodation: Presiding Officer may authorise complementary mechanism per s.44A with reason code MEDICAL_INABILITY on documented inability to present biometrics normally; dual party-agent witness required.
13. FR-M25-013: Assisted voting per Elections Act s.72: patient appoints assistant of own choice; assistant must not be a candidate or party agent; kit logs assistant identity and patient consent; no secrecy leak path.
14. FR-M25-014: Hospital kit operates offline; Form 34A-H synced on return to base station within 60 minutes of reconnect; no case dropped on queue overflow.
15. FR-M25-015: Hospital voter list published anonymised (voter count per facility; no patient names) in Kenya Gazette at T-14.

Category C: KDF/GSU/Prison Service Voters:

16. FR-M25-016: BARRACKS mode: voter roll scoped to service-personnel registered at barracks station under Elections Act s.5 special enrolment.
17. FR-M25-017: Civilian control enforced: IEBC-seconded Returning Officer acts as Presiding Officer; kit's Presiding Officer biometric must match IEBC-seconded officer, not any military credential.
18. FR-M25-018: Ballot secrecy enforcement: physical booth positioned away from line-of-sight of superior officers; any officer approaching defined booth perimeter logged automatically as a SECRECY_VIOLATION event.
19. FR-M25-019: Special enrolment window for service personnel; roll compiled by IEBC from commanding officer's approved list; IEBC retains roll compilation authority and does not delegate to military chain.
20. FR-M25-020: Form 34A-B (barracks variant) published within 30 minutes of close.
21. FR-M25-021: IAB auditor reviews 5% random sample of barracks authentication logs at T+7; all ballot-secrecy violation events reviewed at 100% by IAB within 24 hours.

Category D: Refugees (Conditional Activation):

22. FR-M25-022: REFUGEE_VOTING_ENABLED feature flag (default false); activation via IEBC Chair HSM-signed config change only; no firmware update required; Datacraft monitors litigation and court orders.
23. FR-M25-023: When activated: refugee voter roll derived from UNHCR Kenya registration data cross-referenced with IEBC criteria (minimum residence period, no criminal conviction); roll compiled with UNHCR + Department of Refugee Services.
24. FR-M25-024: Refugee identity verification via UNHCR ProGres v4 biometric match API response; ABIS does not store refugee biometrics independently without explicit UNHCR data-sharing agreement.
25. FR-M25-025: Refugee voting at designated refugee station within or adjacent to refugee camp when flag is activated; standard BVI workflow applies.

Category E: IDPs:

26. FR-M25-026: IDP register compiled from UNHCR Kenya, NDOC, Kenya Red Cross displacement data cross-referenced with standard voter register; any registered voter in IDP camp who cannot travel to registered station offered re-assignment.
27. FR-M25-027: Pre-election IDP verification window: IEBC centres open for 15 days before election day for roll re-assignment; re-assignment recorded on voter register and reflected in BVI kit roll.
28. FR-M25-028: IDP station provisioned in IDP_CAMP mode: voter roll contains re-assigned IDPs; standard BVI workflow applies.
29. FR-M25-029: Form 34A-I variants published and included in correct constituency aggregation based on voter's original registered constituency.

Cross-category:

30. FR-M25-030: All special-category events anchored to Merkle chain within 60 minutes of event or on sync if offline; Merkle reconciliation script verifies 100% coverage at T+1.
31. FR-M25-031: Special-category kit dispatch tracked with same RFID chain-of-custody system as standard kits (M10); no special exemption from custody logging.
32. FR-M25-032: Special-category voter count per facility published on public portal at T-14; no PII; facility type filter available.

NFRs as requirements:

33. FR-M25-033: Kit offline operation >= 8 hours for all five special modes; tested with battery and storage tests on all hardware variants.
34. FR-M25-034: Hospital kit battery life >= 6 hours continuous on supported tablet hardware; battery depleted test run at full-BVI-session load.
35. FR-M25-035: WCAG 2.2 AA for all voter-facing screens; audio guidance mandatory for hospital voter screens; large-text mode available on all special-mode kits.
36. FR-M25-036: DPA 2019 compliance: health status of hospital patients never stored in KIEMS; prison charge data never stored; refugee biometrics not held in KIEMS ABIS without explicit UNHCR agreement; DPIA registered per category.
37. FR-M25-037: Multi-language: EN + SW on all special-mode kit screens; Somali and Oromo added for refugee and IDP facilities in North Eastern and Rift Valley regions.
38. FR-M25-038: SECRECY_VIOLATION event auto-triggered by proximity detection; officer must acknowledge with a note within 5 minutes; unacknowledged event auto-escalates to IEBC Chair; escalation logged.
39. FR-M25-039: Overcounting guard: voter who authenticates at IDP station cannot re-authenticate at original registered station on the same election day; cross-station double-vote check active.

APIs:

40. FR-M25-040: GET /v1/special/facilities and /v1/special/facilities/{id}: JWT+SpiceDB; list and detail of all special voter facilities.
41. FR-M25-041: POST /v1/special/facilities/{id}/events: kit-cert mTLS; idempotent write; submit special voter event from kit.
42. FR-M25-042: POST /v1/special/forms/34 and GET /v1/special/forms/34 and {id}: kit-cert mTLS submission; public read; single form includes Merkle proof.
43. FR-M25-043: POST /v1/special/idp/reassignments: IEBC officer JWT; idempotent write; record IDP voter re-assignment with displacement source.
44. FR-M25-044: GET /v1/special/dashboard: COMMISSION_OPS JWT; ops coordinator live status; map of all facilities; live event feed; violation-event highlight panel.
45. FR-M25-045: GET /v1/special/audit/sample/{category}: IAB_AUDITOR JWT; rate-limited 10/hr; pull random-sample authentication logs for post-election review.
46. FR-M25-046: GET /v1/special/public/counts: public; voter counts per facility; no PII; facility type filter.

UI screens:

47. FR-M25-047: Special voter operations dashboard: map of all special facilities; status badges; live event feed; violation-event highlight panel; one-click field escalation.
48. FR-M25-048: Category-specific kit UI: mode identifier banner colour-coded (orange remand, blue hospital, green barracks, purple IDP); otherwise identical to standard BVI UX to minimise officer training delta.
49. FR-M25-049: Hospital mobile kit UI: large-text default; audio guidance enabled by default; one-handed tablet horizontal operation; "call for assistance" soft button triggers presiding officer alert.
50. FR-M25-050: Witness signature screen: sequential signature collection from presiding officer then each party agent; biometric tap per signer; waiting-status count displayed; cannot proceed until all required signatures collected.
51. FR-M25-051: Violation event logger screen: auto-triggered on SECRECY_VIOLATION event; mandatory officer acknowledgement with typed note; countdown to auto-escalation shown; cannot dismiss without note.
52. FR-M25-052: IDP re-assignment centre console: IEBC officer search by voter ID or name; current station assignment shown; displacement source dropdown; new IDP facility confirmed; re-assignment logged and signed.

## Acceptance criteria

AC-M25-01: 100% of eligible remand facilities offered the vote per gazette schedule; facility log compared against gazette; IEBC Special Voter Coordinator sign-off.
AC-M25-02: >= 95% of hospital voter sessions completed per published schedule; session completion log; IEBC Ops sign-off.
AC-M25-03: 100% of KDF ballot-secrecy violation events reviewed by IAB within 24 hours; violation event log; IAB sign-off.
AC-M25-04: >= 98% of special Form 34 variants published within 30 minutes of close; timestamp delta measurement; Datacraft Technical sign-off.
AC-M25-05: 100% of IAB 5% audit sample reviewed by T+7; audit completion log; IAB Special Voter Auditor sign-off.
AC-M25-06: 100% of IDP re-assignments reflected in correct constituency tally; aggregation cross-check confirmed by Returning Officer.
AC-M25-07: 100% of special-category events in Merkle chain within 60 minutes; Merkle reconciliation at T+1 confirms no gaps.
AC-M25-08: Dual biometric sign-in enforced for REMAND and BARRACKS kit modes; unit test confirms no single-biometric activation path; UAT by IEBC officer.
AC-M25-09: REFUGEE_VOTING_ENABLED flag defaults to false; activation requires IEBC Chair HSM tap and signed config bundle; security test confirms no API path to activate without HSM.
AC-M25-10: WCAG 2.2 AA independent audit pass on voter-facing kit screens and public special-voter-counts page; screen-reader full-task-completion verified.
AC-M25-11: Hospital kit offline operation >= 6 hours on all provisioned tablet models; battery depletion test at full BVI session load confirms no data loss.
AC-M25-12: Overcounting guard prevents same voter authenticating at both original and IDP station on election day; integration test with synthetic roll confirms rejection at second station.
AC-M25-13: Special-category voter count published at T-14 with no PII; automated field scanner and manual review confirm zero patient or prisoner names in response.
AC-M25-14: SECRECY_VIOLATION event auto-escalates to IEBC Chair within 5 minutes of no officer acknowledgement; chaos test removes officer session to verify escalation path.
AC-M25-15: Special-category backend available >= 99.9% during T-7 to T+7 election week; SLO burn-rate alert fires before budget exhausted.

## Applicable edge cases

EC-CON-01: Hospital kit fully offline during patient rounds; 8-hour local queue; signed+sequenced; sync on return to base within 60 minutes of reconnect; no ballot data lost.
EC-CON-03: Remote remand prison with 2G only; kit uses signed SMS fallback for form transmission; USSD for status acknowledgement.
EC-CON-05: Flapping link during special Form 34 submission; idempotent write with form-hash key; cached acknowledgement returned; no duplicate form.
EC-DAT-01: Duplicate special Form 34 submission from kit on reconnect; cached response returned; no re-execution of aggregation.
EC-DAT-02: Same idempotency key with different Form 34 body (corrected tally attempt); 422 idempotency-key-reuse error; amendment flow required.
EC-DAT-11: Spoilt > valid or totals > registered at special facility; block with reason code; Returning Officer override creates DisputeCase with dual-officer sign-off.
EC-SEC-01: Signature mismatch on special Form 34 received from kit; form never applied to aggregation; tamper topic event emitted.
EC-SEC-02: Kit tamper seal broken in transit to facility; quarantine stream; RFID custody event raised; ECC alerted.
EC-SEC-05: Injection in violation event description or IDP re-assignment reason field; parameterised storage only; no dynamic query.
EC-SEC-16: Hospital patient health status or prisoner charge data leaked to log lines; CI log-scanner fixture blocks deployment.
EC-IDN-02: JWT expires mid special-case officer action; in-flight request completes; next request returns 401 with silent refresh.
EC-IDN-08: Duress finger during BVI at special facility; covert ECC alert; UI identical for voter; session continues normally.
EC-IDN-10: SpiceDB down; fail closed on special-facility event write; public counts read-only continues with cached data.
EC-SCL-04: Kafka consumer lag > 60 s on special-voter-event topic; page alert triggered; backpressure applied; no OOM.
EC-LEG-03: s.44A invoked for hospital patient, amputee voter, or sedated voter; dual party-agent witness required; reason code MEDICAL_INABILITY logged and anchored.
EC-OPS-01: Kit battery death mid-session at hospital; journal-before-act pattern; exact resume at same patient step; no double ballot issued.
EC-OPS-06: Facility relocation due to flood or curfew; HSM-signed geofence update required; portal notice published; kit refuses operation at new location until signed bundle acked.
EC-OPS-07: Presiding officer absent at special facility; ECC time-boxed remote co-sign; old credential revoked exactly once; new assignment logged.
EC-HUM-01: Worn fingerprints in manual labourers at barracks or prison; multi-finger retry with adaptive quality threshold; s.44A fallback with reason code and party-agent witness.
EC-HUM-02: Amputee voter at hospital; face/ID path; biometric_exception event logged; no voter denied on biometric grounds alone.
EC-HUM-05: Assisted voting at hospital; assistant identity and patient consent logged in kit; no secrecy leak path; audio guidance available.
EC-HUM-06: Screen readers on hospital kit UI and public special-voter-counts page; WCAG 2.2 AA; full task completion verified.
EC-HUM-08: Somali and Oromo strings for North Eastern IDP and refugee facilities; no truncation; font fallback stack tested at 150% expansion.
EC-KIT-01: GNSS spoof at remote prison or IDP camp; multi-constellation cross-check + IMU validation; kit refuses operation if geofence mismatch confirmed.
EC-KIT-04: Special category mode switch not propagated to kit; signed bundle acked required before kit activates; unacked kits listed in ECC ops dashboard.

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
