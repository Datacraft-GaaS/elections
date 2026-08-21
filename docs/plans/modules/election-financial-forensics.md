# M66 election-financial-forensics

| field | value |
|---|---|
| M-number | M66 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M66-election-financial-forensics.md |

## Purpose

Performs post-election financial forensics using regression discontinuity and instrumental-variable econometric methods to detect anomalous vote-expenditure relationships, produces replication packages, and refers findings to FRC and KNBS with methodology published for contestation.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Forensics admin and analyst console | Django | django/governance-admin/ |
| Public methodology and report portal | Astro | astro/governance-public/ |
| ML forensics worker | Python | py/forensics-ml/ |
| Internal API | Rust (axum) | crates/forensics/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AnalysisCycle | id, election_id, method (RD/IV/DiD), status (draft/running/complete/published), started_at, completed_at | C | T1-consensus |
| EstimateRecord | id, cycle_id, constituency_id, estimate_value, confidence_interval_low, confidence_interval_high, p_value, model_version | D | T2 |
| ReferralPacket | id, cycle_id, referred_to (FRC/KNBS/ODPP), referral_basis, content_url, referred_at, acknowledged_at | C | T1-audit |
| ReplicationPackage | id, cycle_id, code_url, data_hash, environment_spec, doi, published_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/forensics/cycles | 3 accredited | JWT+SpiceDB | n/a | list analysis cycles |
| POST /v1/forensics/cycles | 3 accredited | JWT+SpiceDB | yes | create analysis cycle |
| PUT /v1/forensics/cycles/{id}/run | 3 accredited | JWT+SpiceDB | yes | trigger analysis run |
| GET /v1/forensics/cycles/{id}/estimates | 3 accredited | JWT+SpiceDB | n/a | estimates by constituency |
| POST /v1/forensics/cycles/{id}/referrals | 3 accredited | JWT+SpiceDB | yes | create referral packet |
| POST /v1/forensics/cycles/{id}/replication | 3 accredited | JWT+SpiceDB | yes | publish replication package |
| GET /v1/forensics/reports | 1 signed-static | public | n/a | public methodology and summary reports |
| GET /v1/forensics/reports/{id} | 1 signed-static | public | n/a | report detail with replication package link |

OpenAPI: packages/openapi-contracts/governance/election-financial-forensics.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.forensics_referral_issued.v1 | emits | eel.governance.forensics.v1 | operational | no | at-least-once idempotent |
| eel.governance.replication_package_published.v1 | emits | eel.governance.forensics.v1 | operational | no | at-least-once idempotent |
| eel.declaration.result_declared.v1 | consumes | eel.declaration.result.v1 | event-source | yes | at-least-once idempotent |
| eel.transparency.campaign_finance_disclosed.v1 | consumes | eel.transparency.finance.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M22 campaign-finance-transparency | internal | crates/forensics/ | yes | hard dependency; disclosure data source |
| M10 rts-backend (result data) | internal | crates/forensics/ | yes | hard dependency; tally data source |
| M69 long-term-archive | internal | crates/forensics/ | yes | advisory; replication package archived |
| FRC (Financial Reporting Centre) | cbk-frc | integrations/cbk-frc/ | yes | circuit breaker; referral queued |
| KNBS | knbs (tier B double) | integrations/ | yes | circuit breaker; data exchange deferred |

## ML and AI

- Python econometric worker (py/forensics-ml/): regression discontinuity, IV, and difference-in-differences estimation using statsmodels/numpy; model version pinned in replication package.
- LiteLLM via ai-platform for narrative summary generation on analyst console only; no voter PII in prompts.

## Dependencies

- Modules: M22 (campaign finance), M10 (results), M69 (archive)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

1. FR-M66-001: Create analysis cycle selecting method (RD/IV/DiD) and election.
2. FR-M66-002: Ingest campaign finance disclosure data from M22.
3. FR-M66-003: Ingest result tally data from M10 by constituency.
4. FR-M66-004: Run econometric estimation with per-constituency estimates and confidence intervals.
5. FR-M66-005: Version-pin model and input data; store in replication package.
6. FR-M66-006: Publish methodology report publicly with replication package and DOI.
7. FR-M66-007: Analyst console shows constituency-level estimate heatmap.
8. FR-M66-008: Flag estimates exceeding anomaly threshold for manual review.
9. FR-M66-009: Create referral packet to FRC, KNBS, or ODPP for flagged constituencies.
10. FR-M66-010: Track referral acknowledgement from recipient agency.
11. FR-M66-011: Replication package includes code, data hash, environment spec, and DataCite DOI.
12. FR-M66-012: Public summary report in plain language; no voter PII.
13. FR-M66-013: Admin list of cycles with status and referral count.
14. FR-M66-014: Audit trail of all cycle and referral transitions.
15. FR-M66-015: Methodology published before results so contestation is possible.
16. FR-M66-016: i18n summary report in 11 languages on public portal.
17. FR-M66-017: WCAG 2.2 AA on public portal.

## Acceptance criteria

1. AC-M66-01: Replication package reproduces published estimates from archived inputs; CI test on pinned data hash.
2. AC-M66-02: Referral packet idempotent; double-fire test produces single referral.
3. AC-M66-03: No voter PII in ML prompts or public reports; log-scanner and Semgrep test.
4. AC-M66-04: Methodology published before cycle run; chronological ordering enforced; test.
5. AC-M66-05: DOI minted via DataCite double; replication package URL stable; test.
6. AC-M66-06: API p99 within routine 99.5% SLA.

## Applicable edge cases

**ML:**
- EC-ML-01: model drift across elections; version pinned in card; per-release evaluation.
- EC-ML-04: bias across geographic groups; per-group metrics in replication package.
- EC-ML-05: PII in ML prompts; PIIStrippingEvent before LiteLLM call.

**Security:**
- EC-SEC-16: no voter PII in forensics logs or public reports.
- EC-SEC-05: injection on referral basis or constituency fields.

**Data:**
- EC-DAT-08: large constituency dataset (500+ rows per election); streaming; bounded memory.
- EC-DAT-10: zero results in constituency; valid; no division errors.

**Legal:**
- EC-LEG-08: evidence request at T+6d 23h; forensics cycle data accessible on request.

**Human:**
- EC-HUM-08: 11 languages on public summary report.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Analysis FSM, anomaly threshold, referral routing | nextest (pytest) | M66-UNIT- |
| property | Replication determinism; referral idempotency | proptest | M66-PROP- |
| contract | M22 finance data; M10 results; FRC/KNBS doubles | Pact | M66-PACT- |
| integration | Full analysis lifecycle; referral; replication package | nextest + testcontainers | M66-INT- |
| e2e | Admin cycle run; public methodology report | Playwright | M66-E2E- |
| security | PII log scanner; Semgrep; ZAP | ZAP + Semgrep | M66-SEC- |
| i18n | 11 locales on public report | custom | M66-I18N- |
| a11y | axe on public portal | axe-core | M66-A11Y- |

Tags: `#[trace("FR-M66-006","AC-M66-01","EC-ML-05")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with forensics section)
- platform/observability/governance-svc/forensics-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Econometric method contested: the upstream sw/ text may specify a method that conflicts with KNBS statistical standards; independent methodological review before cycle publication is required.
- DataCite DOI minting requires IEBC DataCite membership; membership application status unknown.
