# M32 mobile-field-support

| field | value |
|---|---|
| M-number | M32 |
| Home service | field-mobile-svc (ADR-0020) |
| Owning work package | WP-4.9 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M32-mobile-field-support.md |

## Purpose

Give every Datacraft field support engineer (~200 on election day) a mobile console for their assigned county or cluster of polling stations: support-ticket queue with SLA timers, remote kit diagnostics, KEBS and warranty-claim initiation, escalation to Datacraft L2/L3 and Mechrevo OEM and IEBC ICT, on-call rotation management, and end-of-shift signed reporting.

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Field Support App (Android 10+ / iOS 15+) | Flutter | services/field-mobile-svc/apps/field-support/ |
| API | Rust (axum) | services/field-mobile-svc/crates/field-mobile-api/ |

Distribution: private via IEBC MDM (Datacraft device roster). Languages: EN + SW.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| SupportTicket | id, station_code, kit_serial, reporter_role, severity (SEV1/SEV2/SEV3), title, description, status, assigned_engineer_id, sla_target_at, resolved_at, escalation_level | B | T1-consensus |
| DiagnosticAction | id, ticket_id, kit_serial, action_type (FETCH_LOGS/REBOOT/FIRMWARE_ATTEST/SELF_TEST), engineer_id, kit_consent_recorded, signed_at, result | A | T1-audit |
| WarrantyClaim | id, kit_serial, oem_code, fault_description, claim_status, engineer_id, submitted_at | B | T1-consensus |
| ShiftReport | id, engineer_id, cluster_id, shift_start, shift_end, tickets_handled, sla_compliance_pct, escalations, spares_consumed, engineer_signature, submitted_at | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/field/field-support.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/support/tickets | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | live ticket queue filtered by cluster + severity |
| POST /v1/support/tickets/{id}/take | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | take / assign ticket |
| POST /v1/support/tickets/{id}/escalate | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | escalate to L2/L3/Mechrevo/IEBC ICT |
| GET /v1/support/kits/{serial} | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | kit health telemetry |
| POST /v1/support/kits/{serial}/action | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | remote diagnostic action (signed, audited, kit-consent required) |
| POST /v1/support/warranty | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | initiate warranty claim |
| GET /v1/support/spares | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | spares availability in cluster via M20 |
| GET /v1/support/oncall | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | n/a | on-call rotation schedule |
| POST /v1/support/handoff | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | shift boundary handoff; sign-off required |
| POST /v1/support/shift-report | 2 | JWT+SpiceDB (DATACRAFT_ENGINEER) | yes | submit signed end-of-shift report |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.field.support_ticket_escalated.v1 | emits | eel.field.support.v1 | operational | yes | exactly-once |
| eel.field.diagnostic_action.v1 | emits | eel.field.support.v1 | operational | yes | at-least-once idempotent |
| eel.field.warranty_claim.v1 | emits | eel.field.support.v1 | operational | yes | at-least-once idempotent |
| eel.field.shift_report.v1 | emits | eel.field.support.v1 | operational | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| O02 Support / Ticketing | O02 | integrations/ticketing/ | yes | ticket lifecycle back-end; circuit breaker; queue |
| M20 Inventory Management | internal | integrations/inventory/ | yes | spares lookup and spare-part dispatch |
| O01 MDM | O01 | integrations/mdm/ | yes | private app distribution; remote device management |
| O04 Warranty | O04 | integrations/warranty/ | yes | warranty claim initiation and status tracking |
| M06 BVI Back-End | internal | integrations/bvi/ | yes | kit operational status; incident cross-reference |
| M16 Election Command Centre | internal | integrations/ecc/ | yes | ECC sees engineer location and ticket status; dispatch visibility |

## ML and AI

- None.

## Dependencies

- Modules: M06, M16, M20
- Platform: none
- Packages: kit-sdk-flutter, eel-resilience-rs

## Feature checklist

1. FR-M32-001: enrolment via Datacraft staff ID + biometric + device attestation
2. FR-M32-002: cluster assignment: engineer bound to counties / constituencies for the election
3. FR-M32-003: MFA sign-in: biometric + PIN + device attestation
4. FR-M32-004: live ticket queue filtered by cluster, severity, age; refresh <= 2 s over LTE
5. FR-M32-005: SEV 1 / 2 / 3 SLA timers visible (2 h / 4 h / 6 h response; 1 d / 2 d / best-effort resolution)
6. FR-M32-006: take / assign / re-assign ticket with dual notification
7. FR-M32-007: in-app messaging with reporter (PO or IEBC County ICT)
8. FR-M32-008: attach evidence: photo, short video, diagnostic log capture
9. FR-M32-009: kit lookup by serial number or QR scan
10. FR-M32-010: live kit health telemetry: battery, connectivity, GPS fix, temperature, error counts, last-seen
11. FR-M32-011: remote diagnostic actions (policy engine + kit consent required): fetch logs, request reboot, request firmware attestation, trigger self-test
12. FR-M32-012: all remote actions signed and audited; kit-side user visible confirmation for privileged actions
13. FR-M32-013: warranty status and spares availability check via M20 and O04
14. FR-M32-014: file warranty claim: OEM identified via kit metadata; ticket sent to Mechrevo and OEM
15. FR-M32-015: track warranty claim status
16. FR-M32-016: coordinate spare-part shipment / hot-swap kit dispatch via M20 IMS API
17. FR-M32-017: one-tap escalation to Datacraft L2, L3, Mechrevo, OEM, IEBC ICT per escalation matrix
18. FR-M32-018: escalation carries ticket context and attachments
19. FR-M32-019: escalation logged permanently
20. FR-M32-020: on-call schedule per engineer visible
21. FR-M32-021: push notification on new ticket assignment or SLA breach approaching
22. FR-M32-022: shift handoff to next on-call at shift boundary; sign-off required from both engineers
23. FR-M32-023: diagnostic runbook library indexed by symptom; searchable offline
24. FR-M32-024: contact card for PO / RO / county support desk / war-room hotline
25. FR-M32-025: kit reset and recovery step-by-step guidance
26. FR-M32-026: spares inventory in reach of engineer; mapped to nearest spare-holding constituency store via M20
27. FR-M32-027: automatic end-of-shift report: tickets handled, SLA compliance, escalations, spares consumed, incidents
28. FR-M32-028: shift report signed and submitted to Datacraft Ops and IEBC ICT
29. FR-M32-029: runbook library fully offline
30. FR-M32-030: ticket queue read-only offline (last-sync)
31. FR-M32-031: diagnostic actions queue if kit-gateway unreachable; retry on connectivity
32. FR-M32-032: incidents and reports queued locally; submitted on reconnect
33. Battery cost <= 5% per hour foreground; certificate pinning + device attestation mandatory; screen-capture protection on kit-diagnostic screens; no PII in logs

## Acceptance criteria

1. AC-M32-01: end-to-end ticket cycle completed within SLA on real device; UAT
2. AC-M32-02: remote diagnostic action signed and audited; audit trace
3. AC-M32-03: escalation matrix reachable and correct; bench test
4. AC-M32-04: offline runbook search; airplane-mode test
5. AC-M32-05: end-of-shift report signed and submitted; UAT

## Applicable edge cases

**connectivity**
- EC-CON-01: engineer phone offline; runbook fully offline; ticket queue read-only from last-sync; diagnostic actions queued; SMS fallback for escalation via special short-code

**operations**
- EC-OPS-02: kit unreachable for diagnostic action; queue diagnostic; escalate after N minutes per SLA
- EC-OPS-04: warranty claim rejected by OEM; auto-escalate to Mechrevo regional contact
- EC-OPS-06: SLA breach on SEV 1; automatic escalation cascade; alert Datacraft regional lead and IEBC ICT

**mobile**
- EC-MOB-02: engineer's mid-tier Android 10; battery <= 5% per hour; runbook search on-device
- EC-MOB-06: mTLS + certificate pinning; attestation hard-fail

**scale**
- EC-SCL-01: 200 engineers filing shift reports simultaneously at end of day; back-end worker handles burst

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | SLA timer, escalation matrix routing, diagnostic action audit, spares lookup | nextest + flutter_test | M32-UNIT- |
| contract | ticketing O02, inventory M20, warranty O04, BVI M06 | Pact | M32-PACT- |
| integration | full ticket cycle; remote diagnostic with kit-consent; warranty claim; shift handoff | integration_test | M32-INT- |
| e2e | engineer check-in, take ticket, run diagnostic, escalate, shift report | Flutter driver | M32-E2E- |
| security | all diagnostic actions signed and audited; no action without kit consent | ZAP + audit-trace | M32-SEC- |
| offline | runbook offline; ticket queue read-only offline | netem | M32-OFFLN- |

Tags: `testWidgets('diagnostic action [FR-M32-011,AC-M32-02]')`

## Runbooks and dashboards

- `docs/runbooks/field-mobile-svc-degradation.md` - extend with field-support offline and SLA-breach paths
- Grafana dashboard: `platform/observability/field-mobile-svc/field-support-dashboard.json`
- SLO alerts: `platform/observability/field-mobile-svc/field-support-slo-alerts.yaml` (SEV1 response <= 2 h)

## Open questions

- O02 Support / Ticketing back-end: scope and provider for ticketing system must be confirmed; MeGuard-derived system or commercial ITSM; integration contract required before M32 integration sprint.
- O04 Warranty back-end: Mechrevo OEM API for warranty claim submission; API spec and authentication must be obtained before integration sprint.
- Kit diagnostic action policy engine: who defines the policy (IEBC ICT or Datacraft SOC)? Policy schema must be agreed before remote-action feature is built.
