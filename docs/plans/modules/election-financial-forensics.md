# M66 election-financial-forensics

| field | value |
|---|---|
| M-number | M66 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.3 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M66-election-financial-forensics.md |

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

Data ingestion

1. FR-M66-001: Ingest per-candidate disclosed expenditure by category and date from M22 campaign finance disclosure stream. (FR-66.01)
2. FR-M66-002: Ingest per-constituency and per-station vote totals from M10 RTS backend result stream. (FR-66.01)
3. FR-M66-003: Accept FRC anonymised constituency-level cash-flow anomaly index via authenticated SFTP or API under MOU; reject if index contains individual account-holder data. (FR-66.01, FR-66.03)
4. FR-M66-004: Accept KEPSA field cash-flow estimates with confidence intervals via authenticated upload API. (FR-66.01, FR-66.08)
5. FR-M66-005: Ingest KNBS demographic and poverty covariates: population density, poverty index, unemployment rate. (FR-66.01)
6. FR-M66-006: Ingest historical vote-share from prior cycles stored in the accumulated panel dataset. (FR-66.01, FR-66.09)
7. FR-M66-007: Validate all ingested rows for schema compliance before pipeline run; produce per-row report on rejection; commit valid rows; fail cycle if below 200-constituency threshold. (EC-DAT-07, FR-66.02)
8. FR-M66-008: Enforce MOU boundary: reject FRC ingest payload if individual account fields are detected; raise AdjudicationCase. (NFR-66.03)
9. FR-M66-009: Idempotent ingest; duplicate source event with same key and body returns cached ingest record; no re-execution. (EC-DAT-01)

Regression discontinuity

10. FR-M66-010: Run regression discontinuity estimation per candidate per constituency using vote margin as running variable. (FR-66.02)
11. FR-M66-011: Apply CCT algorithm for bandwidth selection per constituency. (FR-66.02)
12. FR-M66-012: Output per-constituency RD results: LATE estimate, standard error, 95% CI, bandwidth, polynomial order. (FR-66.02)
13. FR-M66-013: Require at least 200 constituencies before proceeding with national-level RD; fail with structured reason if below threshold. (FR-66.02)

Instrumental variables estimation

14. FR-M66-014: Run two-stage least squares IV estimation using FRC cash-flow anomaly index as instrument. (FR-66.03)
15. FR-M66-015: Report first-stage F-statistic and Kleibergen-Paap weak-instrument test result per constituency. (FR-66.03)
16. FR-M66-016: Exclude constituencies where IV F-statistic is below 10 from published IV estimates. (FR-66.03)

Difference-in-differences estimation

17. FR-M66-017: Run DID panel estimation across election cycles with constituency and time fixed effects. (FR-66.04)
18. FR-M66-018: Import cleaned panel dataset from prior cycles (2017, 2022) for DID baseline via historical panel API. (FR-66.04)

Pipeline infrastructure

19. FR-M66-019: All analysis runs containerised with pinned OCI image digests following M14 reproducibility pattern. (NFR-66.02)
20. FR-M66-020: Fix random seed for any randomisation step; outputs bit-identical across independent runs from identical inputs. (NFR-66.02)
21. FR-M66-021: Full pipeline (290 constituencies, three methods) completes within 4 hours on 16-core compute instance. (NFR-66.01)
22. FR-M66-022: Parallelise estimation by constituency across available cores. (NFR-66.01)
23. FR-M66-023: Pipeline run status exposed via API with 30-second polling interval. (FR-66.01)

Cost-per-vote derivation

24. FR-M66-024: Compute cost-per-vote point estimate as effective_spend_estimate divided by marginal_votes; propagate confidence interval. (FR-66.05)
25. FR-M66-025: Flag constituency where cost-per-vote point estimate exceeds KES 2,000 and CI lower bound exceeds KES 1,000 for FRC or ODPP referral review. (FR-66.05)
26. FR-M66-026: Publish null-effect findings equally with positive findings; no suppression by method or magnitude. (NFR-66.04)

Referral pipeline

27. FR-M66-027: Generate referral packet for each flagged constituency containing constituency code, candidate name, party, cost-per-vote estimate, CI, data sources, and methodology note. (FR-66.06)
28. FR-M66-028: Transmit referral packet to ODPP via authenticated SFTP with PGP encryption. (FR-66.06)
29. FR-M66-029: Record ODPP acknowledgement reference against each referral packet. (FR-66.06)
30. FR-M66-030: Idempotent referral: double-fire produces one referral packet per constituency per cycle; second call returns cached record. (EC-DAT-01, FR-66.06)
31. FR-M66-031: Circuit breaker on ODPP SFTP; referral queued on unavailability; encrypted email fallback after two delivery failures. (FR-66.R5)
32. FR-M66-032: IAB access to full referral packet list; ODPP Investigator access limited to authorised referrals via SpiceDB. (NFR-66.06)

Published report

33. FR-M66-033: Aggregate national and county-level cost-per-vote distribution for public tier; candidate-level estimates suppressed to constituency-level aggregates. (FR-66.07)
34. FR-M66-034: Public report released within 90 days of election as PDF and machine-readable JSON. (FR-66.07)
35. FR-M66-035: Report includes kernel density chart, box plots by party, and time-series comparison across cycles. (FR-66.07)
36. FR-M66-036: Methodology document published before analysis cycle run; chronological ordering enforced; system prevents run if methodology is not yet published. (NFR-66.04)
37. FR-M66-037: Public report and summary page available in EN and SW; plain-language summary in all 11 gazetted languages. (EC-HUM-08)
38. FR-M66-038: WCAG 2.2 AA on public methodology and report portal. (NFR-66.05)

Replication package

39. FR-M66-039: Replication package includes analysis code (Python, open-source), input data (M22 public disclosures, KNBS covariates, anonymised FRC index), and output tables. (FR-66.08)
40. FR-M66-040: Publish replication package on GitHub within 90 days of election. (FR-66.08)
41. FR-M66-041: Mint DataCite DOI for replication package; store DOI in ReplicationPackage record; URL stable. (FR-66.011)
42. FR-M66-042: Any IAB member can reproduce all public-tier results from published replication package inputs. (NFR-66.02)

Historical panel

43. FR-M66-043: Maintain panel dataset across cycles; each election adds new observation layer to accumulated panel. (FR-66.09)
44. FR-M66-044: Archive panel and reports to M69 long-term archive for 15-year retention. (FR-66.09)
45. FR-M66-045: Panel access restricted to accredited analysts via SpiceDB; data governance policy enforced. (NFR-66.03, NFR-66.06)

Security and access control

46. FR-M66-046: Candidate-level raw outputs and ODPP referral packets not exposed in public API response; classified restricted. (NFR-66.06)
47. FR-M66-047: TLS 1.3 and mTLS for FRC and ODPP integrations; classical TLS 1.3 only; 1.2 refused. (NFR-66.06)
48. FR-M66-048: Audit log on every access to candidate-level estimates and referral packets; retained in P05. (NFR-66.06)
49. FR-M66-049: SpiceDB RBAC: ODPP Investigator read scope limited to authorised referrals; IAB all tiers; public tier unauthenticated. (NFR-66.06)
50. FR-M66-050: SpiceDB-down: fail closed on writes; class-C reads only; return 503 problem+json. (EC-IDN-10)
51. FR-M66-051: All inputs parameterised; no injection on constituency code, referral-basis, or estimate fields; Semgrep CI gate. (EC-SEC-05)
52. FR-M66-052: No voter PII or individual account-holder data in any log, trace, or ML prompt. (NFR-66.03, EC-SEC-16)
53. FR-M66-053: PIIStrippingEvent fires before every LiteLLM narrative-summary call on analyst console. (EC-ML-05)
54. FR-M66-054: JWT silent refresh mid-analysis-trigger POST; next unauthenticated call returns 401. (EC-IDN-02)

Observability and dashboards

55. FR-M66-055: Grafana dashboard eel-M66 with pipeline run status, referral count, method completion, and SLO gauges. (NFR-M66-MONITORING)
56. FR-M66-056: Alert fires on pipeline failure or stall exceeding 4-hour SLA. (NFR-66.01)
57. FR-M66-057: Analyst console constituency-level estimate heatmap with CI visualisation; flagged constituencies highlighted. (FR-66.07)

## Acceptance criteria

1. AC-M66-01: Replication package reproduces published estimates from archived inputs; two independent runs from same inputs produce bit-identical output tables; CI test on pinned data hash. (NFR-66.02)
2. AC-M66-02: Referral packet idempotent; double-fire produces exactly one referral per constituency per cycle; second call returns cached 200; no duplicate record. (EC-DAT-01)
3. AC-M66-03: No voter PII or individual account-holder data in any M66 database table, API response, log, or ML prompt; automated schema scan and Semgrep CI pass. (EC-SEC-16, NFR-66.03)
4. AC-M66-04: Methodology document published before cycle run; attempt to trigger run before methodology publication returns 422; chronological ordering enforced in integration test.
5. AC-M66-05: DataCite DOI minted via double; replication package URL stable and resolvable after publication; integration test.
6. AC-M66-06: Public report API p99 within routine 99.5% SLA at expected post-election read traffic; load test.
7. AC-M66-07: Full pipeline completes within 4 hours for 290 constituencies on 16-core instance; performance test with synthetic full dataset.
8. AC-M66-08: Kleibergen-Paap F-statistic below 10 excludes constituency from IV published estimates; unit test with planted weak-instrument dataset.
9. AC-M66-09: Flagging rule correct: cost-per-vote above KES 2,000 and CI lower bound above KES 1,000 triggers referral; cost-per-vote below KES 1,000 or CI lower bound below KES 1,000 does not; unit tests with planted known-effect datasets.
10. AC-M66-10: Null-effect constituency not suppressed; published output contains all constituencies including zero-effect; property test over randomly sampled synthetic elections.
11. AC-M66-11: SpiceDB down triggers fail-closed on writes; class-C reads returned to callers; PodChaos test.
12. AC-M66-12: ODPP SFTP circuit breaker on unavailability; referral queued; encrypted email fallback triggered after exactly two delivery failures; integration test.
13. AC-M66-13: ODPP mTLS connection rejected on self-signed cert; TLS 1.3 enforced; 1.2 connection refused; testssl check.
14. AC-M66-14: WCAG 2.2 AA on public portal; axe-core CI pass with zero critical violations on methodology and report pages.
15. AC-M66-15: IAB member reproduces all public-tier results from published replication package; reproducibility integration test passes.
16. AC-M66-16: PIIStrippingEvent fires before every LiteLLM call; regression corpus test; Semgrep rule enforced; no prompt contains candidate name or account fields.

## Applicable edge cases

Data (strongly applicable)

EC-DAT-01: Duplicate analysis cycle trigger or duplicate FRC ingest event with same key and body; cached response returned; no second record or re-execution.
EC-DAT-02: Same idempotency key with different cycle or ingest body; 422 idempotency-key-reuse returned; original record unchanged.
EC-DAT-04: Replayed signed FRC ingest event; reject on sequence counter; tamper topic raised; original record preserved.
EC-DAT-06: M22 campaign-finance schema v1 and v2 coexist in ingest stream; both accepted; dual-emit; Karapace compatibility check on schema registry.
EC-DAT-07: Partial ingest batch failure from M22 or KNBS; per-row report generated; valid rows committed; cycle fails with structured reason if below 200-constituency threshold.
EC-DAT-08: Large constituency dataset (500 rows per cycle per method); streaming estimation; checkpoint-resumable pipeline; bounded memory on worker.
EC-DAT-10: Zero-result constituency with no spending disclosed and zero margin; valid; cost-per-vote denominator guarded against division-by-zero; zero result published.
EC-DAT-15: FRC data contains two records for same constituency; AdjudicationCase raised; no auto-merge; cycle paused until resolved.

Security (strongly applicable)

EC-SEC-04: KEK rotation mid-analysis cycle; online re-wrap of stored estimates; old key readable until rotation complete; audit logged.
EC-SEC-05: Injection on constituency code, referral-basis, or estimate value fields; parameterised queries only; Semgrep CI gate enforces no string interpolation; ZAP scan on all write endpoints.
EC-SEC-06: SSRF via KEPSA estimate upload URL field; allowlist enforced; private IP ranges blocked; rebinding resolver guard.
EC-SEC-08: XXE or deserialisation attack via KEPSA XML estimate upload; DTD off; strict Pydantic model; billion-laughs fuzz test.
EC-SEC-16: Candidate-level estimates and ODPP packet fields excluded from all logs and traces; log-scanner CI fixture returns zero matches on Tier A and B field patterns.

Identity

EC-IDN-02: JWT expires mid-analysis-trigger POST; in-flight request completes with original token; next call returns 401 or triggers silent refresh.
EC-IDN-10: SpiceDB down during referral-packet write; fail closed; 503 problem+json returned; class-C reads served from replica.

Scale

EC-SCL-04: Consumer lag exceeds 60 seconds on forensics event topic; backpressure applied; page fired; no OOM on worker pod.
EC-SCL-06: Cold replay from tiered storage for prior-cycle panel data; throttle at 10 MB per second; snapshot-then-tail strategy applied.

ML

EC-ML-01: Model drift between election cycles; RD, IV, and DID model versions pinned in replication package card; per-release held-out evaluation required before publication.
EC-ML-04: Bias across geographic groups (urban vs rural constituencies); per-group fairness metrics published in replication package; tolerance checked against threshold.
EC-ML-05: PII in LiteLLM narrative-summary prompt; PIIStrippingEvent fires before every call; regression corpus test; Semgrep rule enforces no candidate name or account fields in prompt.

Legal

EC-LEG-08: Evidence request at T+6d 23h; forensics cycle data and referral packets accessible to authorised requestors; not freeze-blocked by tally process.

Time

EC-TIM-06: M10 result event time precedes M22 disclosure causation time; accept; audit skew anomaly logged; no reorder of estimation pipeline.

Operations

EC-OPS-10: PITR restore during active analysis cycle; cycle resumes from last checkpoint; restore event logged to transparency log; no gap in referral audit trail.

Human

EC-HUM-08: 11 languages on public summary report; no truncation of long translated methodology headings; ICU plurals; font fallback for all scripts.
- EC-DAT-03: out-of-order events on this module's consumed topics are buffered until the gap fills or routed to DLQ
- EC-DAT-05: a schema-valid but semantically impossible payload is routed to DLQ with reason, partition continues
- EC-DAT-09: Unicode names in every text field are NFC-normalised and never truncated mid-grapheme
- EC-DAT-13: soft-deleted rows never resurface in search or exports

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
- DataCite DOI minting requires the Commission DataCite membership; membership application status unknown.
