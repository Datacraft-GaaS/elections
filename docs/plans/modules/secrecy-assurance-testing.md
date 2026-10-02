# M61 secrecy-assurance-testing

| field | value |
|---|---|
| M-number | M61 |
| Home service | ai-platform-svc (ADR-0020) |
| Owning work package | WP-4.5 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M61-secrecy-assurance-testing.md |

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

Engagement registry and setup:

1. FR-M61-001: Create red-team engagement with type (physical_coercion, kit_spoofing, rts_metadata, party_agent_app, full_stack), target modules, staging clone ID, scope description, and red-team lead (FR-M61-01).
2. FR-M61-002: Enforce hard isolation: staging clone namespace has no network path to production systems; verified by k8s NetworkPolicy test before activation.
3. FR-M61-003: Require rules-of-engagement signed by Datacraft Security Lead and Commission CISO before engagement activation (FR-M61-01).
4. FR-M61-004: Activate engagement; provision red-team credentials scoped to staging clone only.
5. FR-M61-005: Track engagement lifecycle: planned, active, report_pending, remediation, attested, closed.
6. FR-M61-006: Maintain attack surface taxonomy: physical_coercion, kit_spoofing, rts_metadata_leak, party_agent_app, full_stack; each entry maps to the constitutional or statutory obligation it threatens (FR-M61-02).
7. FR-M61-007: Version-control taxonomy; update with each new engagement.
8. FR-M61-008: Enforce minimum testing cadence: quarterly for M01-M10 kit stack, monthly from T-90 for M30 party agent app (FR-M61-05).
9. FR-M61-009: Escalate overdue cadence to Commission CISO via automated alert.

Finding management:

10. FR-M61-010: Submit finding with severity (critical, high, medium, low), affected module list, attack surface taxonomy tag, and proof-of-concept blob reference (encrypted, hash-linked, not plain text) (FR-M61-03).
11. FR-M61-011: Assign finding to module owner with severity-based remediation deadline: critical 48 h, high 7 days, medium 30 days, low 90 days (FR-M61-04).
12. FR-M61-012: Alert Commission CISO at 50% and 100% of remediation deadline if finding not yet in remediation_in_progress (NFR-M61-04).
13. FR-M61-013: Generate hourly alerts for critical findings that have breached the 48-hour deadline.
14. FR-M61-014: Module owner updates remediation status; Datacraft Security Lead verifies fix with a dedicated re-test run.
15. FR-M61-015: Re-test result logged with evidence hash; finding advances to verified_closed only after re-test passes.
16. FR-M61-016: Accepted-risk finding requires dual sign-off from Commission CISO and IAB Technical Member before status changes (FR-M61-03).
17. FR-M61-017: ODPC notification record created for any accepted-risk finding with high privacy impact.
18. FR-M61-018: Commission DPO annotates findings implicating personal data with privacy-impact rating (none, low, high) (FR-M61-09).
19. FR-M61-019: Finding lifecycle enforced: open, assigned, remediation_in_progress, remediation_complete, verified_closed, accepted_risk.
20. FR-M61-020: Proof-of-concept blob stored encrypted in object store; accessible only to Datacraft Security Lead and Commission CISO (NFR-M61-01).
21. FR-M61-021: Finding summaries (no PoC detail) visible to assigned module owners and IAB Technical Members.

RTS metadata analysis and kit-spoofing simulation:

22. FR-M61-022: Run continuous RTS metadata analysis after each RTS simulation exercise and after each M10 configuration change (FR-M61-07).
23. FR-M61-023: RTS metadata analysis result (pass/fail) logged with evidence hash; any failure generates an immediate critical finding.
24. FR-M61-024: Conduct periodic kit-spoofing simulation against staging kits only; results logged with PoC hash (FR-M61-08).
25. FR-M61-025: LiteLLM probe generation runs only on staging clone; production API endpoint unreachable from engagement namespace; network policy enforced.

IAB attestation:

26. FR-M61-026: Generate IAB attestation report at T-30 and T-7: all findings by severity, remediation status, accepted-risk justifications, testing coverage per module (FR-M61-06).
27. FR-M61-027: Block IAB attestation until all critical and high findings are remediated or accepted-risk with dual sign-off.
28. FR-M61-028: IAB Technical Member counter-signs attestation; attestation HSM-signed via P06 and Merkle-anchored in P05.
29. FR-M61-029: Any critical finding open at T-7 triggers mandatory IAB escalation event.
30. FR-M61-030: Attestation hash stored in P05 audit log and forwarded to M15 IAB Dashboard (FR-M61-06).

Audit, admin, and operations:

31. FR-M61-031: Immutable audit trail in P05 for all engagement state transitions, finding lifecycle events, re-tests, and attestation sign-offs with actor, timestamp, and data hash (NFR-M61-02).
32. FR-M61-032: Security dashboard: open findings by severity, overdue remediations, RTS analysis last-run status, next scheduled engagement, T-30 and T-7 attestation countdown.
33. FR-M61-033: SpiceDB policy: Red Team Operator scoped to assigned engagement findings only; cannot access findings from other engagements.
34. FR-M61-034: Close engagement after attestation; staging clone access revoked; clone namespace torn down.

## Acceptance criteria

1. AC-M61-01: Staging clone has no network path to production; k8s NetworkPolicy integration test blocks any probe to production endpoints; engagement activation blocked if any reachable path detected.
2. AC-M61-02: IAB attestation blocked when any critical or high finding is unresolved; state-machine integration test confirms 422 returned until all such findings reach verified_closed or accepted_risk with dual sign-off.
3. AC-M61-03: Attestation hash matches SHA-256 of engagement summary content; tamper-detectable; comparison test rejects any hash not matching recomputed digest.
4. AC-M61-04: Critical finding alert to Commission CISO within 60 s of submission; integration test with Pact CISO notification double; alert timestamp verified.
5. AC-M61-05: Remediation deadline alert fires at 50% and 100% of deadline for unresolved findings; timed integration test with simulated clock advancement.
6. AC-M61-06: Dual sign-off (Commission CISO + IAB Technical Member) required before finding moves to accepted_risk; SpiceDB policy test blocks unilateral acceptance.
7. AC-M61-07: Critical finding open at T-7 generates mandatory IAB escalation event published to eel.ai.redteam.v1; e2e test with M15 double confirms receipt.
8. AC-M61-08: RTS metadata analysis triggered automatically on every M10 configuration-change event within 60 s; integration test with M10 double emitting config-change events.
9. AC-M61-09: PoC blob inaccessible to module owners and IAB Technical Members; API returns 403; access-control integration test with each role.
10. AC-M61-10: LiteLLM probe runs only on staging; no production endpoint reachable from engagement namespace; network policy test confirms 0 successful probes to production.
11. AC-M61-11: All engagement state transitions logged in P05 with actor, timestamp, and data hash; audit replay test confirms full reconstruction.
12. AC-M61-12: API p99 within routine 99.5% SLA under load of 100 concurrent red-team sessions; k6 load test.
13. AC-M61-13: ODPC notification record created for accepted-risk finding with high privacy impact; integration test confirms record exists before dual sign-off completes.

## Applicable edge cases

EC-DAT-01: duplicate finding submission with same idempotency key reuses cached response; no second finding record created.
EC-DAT-02: same idempotency key with different finding body returns 422 idempotency-key-reuse.
EC-DAT-04: replayed signed attestation event; rejected on sequence counter; tamper topic published.
EC-DAT-06: schema v1/v2 finding format coexist during rolling deployment; both accepted; Karapace compat check.
EC-DAT-10: zero findings in an engagement; attestation summary correctly renders 0 open and 0 closed; no division errors in coverage calculations.
EC-SEC-01: signature mismatch on consumed attestation event; event never applied; tamper topic published; SRE page within 30 s.
EC-SEC-03: HSM unavailable during attestation PDF signing; sign queue accumulates; attestation signed on restore; degraded-mode flag set on the record.
EC-SEC-04: KEK rotation mid-engagement; PoC blobs re-wrapped online; old encrypted blobs readable until re-wrap completes; audit trail logged.
EC-SEC-05: injection on finding description, component, and taxonomy tag fields; parameterised queries only; ZAP active scan and Semgrep at CI.
EC-SEC-09: rate-limit evasion on finding submission; keyed by red-team operator subject; 429 after burst; priority shed.
EC-SEC-13: split-view on P05 transparency log detected during attestation; witness divergence alert raised; attestation paused until consistency restored.
EC-SEC-14: stolen red-team operator credential; honey mode sandbox served; attacker biometric captured; revoked-cert simulator test.
EC-SEC-16: finding descriptions must not contain production voter data or PII; log-scanner fixture blocks Tier A/B data in telemetry; CI regex.
EC-IDN-02: JWT expires mid-finding submission; in-flight request completes; next request returns 401 and triggers silent refresh.
EC-IDN-07: SIM swap on red-team operator OTP account; no SMS OTP accepted for accredited operators; device-bound credential only; ASVS V2.
EC-IDN-09: dual-control Commission CISO plus IAB approver, one approver revoked mid-attestation; both re-validated at commit; attestation blocked until both active.
EC-IDN-10: SpiceDB down; writes fail closed; PoC blob reads blocked; 503 problem+json returned.
EC-SCL-04: Kafka consumer lag > 60 s on redteam topic; SRE paged; backpressure applied; no OOM.
EC-ML-05: PII in LiteLLM probe generation prompts on staging; PIIStrippingEvent emitted before external call; regression corpus test.
EC-ML-06: unsigned probe model pushed to staging kit; cosign verification required; kit refuses unsigned push; alert generated.
EC-CON-05: flapping M10 event-stream trigger for RTS metadata analysis; backoff-with-jitter; retry budget; cadence alert if trigger missed.
EC-LEG-03: s.44A invoked during active red-team engagement; engagement continues on staging clone; production s.44A event not accessible from engagement namespace.
EC-OPS-09: wrong degradation mode activated for M61 itself; dual sign-off required to enable; fully logged; reversible.
EC-OPS-10: PITR restore during active engagement; restore logged to tlog; engagement status preserved; finding states recoverable from event log.
EC-HUM-06: IAB Technical Member uses screen reader on attestation UI; full task completion at WCAG 2.2 AA; axe + manual test.

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
