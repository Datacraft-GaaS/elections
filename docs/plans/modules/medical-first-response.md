# M60 medical-first-response

| field | value |
|---|---|
| M-number | M60 |
| Home service | training-svc (ADR-0020) |
| Owning work package | WP-5.4 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M60-medical-first-response.md |

## Purpose

Manages medical incident reporting, dispatch requests, and cluster alerts for election-day medical emergencies at polling stations, integrating with KRCS, MoH, and NDMU partners, with strict no-PHI schema enforcement.

## Surfaces and languages

| surface | technology | path under services/training-svc/ |
|---|---|---|
| Medical incident admin | Django | django/medical/ |
| Mobile incident reporting (M31 integration) | Flutter (via M31 poll-clerk app) | apps/poll-clerk (field in M31) |
| Internal API | Rust (axum) | crates/training-api/ (medical endpoints) |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| MedicalIncident | id, station_id, incident_type (injury/illness/cardiac/other), severity (minor/moderate/critical), reported_at, reporter_id, status (reported/dispatched/resolved/escalated) | C | T1-consensus |
| DispatchRequest | id, incident_id, dispatched_to (KRCS/MoH/NDMU), dispatched_at, eta_minutes, acknowledged_at | C | T1-consensus |
| ClusterAlert | id, county_code, incident_count, alert_threshold, raised_at, resolved_at | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/medical/incidents | 3 accredited | JWT+SpiceDB | yes | report medical incident |
| GET /v1/medical/incidents | 3 accredited | JWT+SpiceDB | n/a | list incidents (county-scoped) |
| POST /v1/medical/incidents/{id}/dispatch | 3 accredited | JWT+SpiceDB | yes | request dispatch |
| PUT /v1/medical/incidents/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | mark resolved |
| GET /v1/medical/cluster-alerts | 3 accredited | JWT+SpiceDB | n/a | active cluster alerts |
| POST /v1/medical/cluster-alerts/{id}/resolve | 3 accredited | JWT+SpiceDB | yes | resolve cluster alert |

OpenAPI: packages/openapi-contracts/training/medical-first-response.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.training.medical_incident_reported.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |
| eel.training.medical_cluster_alert.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |
| eel.training.dispatch_requested.v1 | emits | eel.training.medical.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| KRCS | krcs (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| MoH | moh (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| NDMU | ndmu (tier B double) | integrations/ | yes | circuit breaker; dispatch alert to ECC |
| M16 election-command-centre | internal | crates/training-api/ | yes | advisory; cluster alert forwarded |
| M31 mobile-poll-clerk | internal | crates/training-api/ | yes | hard dependency; incident reporting surface |

## ML and AI

none

## Dependencies

- Modules: M16 (ECC cluster alert), M31 (mobile incident surface)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

1. FR-M60-001: Report medical incident from M31 poll-clerk mobile app.
2. FR-M60-002: Classify incident by type and severity.
3. FR-M60-003: Enforce no-PHI schema: no patient names, IDs, or diagnoses in any field.
4. FR-M60-004: Request dispatch to KRCS, MoH, or NDMU with estimated response time.
5. FR-M60-005: Track dispatch acknowledgement from partner agency.
6. FR-M60-006: Resolve incident with resolution notes (no PHI).
7. FR-M60-007: Raise cluster alert when incident count in a county exceeds threshold within time window.
8. FR-M60-008: Forward cluster alert to M16 ECC.
9. FR-M60-009: Resolve cluster alert when incident rate drops below threshold.
10. FR-M60-010: Admin list of incidents with filter by county, severity, status.
11. FR-M60-011: Admin dashboard with incident heatmap by county.
12. FR-M60-012: Alert operations when critical incident not dispatched within 5 min.
13. FR-M60-013: Audit trail of all incident and dispatch transitions.
14. FR-M60-014: Schema-enforced no-PHI; Semgrep rule and test fixtures confirm.
15. FR-M60-015: County-scoped access; no cross-county incident visibility without national role.

## Acceptance criteria

1. AC-M60-01: PHI schema enforcement: any field containing name/ID/diagnosis rejected at API boundary; Semgrep + fixture test.
2. AC-M60-02: Critical incident alert to ECC within 60 s of reporting if not dispatched; integration test.
3. AC-M60-03: Cluster alert raised correctly at threshold; property test with synthetic incident stream.
4. AC-M60-04: Dispatch request idempotent; double-fire test.
5. AC-M60-05: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Security:**
- EC-SEC-16: PHI must not appear in logs or traces; log-scanner fixture.
- EC-SEC-05: injection on incident description fields.

**Data:**
- EC-DAT-01: duplicate incident report; idempotent.
- EC-DAT-10: zero incidents in county; cluster alert not raised; no division errors.

**Connectivity:**
- EC-CON-05: flapping KRCS/MoH/NDMU APIs; backoff; alert ECC on persistent failure.
- EC-MOB-01: OS kills M31 mid-incident report; resumable; no duplicate.

**Human:**
- EC-HUM-05: assisted voting incident (accessibility need); incident type captures need without PHI.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Incident FSM, cluster alert threshold, no-PHI guard | nextest | M60-UNIT- |
| property | Cluster threshold boundary; idempotency | proptest | M60-PROP- |
| contract | KRCS/MoH/NDMU doubles; M16 ECC alert; M31 mobile | Pact | M60-PACT- |
| integration | Full incident-dispatch-resolve lifecycle | nextest + testcontainers | M60-INT- |
| e2e | Admin dashboard; dispatch workflow | Playwright | M60-E2E- |
| security | Semgrep PHI rules; ZAP; log scanner | ZAP + Semgrep | M60-SEC- |

Tags: `#[trace("FR-M60-003","AC-M60-01","EC-SEC-16")]`

## Runbooks and dashboards

- docs/runbooks/training-svc-degradation.md (extend with medical section)
- docs/runbooks/medical-cluster-alert-response.md (new)
- platform/observability/training-svc/medical-dashboard.json
- platform/observability/training-svc/slo-alerts.yaml

## Open questions

- MoU status with KRCS, MoH, and NDMU for API integration; doubles gate development until MoUs signed.
- Cluster alert threshold values (incident count and time window) subject to operational policy from medical coordination committee.
