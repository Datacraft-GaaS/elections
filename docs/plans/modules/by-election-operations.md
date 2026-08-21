# M50 by-election-operations

| field | value |
|---|---|
| M-number | M50 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M50-by-election-operations.md |

## Purpose

Manages the full lifecycle of by-elections triggered by recall (M48), vacancy, or court order, supporting up to 10 concurrent by-elections with register subsets, kit plan, cost reporting, and the short-notice (<21-day) notice flow under Elections Act s.24.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Admin by-election configuration | Django | django/governance-admin/ |
| Public by-election notice portal | Astro | astro/governance-public/ |
| Internal API | Rust (axum) | crates/by-election/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ByElection | id, constituency_id, trigger (recall/vacancy/court), status (notice/planning/active/closed/declared), notice_date, election_date, cost_ceiling_kes | C | T1-consensus |
| RegisterSubset | id, by_election_id, snapshot_id (FK to M02), station_count, voter_count, generated_at | C | T1-consensus |
| KitPlan | id, by_election_id, station_assignments, kit_ids, signed_at | C | T1-consensus |
| CostReport | id, by_election_id, budget_lines, actual_expenditure, variance, submitted_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/by-elections | 1 signed-static | public | n/a | list by-elections with status |
| GET /v1/by-elections/{id} | 2 live-per-user | public | n/a | by-election detail |
| POST /v1/by-elections | 3 accredited | JWT+SpiceDB | yes | create by-election (Commissioner dual-sign) |
| POST /v1/by-elections/{id}/register-subset | 3 accredited | JWT+SpiceDB | yes | generate register subset from M02 |
| POST /v1/by-elections/{id}/kit-plan | 3 accredited | JWT+SpiceDB | yes | create and sign kit plan |
| PUT /v1/by-elections/{id}/activate | 3 accredited | JWT+SpiceDB | yes | activate; triggers M04 register subset notice |
| POST /v1/by-elections/{id}/cost-report | 3 accredited | JWT+SpiceDB | yes | submit cost report |
| GET /v1/by-elections/{id}/cost-report | 3 accredited | JWT+SpiceDB | n/a | cost report |

OpenAPI: packages/openapi-contracts/governance/by-election-operations.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.by_election_created.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_activated.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_declared.v1 | emits | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_threshold_met.v1 | consumes | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M04 VPRS (register notice) | internal | crates/by-election/ | yes | advisory; notice continues |
| M10 rts-backend (tally) | internal | crates/by-election/ | yes | circuit breaker; provisional from snapshot |
| M12 CRMS (candidate nomination) | internal | crates/by-election/ | yes | advisory; nomination window opens |
| M20 inventory-management | internal | crates/by-election/ | yes | advisory; kit allocation |
| M22 campaign-finance | internal | crates/by-election/ | yes | advisory; disclosure window |
| M51 result-declaration | internal | crates/by-election/ | yes | hard dependency for declaration |
| M52 election-calendar-gazette | internal | crates/by-election/ | yes | advisory; gazette notice |
| M55 hr-payroll (cost report link) | internal | crates/by-election/ | yes | advisory |

## ML and AI

none

## Dependencies

- Modules: M04, M10, M12, M20, M22, M51, M52, M55
- Platform: P01 (SpiceDB), P04 (Kafka outbox), P06 (HSM for kit plan signing)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

1. FR-M50-001: Create by-election with trigger type (recall/vacancy/court) and Commissioner dual-sign.
2. FR-M50-002: Enforce <21-day notice flow with accelerated approval when notice_date to election_date < 21 days.
3. FR-M50-003: Generate register subset from M02 snapshot scoped to constituency.
4. FR-M50-004: Sign register subset with HSM; distribute to stations.
5. FR-M50-005: Create kit plan assigning available kits from M20 to stations.
6. FR-M50-006: Sign kit plan with HSM and distribute.
7. FR-M50-007: Support up to 10 concurrent by-elections without resource contention.
8. FR-M50-008: Publish gazette notice for by-election via M52.
9. FR-M50-009: Open nomination window in M12 for constituency candidates.
10. FR-M50-010: Activate by-election; trigger BVR/BVI/RTS mode for constituency.
11. FR-M50-011: Receive tally results from M10 for constituency.
12. FR-M50-012: Trigger declaration via M51.
13. FR-M50-013: Submit cost report with budget lines and actuals.
14. FR-M50-014: Export cost report as OAG-ready CSV/PDF.
15. FR-M50-015: Alert Commission on cost overrun above threshold.
16. FR-M50-016: Public notice portal lists by-elections with status and dates.
17. FR-M50-017: Admin lists register subset with voter count and station breakdown.
18. FR-M50-018: Audit trail of all by-election state transitions.
19. FR-M50-019: By-election creation from recall is idempotent (M48 fires once).
20. FR-M50-020: i18n on public notice portal in 11 languages.
21. FR-M50-021: WCAG 2.2 AA on public surfaces.
22. FR-M50-022: Kit plan tracks acknowledgement from each station kit.

## Acceptance criteria

1. AC-M50-01: Up to 10 concurrent by-elections activate simultaneously without resource deadlock; load test.
2. AC-M50-02: <21-day notice flow routes to accelerated approval; e2e test with mock court order.
3. AC-M50-03: By-election creation from M48 recall event is idempotent; double-fire test.
4. AC-M50-04: Register subset generation scoped exactly to constituency; no extra voters; property test.
5. AC-M50-05: Declaration in M51 triggered correctly; e2e test.
6. AC-M50-06: Cost report export contains all budget lines and actuals; integration test.
7. AC-M50-07: API p99 latency under election-critical 99.99% SLA under load.
8. AC-M50-08: RTO ≤30 min, RPO ≤5 min in DR drill.

## Applicable edge cases

**Legal:**
- EC-LEG-02: court-ordered postponement; signed calendar amendment; kits refuse.
- EC-LEG-01: candidate withdraws post-notice; status annotated; vacancy rules.

**Time:**
- EC-TIM-07: notice window cut short; signed extension.
- EC-TIM-09: register freeze race T-60/T-30; OPA hard-close.

**Scale:**
- EC-SCL-07: concurrent by-elections within election-critical SLA.

**Data:**
- EC-DAT-01: duplicate by-election creation from same recall event; idempotent.

**Security:**
- EC-SEC-01: signature mismatch on kit plan; tamper event.
- EC-SEC-03: HSM unavailable; kit plan queued in degraded mode.

**Connectivity:**
- EC-CON-04: site partition; Patroni fencing; RPO ≤5 min.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ByElection FSM, register subset scoping, concurrency count guard | nextest | M50-UNIT- |
| property | Idempotent creation; subset voter count invariant | proptest | M50-PROP- |
| contract | M48 recall event; M51 declaration; M52 gazette | Pact | M50-PACT- |
| integration | Full by-election lifecycle; concurrent 10 instances | nextest + testcontainers | M50-INT- |
| e2e | Notice portal; admin activate; cost report | Playwright | M50-E2E- |
| load | 10 concurrent by-elections at election-critical load | k6 | M50-LOAD- |
| chaos | Kafka ISR shrink; site partition | Chaos Mesh | M50-CHAOS- |
| security | ZAP; HSM failure degraded; log scanner | ZAP + Semgrep | M50-SEC- |
| i18n | 11 locales | custom | M50-I18N- |
| a11y | axe on public surfaces | axe-core | M50-A11Y- |

Tags: `#[trace("FR-M50-007","AC-M50-01","EC-SCL-07")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with by-election section)
- docs/runbooks/by-election-concurrent-operations.md (new)
- platform/observability/governance-svc/by-election-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml (election-critical burn rate)

## Open questions

- Maximum concurrent by-elections: inventory confirms up to 10 but legal opinion on simultaneous recall + vacancy scenarios needed.
- Whether cost ceiling is set by Treasury or IEBC Budget department; integration with M54 budget-expenditure module.
