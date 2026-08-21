# M61 secrecy-assurance-testing

| field | value |
|---|---|
| M-number | M61 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M61-secrecy-assurance-testing.md |

## Purpose

Manages adversarial red-team engagements on isolated staging clones to test ballot secrecy assurance, with finding tracking, remediation workflows, and IAB attestations confirming isolation and no information leakage to production systems.

## Surfaces and languages

| surface | technology | path under services/ai-platform-svc/ |
|---|---|---|
| Red-team admin console | Django | django/ai-admin/ |
| Internal API | Python FastAPI | py/secrecy-redteam/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| RedTeamEngagement | id, election_cycle, scope_description, staging_clone_id, status (planned/active/remediation/attested/closed), started_at, closed_at | C | T1-consensus |
| Finding | id, engagement_id, severity (critical/high/medium/low), description, affected_component, remediation_status, found_at | C | T1-consensus |
| IABAttestation | id, engagement_id, attested_by, attestation_hash, no_production_leak_confirmed, attested_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/redteam/engagements | 3 accredited | JWT+SpiceDB | n/a | list engagements |
| POST /v1/redteam/engagements | 3 accredited | JWT+SpiceDB | yes | create engagement |
| PUT /v1/redteam/engagements/{id}/activate | 3 accredited | JWT+SpiceDB | yes | activate on staging clone |
| POST /v1/redteam/engagements/{id}/findings | 3 accredited | JWT+SpiceDB | yes | submit finding |
| PUT /v1/redteam/engagements/{id}/findings/{fid}/remediate | 3 accredited | JWT+SpiceDB | yes | mark finding remediated |
| POST /v1/redteam/engagements/{id}/attest | 3 accredited | JWT+SpiceDB | yes | IAB attestation |
| PUT /v1/redteam/engagements/{id}/close | 3 accredited | JWT+SpiceDB | yes | close engagement |

OpenAPI: packages/openapi-contracts/ai-platform/secrecy-assurance-testing.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.ai.redteam_finding_submitted.v1 | emits | eel.ai.redteam.v1 | operational | no | at-least-once idempotent |
| eel.ai.iab_attestation_issued.v1 | emits | eel.ai.redteam.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M15 IAB dashboard | internal | py/secrecy-redteam/ | yes | advisory; attestation forwarded |
| Staging clone infrastructure | internal (k8s namespace isolation) | platform/environments/staging | yes | hard dependency; engagement blocked if clone unavailable |

## ML and AI

- Red-team uses LiteLLM via ai-platform for automated secrecy-violation probe generation on staging clones; production systems never contacted.

## Dependencies

- Modules: M15 (IAB attestation consumer)
- Platform: P01 (SpiceDB), P04 (Kafka outbox), staging clone namespace isolation
- Packages: eel-outbox-py, eel-resilience-py, eel-web-py

## Feature checklist

1. FR-M61-001: Create red-team engagement scoped to specific staging clone namespace.
2. FR-M61-002: Enforce hard isolation: staging clone has no network path to production.
3. FR-M61-003: Activate engagement; provision red-team access to staging clone only.
4. FR-M61-004: Submit findings with severity and affected component.
5. FR-M61-005: Track remediation status of each finding.
6. FR-M61-006: Block IAB attestation until all critical and high findings are remediated.
7. FR-M61-007: IAB attestation confirms: no production data leaked, isolation intact, findings resolved.
8. FR-M61-008: Attestation hash stored in audit log and forwarded to M15.
9. FR-M61-009: Close engagement after attestation.
10. FR-M61-010: Admin list of engagements with finding summary and status.
11. FR-M61-011: Alert on critical finding within 1 h of submission.
12. FR-M61-012: Audit trail of all engagement state transitions.
13. FR-M61-013: LiteLLM probe generation runs only on staging; production API not reachable from engagement namespace.

## Acceptance criteria

1. AC-M61-01: Staging clone has no network path to production; network policy test.
2. AC-M61-02: IAB attestation blocked when critical/high findings unresolved; test.
3. AC-M61-03: Attestation hash matches engagement summary; tamper-detectable; test.
4. AC-M61-04: Critical finding alert to admin within 60 s; integration test.
5. AC-M61-05: API p99 within routine 99.5% SLA.

## Applicable edge cases

**Security:**
- EC-SEC-16: finding descriptions must not contain production voter data; log-scanner.
- EC-SEC-05: injection on finding description or component fields.

**Data:**
- EC-DAT-01: duplicate finding submission; idempotent.

**ML:**
- EC-ML-05: PII in ML prompts during probe generation; PIIStrippingEvent before LiteLLM call.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Engagement FSM, attestation block logic, finding severity | nextest (pytest) | M61-UNIT- |
| property | Attestation pre-condition invariant | proptest | M61-PROP- |
| contract | M15 IAB attestation event | Pact | M61-PACT- |
| integration | Full engagement lifecycle; isolation check | nextest + testcontainers | M61-INT- |
| e2e | Admin engagement, finding, attestation flows | Playwright | M61-E2E- |
| security | Network policy isolation test; log scanner | k8s netpol + Semgrep | M61-SEC- |

Tags: `#[trace("FR-M61-002","AC-M61-01","EC-SEC-16")]`

## Runbooks and dashboards

- docs/runbooks/ai-platform-svc-degradation.md (extend with redteam section)
- docs/runbooks/redteam-staging-isolation-breach.md (new)
- platform/observability/ai-platform-svc/redteam-dashboard.json
- platform/observability/ai-platform-svc/slo-alerts.yaml

## Open questions

- Isolation breach response: if a finding reveals actual production data access, protocol for incident escalation to ODPC not yet defined.
- Frequency of mandatory engagements before each election cycle: per-quarter or per-election-cycle; policy decision required.
