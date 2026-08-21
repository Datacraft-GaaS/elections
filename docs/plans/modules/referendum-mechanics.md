# M49 referendum-mechanics

| field | value |
|---|---|
| M-number | M49 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M49-referendum-mechanics.md |

## Purpose

Configures and operates referendum elections under Article 255-257, managing county and national YES/NO tallying with the 24-of-47 county threshold, kit OTA mode switching, and declaration handoff to M51. Includes TLA+ verified threshold logic.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Admin referendum configuration | Django | django/governance-admin/ |
| Public referendum tracker | Astro + LiveView | astro/governance-public/, ex/governance_live/ |
| Internal API | Rust (axum) | crates/referendum/ |
| TLA+ formal model | TLA+ / TLC | docs/tla/Referendum.tla |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| Referendum | id, election_id, question_text (multilingual), status (draft/active/closed/declared), county_threshold (24), national_majority (50%+1), scheduled_date | C | T1-consensus |
| ReferendumQuestion | id, referendum_id, language_code, question_text, yes_label, no_label | C | T1-consensus |
| CountyResult | id, referendum_id, county_code, yes_votes, no_votes, registered_voters, status (provisional/certified), certified_at | C | T1-consensus |
| NationalResult | id, referendum_id, total_yes, total_no, counties_passed, meets_county_threshold, meets_national_majority, computed_at | C | T1-consensus |
| KitModeBundle | id, referendum_id, signed_bundle_hash, distributed_at, ack_count | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/referendums | 1 signed-static | public | n/a | list referendums |
| GET /v1/referendums/{id} | 2 live-per-user | public | n/a | referendum status and results |
| POST /v1/referendums | 3 accredited | JWT+SpiceDB | yes | create referendum config |
| PUT /v1/referendums/{id}/activate | 3 accredited | JWT+SpiceDB | yes | activate; triggers kit OTA mode bundle |
| POST /v1/referendums/{id}/county-results | 3 accredited | JWT+SpiceDB | yes | submit county result |
| PUT /v1/referendums/{id}/county-results/{county}/certify | 3 accredited | JWT+SpiceDB | yes | certify county result |
| GET /v1/referendums/{id}/national-result | 2 live-per-user | public | n/a | national aggregate |
| POST /v1/referendums/{id}/declare | 3 accredited | JWT+SpiceDB | yes | trigger declaration via M51 |

OpenAPI: packages/openapi-contracts/governance/referendum-mechanics.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.referendum_activated.v1 | emits | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |
| eel.governance.county_result_certified.v1 | emits | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |
| eel.governance.referendum_threshold_met.v1 | emits | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |
| eel.governance.referendum_threshold_failed.v1 | emits | eel.governance.referendum.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | exactly-once |
| eel.rts.form34a_captured.v1 | consumes | eel.rts.form34a.v1 | event-source | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M08 RTS kit (mode switch) | internal | crates/referendum/ | yes | signed bundle queued until ack |
| M10 rts-backend (tally source) | internal | crates/referendum/ | yes | circuit breaker; provisional from last snapshot |
| M11 public-portal | internal | crates/referendum/ | yes | advisory; portal reads national-result endpoint |
| M51 result-declaration-certification | internal | crates/referendum/ | yes | hard dependency for declaration |
| M52 election-calendar-gazette | internal | crates/referendum/ | yes | advisory; gazette schedule update |

## ML and AI

none

## Dependencies

- Modules: M08 (kit mode switch), M10 (tally feed), M11 (portal), M51 (declaration), M52 (gazette)
- Platform: P01 (SpiceDB), P04 (Kafka outbox), P06 (HSM for kit bundle signing)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs
- Formal model: docs/tla/Referendum.tla (TLC in CI per ADR DoD item 14)

## Feature checklist

1. FR-M49-001: Create referendum configuration with question text in 11 languages.
2. FR-M49-002: Configure county threshold (default 24 of 47) and national majority (50%+1).
3. FR-M49-003: Validate county and national threshold rules via TLA+ model before activation.
4. FR-M49-004: Activate referendum and trigger HSM-signed kit OTA mode bundle to switch kits.
5. FR-M49-005: Track kit mode bundle acknowledgement across all deployed kits.
6. FR-M49-006: Alert ECC on unacknowledged kit mode switches (EC-KIT-04).
7. FR-M49-007: Receive county YES/NO results from RO via M10 feed.
8. FR-M49-008: Certify county result with RO signature.
9. FR-M49-009: Compute national aggregate in real time as counties certify.
10. FR-M49-010: Enforce exact 24-of-47 county threshold (no rounding; TLA+ invariant).
11. FR-M49-011: Display real-time county-by-county tracker on public portal.
12. FR-M49-012: Block declaration if county threshold not met; show reason.
13. FR-M49-013: Trigger declaration via M51 when both thresholds satisfied.
14. FR-M49-014: Generate national result summary signed by Commission.
15. FR-M49-015: Public tracker shows YES/NO percentages per county without PII.
16. FR-M49-016: Admin can correct county result via compensating event with dual approval.
17. FR-M49-017: Petition window tracking delegated to M51 post-declaration.
18. FR-M49-018: Support multiple simultaneous referendum questions (e.g., constitutional + initiative).
19. FR-M49-019: Gazette notice for referendum schedule published via M52.
20. FR-M49-020: i18n of question text mandatory before activation; all 11 languages required.
21. FR-M49-021: WCAG 2.2 AA on public tracker.
22. FR-M49-022: County result corrections emit compensating events; original immutable.
23. FR-M49-023: Audit trail of all threshold computation changes with version number.
24. FR-M49-024: Off-by-one boundary tests: 23, 24, 25 counties; property test.
25. FR-M49-025: Presidential run-off logic: declaration blocked if national majority not met.

## Acceptance criteria

1. AC-M49-01: 24-of-47 county threshold computed exactly; TLA+ model checks in CI; boundary property test at 23, 24, 25 (EC-LEG-07).
2. AC-M49-02: Kit mode bundle signed by HSM; unacknowledged bundles listed in ECC within 1 min (EC-KIT-04).
3. AC-M49-03: Declaration blocked in M51 when threshold not met; e2e test.
4. AC-M49-04: County result certified event emitted exactly once per county; idempotency test.
5. AC-M49-05: national-result endpoint p95 200 ms at election-critical 99.99% SLA under load.
6. AC-M49-06: All 11 language question texts required before activation; API returns 422 if missing.
7. AC-M49-07: Compensating event issued for county result correction; original event immutable; kani append-only test.
8. AC-M49-08: RTO ≤30 min, RPO ≤5 min in DR drill.

## Applicable edge cases

**Legal:**
- EC-LEG-07: referendum 24-of-47 county threshold off-by-one; exact semantics enforced; TLA+ + boundary tests.
- EC-LEG-05: presidential run-off analogy; declaration blocked; run-off state created.
- EC-LEG-04: recount ordered; compensating event; both versions shown with proofs.
- EC-LEG-02: court-ordered postponement; signed calendar amendment; kits refuse to open.

**Time:**
- EC-TIM-01: kit clock skew; refuse session until NTS/GNSS time.
- EC-TIM-04: DST on diaspora kits; UTC storage, gazetted TZ display.

**Scale:**
- EC-SCL-07: 250k check-ins, 60k RTS, 18k BVI at election-critical load; SLAs met.
- EC-SCL-04: consumer lag on county result topic; backpressure; no OOM.

**Data:**
- EC-DAT-11: county totals exceed registered voters; block with reason; TLA+ invariant.
- EC-DAT-10: zero results on close day; valid; no division by zero.

**Kit:**
- EC-KIT-04: mode switch not propagated; signed bundle acked; unacked listed in ECC.

**Security:**
- EC-SEC-01: signature mismatch on county result; tamper topic; page.
- EC-SEC-03: HSM unavailable; kit bundle signs queue in degraded mode.

**Connectivity:**
- EC-CON-04: site partition primary/mirror; Patroni fencing; RPO ≤5 min.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | County threshold FSM, aggregate computation, question language validation | nextest | M49-UNIT- |
| property | 24-of-47 boundary; county result idempotency; zero-result handling | proptest | M49-PROP- |
| formal | Referendum.tla TLA+ spec; TLC in CI | TLC | M49-TLA- |
| contract | M10 tally feed; M51 declaration trigger | Pact | M49-PACT- |
| integration | Full referendum lifecycle with testcontainers | nextest + testcontainers | M49-INT- |
| e2e | Public tracker; admin certify; declare flow | Playwright | M49-E2E- |
| fuzz | County result payload; question text fields | cargo-fuzz | M49-FUZZ- |
| load | National result endpoint at 5M/hr election-critical load | k6 + Gatling | M49-LOAD- |
| chaos | Kafka ISR shrink during county result ingest; site partition | Chaos Mesh | M49-CHAOS- |
| security | ZAP; HSM failure degraded mode; log scanner | ZAP + Semgrep | M49-SEC- |
| i18n | 11 locales; question text completeness gate | custom | M49-I18N- |
| a11y | axe on public tracker | axe-core | M49-A11Y- |

Tags: `#[trace("FR-M49-010","AC-M49-01","EC-LEG-07")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with referendum section)
- docs/runbooks/referendum-kit-mode-switch.md (new)
- platform/observability/governance-svc/referendum-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml (election-critical burn rate)

## Open questions

- Multiple simultaneous referendum questions (Article 256 + citizens initiative on same day): whether single Referendum entity supports question lists or one entity per question.
- Whether M49 owns the poll-day BVI/RTS orchestration for referendum or delegates entirely to M08/M10; scope boundary with rts-svc needs clarification.
