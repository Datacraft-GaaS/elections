# M49 referendum-mechanics

| field | value |
|---|---|
| M-number | M49 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | election-critical 99.99% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M49-referendum-mechanics.md |

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

Referendum configuration:
FR-M49-001: Create referendum record with constitutional pathway selector (Art 255, Art 256, Art 257, popular initiative).
FR-M49-002: Store question text in all 11 languages; reject activation if any locale is missing.
FR-M49-003: Configure county threshold (default 24 of 47) and national majority rule (50%+1 of total votes cast).
FR-M49-004: Configure participating county list (all 47 for constitutional; subset for devolved referenda).
FR-M49-005: Validate threshold rules against TLA+ formal model before allowing activation; CI gate blocks activation if TLC fails.
FR-M49-006: Support up to 5 simultaneous referendum questions on a single ballot (multi-question mode, activation gated).
FR-M49-007: Publish gazette notice for referendum schedule via M52 before activation; activation blocked without gazette reference.
FR-M49-008: Store statutory basis citation (e.g. "Constitution Art 255(2)") per referendum and surface it on declaration output.

Kit mode management:
FR-M49-009: Activate referendum and emit HSM-signed OTA kit mode bundle to all deployed kits.
FR-M49-010: Kit mode bundle delivery: all kits must acknowledge within 4 hours of bundle publication.
FR-M49-011: Track per-kit OTA acknowledgement status; surface unacknowledged kits in ECC operations console.
FR-M49-012: Alert ECC when any kit remains unacknowledged at T-2 h and T-30 min before polls open.
FR-M49-013: Block kit from opening polls if referendum mode bundle not confirmed (kit enforces locally).
FR-M49-014: After referendum close, emit OTA reset bundle returning kits to general-election mode; archive referendum data on kit.
FR-M49-015: Kit mode bundle signing uses HSM (P06); degraded-mode queue accumulates if HSM unavailable per EC-SEC-03.

Tally ingest:
FR-M49-016: Ingest Form 37A (polling station) referendum tallies from M10 isolated referendum pipeline; no cross-contamination with general-election streams.
FR-M49-017: Aggregate Form 37A to Form 37B (constituency) automatically; no manual re-entry by RO.
FR-M49-018: Aggregate Form 37B to Form 37C (county); require RO digital signature on 37C before county result is accepted.
FR-M49-019: Ingest Form 37C county result; reject if yes_votes + no_votes + rejected_votes exceeds registered voters (EC-DAT-11).
FR-M49-020: Emit `eel.governance.county_result_certified.v1` exactly once per county per certification; idempotency enforced by outbox deduplication.
FR-M49-021: Store rejected_votes separately from yes and no; include in published county result.
FR-M49-022: Tally pipeline ingest-to-confirmation p99 <= 30 s under election-critical load.

County threshold engine:
FR-M49-023: Evaluate county YES majority per county (yes_votes > no_votes); no tie-break rounding; exact integer comparison.
FR-M49-024: Maintain running count of counties where YES majority is confirmed; emit `eel.governance.referendum_threshold_met.v1` when count reaches county_threshold.
FR-M49-025: Emit `eel.governance.referendum_threshold_failed.v1` when all counties reported and count is below threshold.
FR-M49-026: County threshold engine updated within 60 s of each Form 37C certification.
FR-M49-027: Expose county threshold counter in real time on public tracker ("YES leads in X of 47 counties").
FR-M49-028: Property test: boundary at 23, 24, and 25 counties; invariant: threshold_met = (counties_yes_majority >= county_threshold).
FR-M49-029: TLA+ model (docs/tla/Referendum.tla) checked by TLC in CI; any violated invariant blocks merge.

National result computation:
FR-M49-030: Compute national result automatically once all 47 county Forms 37C are confirmed; surface as NationalResult record.
FR-M49-031: National majority check: threshold_met = total_yes > total_no (50%+1 of valid votes); block declaration if not met.
FR-M49-032: Compute Form 37D (national) hash and Merkle root; anchor via P05; include in NationalResult.
FR-M49-033: National result endpoint serves latest computed aggregate; p95 <= 200 ms, p99 <= 500 ms at election-critical load (5M/hr).
FR-M49-034: National result recomputes on county result correction; versioned; old version retained with proof.
FR-M49-035: Public national result omits all PII; shows only aggregate YES/NO counts, percentage, and county breakdown.

Declaration and M51 handoff:
FR-M49-036: Block POST /v1/referendums/{id}/declare if county threshold or national majority is not met; return 422 with machine-readable reason.
FR-M49-037: Declaration requires Commissioner HSM signature (M-of-N dual approval); POST idempotent.
FR-M49-038: On declaration, emit signed national result snapshot to M51 for certification and to M11 for publication.
FR-M49-039: Trigger M52 gazette notice automatically on successful declaration; include statutory basis, date, county breakdown.
FR-M49-040: Petition window alert: send ECC and legal-team notifications at T-48 h and T-24 h before petition window closes (petition window = 7 days post-declaration per Art 87 / Elections Act s.75).

Public dashboard and portal:
FR-M49-041: Serve live county-by-county YES/NO tracker via SSE push; 500k concurrent public viewers; p99 page load <= 2 s via CDN.
FR-M49-042: 47-county grid shows YES/NO lead badge per county, confirmed status, and running threshold counter.
FR-M49-043: National YES/NO bar chart and turnout display; auto-refresh without full page reload.
FR-M49-044: Public results page shows Form 37A image links with Merkle proof per polling station (same as general election via M11).
FR-M49-045: Public tracker is WCAG 2.2 AA compliant; axe-core scan runs in CI on every deploy.
FR-M49-046: All public-facing referendum strings available in all 11 languages; language toggle present on tracker.

Correction and immutability:
FR-M49-047: County result correction requires dual-approval workflow; original event immutable; compensating event carries amended values.
FR-M49-048: Correction triggers recomputation of national aggregate; new NationalResult version emitted; both versions visible with cryptographic proofs.
FR-M49-049: Admin audit log records who initiated correction, approvers, timestamp, and diff; retained permanently.
FR-M49-050: Corrections after declaration require re-declaration flow through M51; system enforces state machine transition.

Security and integrity:
FR-M49-051: All county result events carry RO digital signature; signature verified on ingest; mismatched events routed to tamper topic (EC-SEC-01).
FR-M49-052: Kit mode bundle integrity: bundle hash verified by kit firmware before applying; tampered bundle rejected and logged.
FR-M49-053: All API inputs validated and parameterised; ZAP + Semgrep + cargo-fuzz gates in CI (EC-SEC-05).
FR-M49-054: Telemetry sanitiser strips Tier A/B PII fields from logs and traces before export (EC-SEC-16).
FR-M49-055: SpiceDB authorisation enforced on all state-mutating endpoints; fail-closed on SpiceDB unavailability (EC-IDN-10).

Observability and operations:
FR-M49-056: OpenTelemetry traces exported for all referendum lifecycle transitions; trace IDs correlate kit OTA, tally ingest, and threshold events.
FR-M49-057: Grafana dashboard `eel-iems-49` exists at OAT gate; alert ruleset `eel-49-alerts` active in Alertmanager.
FR-M49-058: SLO burn-rate alert fires when election-critical 99.99% budget is at risk; page ECC on-call.
FR-M49-059: Circuit breaker on M10 tally feed; on open, serve provisional national result from last confirmed snapshot with staleness label.
FR-M49-060: Kafka consumer lag on `eel.governance.referendum.v1` topic monitored; backpressure applied; OOM prevention verified (EC-SCL-04).

## Acceptance criteria

AC-M49-01: County threshold engine computes exactly county_yes_majority >= 24; TLA+ invariant checked by TLC in CI; property test covers boundary at 23, 24, and 25 counties (EC-LEG-07). Zero tolerance for off-by-one.
AC-M49-02: Kit mode bundle signed by HSM and delivered to all kits within 4 hours of activation; unacknowledged kits listed in ECC operations console within 60 s of bundle publish (EC-KIT-04).
AC-M49-03: POST /declare returns 422 with machine-readable reason when county threshold or national majority is not met; e2e Playwright test asserts blocked state.
AC-M49-04: `eel.governance.county_result_certified.v1` emitted exactly once per county per certification; duplicate-submission replay returns cached 200 with no second event (EC-DAT-01).
AC-M49-05: GET /v1/referendums/{id}/national-result p95 <= 200 ms, p99 <= 500 ms at 5M requests/hr; availability 99.99%; verified by k6 load test in pre-prod.
AC-M49-06: Tally ingest pipeline p99 form-ingest-to-confirmation <= 30 s at election-critical load; measured end-to-end from Form 37C POST to county_result_certified event on Kafka.
AC-M49-07: Public dashboard serves 500k concurrent viewers with p99 page load <= 2 s via CDN; CDN hit ratio >= 95%; k6 + Gatling load test.
AC-M49-08: All 11 language question texts must be present before activation; POST /activate returns 422 listing missing locales if any are absent (FR-M49-002).
AC-M49-09: County result correction emits compensating event; original event immutable; append-only invariant verified by kani proof; both event versions retrievable with cryptographic proof.
AC-M49-10: RTO <= 30 min, RPO <= 5 min; verified by DR drill against isolated referendum schema partition; drill result documented in runbook.
AC-M49-11: Public tracker achieves WCAG 2.2 AA; axe-core zero critical or serious violations on every CI deploy; manual keyboard-navigation pass on declaration screen.
AC-M49-12: Petition window alerts delivered to ECC and legal team at T-48 h and T-24 h with 100% delivery rate; alert system log audited post-declaration.
AC-M49-13: Secrets and PII (Tier A/B fields) absent from all exported logs and traces; log-scanner CI fixture passes; no regex hit on voter ID or biometric fields (EC-SEC-16).
AC-M49-14: SpiceDB authorisation enforced on POST, PUT, and declare endpoints; chaos test (SpiceDB pod killed) confirms writes fail-closed with HTTP 503 and no unauthorised state change (EC-IDN-10).
AC-M49-15: Unit test line coverage >= 80% on county threshold FSM and national result computation (Rust); mutation score >= 60% on threshold boundary logic; lcov report uploaded to CI.

## Applicable edge cases

Legal:
EC-LEG-07: Referendum 24-of-47 county threshold off-by-one; exact integer comparison enforced; TLA+ invariant checked in CI; boundary property test at 23/24/25.
EC-LEG-05: National majority not met (YES <= NO nationally); declaration blocked with statutory reason; system remains in CLOSED state awaiting Commission review.
EC-LEG-04: Recount ordered post-certification; compensating event issued with dual approval; both original and amended result versions retained with Merkle proofs; re-triggers national recomputation.
EC-LEG-02: Court-ordered postponement of referendum; signed calendar amendment via M52; kits receive OTA signal to refuse opening; portal displays postponement notice.
EC-LEG-08: Petition evidence request arrives at T+6d 23h inside the 7-day window; petition window not yet closed; system honours statutory 24 h clock without premature freeze.
EC-LEG-09: Gazette notice auto-trigger races declaration event; idempotent scheduler ensures single gazette referral even if declaration event fires twice.
EC-LEG-11: County result correction arrives after national declaration; amendment requires re-declaration dual-custody workflow; both versions published with cryptographic proofs.

Time:
EC-TIM-01: Kit clock skew > 5 min at boot; kit refuses to open referendum session until NTS/GNSS time confirmed; session FSM unit test with faked RTC.
EC-TIM-04: DST or non-EAT timezone on diaspora kits; all timestamps stored UTC; close time computed in UTC; displayed in gazetted EAT for public tracker.
EC-TIM-08: Petition window boundary at T+7 midnight EAT (Elections Act s.75(1) inclusive); boundary table test covers T+6d 23:59:59 and T+7d 00:00:01.

Scale:
EC-SCL-04: Kafka consumer lag on `eel.governance.referendum.v1` exceeds 60 s; backpressure applied; no OOM; lag alert pages ECC; verified by slow-consumer integration test.
EC-SCL-07: 250k check-ins, 60k RTS, 18k BVI concurrent at election-critical load; referendum endpoints remain within SLA (p95 200 ms, p99 500 ms); verified by pre-prod simulator.
EC-SCL-03: Hot partition on referendum result topic (all 47 county events arriving within minutes); stream-id spread over partitions; lag alert; HPA scales consumers.

Data:
EC-DAT-01: Duplicate Form 37C submission with identical idempotency key and body; cached 200 response returned; no second county_result_certified event emitted.
EC-DAT-02: Same idempotency key reused with different county body (body mismatch); API returns 422 `idempotency-key-reuse`; original result preserved unchanged.
EC-DAT-10: Zero YES or NO votes for a county on close day (valid low-turnout result); national computation proceeds; no division-by-zero; county_yes_majority = false when yes = 0.
EC-DAT-11: County result where yes + no + rejected > registered voters; ingest blocked with machine-readable reason; RO must correct Form 37C before certification; TLA+ invariant asserted.
EC-DAT-03: Form 37A events arrive out of order from M10 pipeline; buffer until constituency gap filled or route to DLQ; shuffled-replay property test.

Security:
EC-SEC-01: RO digital signature mismatch on county result event; event never applied to state; routed to tamper topic; ECC paged within 30 s.
EC-SEC-03: HSM (P06) unavailable when kit mode bundle needs signing; bundle queued in degraded mode; ingest continues; countersign completes when HSM recovers; no gap in audit trail.
EC-SEC-05: Injection attack on question text, county result payload, and all API inputs; parameterised queries and input validation enforced; Semgrep + sqlx compile-time + ZAP + cargo-fuzz gates in CI.
EC-SEC-16: Voter ID, biometric reference, or other Tier A/B PII inadvertently included in referendum event payload; telemetry sanitiser strips field before export; CI log-scanner regex fixture asserts no hit.

Identity:
EC-IDN-02: Commissioner JWT expires mid-declaration request; in-flight request completes (token valid at entry); subsequent retry receives 401 and silent refresh; re-auth required for HSM step.
EC-IDN-10: SpiceDB unavailable during county result certification or declaration; all state-mutating endpoints fail-closed (HTTP 503); class-C read-only endpoints continue; chaos PodChaos test asserts no unauthorised write.

Connectivity:
EC-CON-04: Site partition between primary and mirror PostgreSQL; Patroni fencing prevents split-brain; MirrorMaker lag alert fires; RPO <= 5 min; linearizability check on tally stream post-partition.
EC-CON-11: Kafka ISR shrinks to 1 during county result ingest; producer blocks; outbox accumulates; no event loss; ECC paged; outbox drains when ISR recovers.
EC-CON-03: 2G-only connectivity to kit in remote county; kit refuses 2G data path for OTA bundle; uses signed SMS/USSD fallback; mode switch confirmed via SMS ACK.

Kit:
EC-KIT-03: OTA mode-switch bundle download interrupted (power cut or flapping link); A/B slot mechanism allows rollback; kit retries signed bundle on next connection; old mode preserved until new bundle verified.
EC-KIT-04: Referendum mode bundle not propagated to one or more kits by T-2 h; unacknowledged kits listed in ECC console within 60 s of bundle publish; ECC must resolve before polls-open gate passes.

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
