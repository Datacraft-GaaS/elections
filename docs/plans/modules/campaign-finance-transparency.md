# M22 campaign-finance-transparency

| field | value |
|---|---|
| M-number | M22 |
| Home service | transparency-svc (ADR-0020) |
| Owning work package | WP-4.6 (docs/plans/05-wave-4-operations.md) |
| Size | M |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no (ADR-0013) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M22-campaign-finance-transparency.md |

## Purpose

Statutory campaign finance disclosure: accept contribution and expenditure disclosures from candidates and parties, cross-reference with CBK/FRC transaction data, flag anomalies, publish aggregated public reports, and generate evidence packages for the Commission and financial regulators.

## Surfaces and languages

| surface | technology | path under services/transparency-svc/ |
|---|---|---|
| Party/candidate disclosure portal | Astro on packages/eel-web-astro | astro/finance-portal/ |
| Disclosure API | Rust (axum + utoipa) | crates/finance-api/ |
| Anomaly detection worker | Rust | crates/finance-anomaly/ |
| Admin back-office | Django on packages/eel-web-py | django/finance-admin/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| CandidateDisclosure | disclosure_id, candidate_id, election_id, period, contributions_kes, expenditures_kes, submitted_at, sig | C | T2 |
| CashAggregate | agg_id, candidate_id, period, total_in, total_out, ceiling_kes, status, computed_at | D | T2 |
| AnomalyFlag | flag_id, disclosure_id, reason, severity, cbk_ref, status, resolved_at | C | T2 |
| EvidencePackage | pkg_id, candidate_id, election_id, content_refs, chain_of_custody, sig | C | T1-audit |
| AuditEvent | event_id, actor, action, ts_hlc, sig | C | T1-audit |

## APIs

OpenAPI file: packages/openapi-contracts/transparency/finance.yaml

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /finance/v1/disclosures | 2 live-per-user | JWT | yes | Submit disclosure for a period |
| GET /finance/v1/disclosures/{id} | 2 live-per-user | JWT | n/a | Disclosure status |
| GET /finance/v1/candidates/{id}/summary | 1 signed-static | public | n/a | Public aggregate for a candidate (ceiling status) |
| GET /finance/v1/reports | 1 signed-static | public | n/a | Published election-cycle finance report |
| GET /finance/v1/flags | 3 accredited | JWT+SpiceDB | n/a | Anomaly flag feed (the Commission internal) |
| PATCH /finance/v1/flags/{id} | 3 accredited | JWT+SpiceDB | yes | Adjudicate anomaly flag |
| POST /finance/v1/evidence/{candidate_id} | 3 accredited | JWT+SpiceDB | yes | Assemble evidence package |
| GET /finance/v1/evidence/{id} | 3 accredited | JWT+SpiceDB | n/a | Evidence package for regulator |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.finance.disclosure_submitted.v1 | emits | eel.finance.disclosure.v1 | operational | yes | at-least-once idempotent |
| eel.finance.anomaly_flagged.v1 | emits | eel.finance.anomaly.v1 | operational | yes | at-least-once idempotent |
| eel.finance.evidence_assembled.v1 | emits | eel.finance.evidence.v1 | operational | yes (HSM) | exactly-once |
| eel.crms.candidate_approved.v1 | consumes | eel.crms.candidate.v1 | operational | yes | at-least-once idempotent |
| eel.grievance.case_created.v1 | consumes | eel.grievance.case.v1 | operational | yes | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| CBK/FRC | external | integrations/cbk/ | yes (stub) | async cross-reference; queue; alert on fail |
| CRMS M12 | internal | integrations/crms/ | yes | candidate identity verify |
| Grievance channels M23 | internal | integrations/grievance/ | yes | finance-complaint signal |
| HSM (P06) | I07 | integrations/hsm/ | yes (SoftHSM) | fail closed on evidence package sign |
| Transparency log P05 | internal | integrations/tlog/ | yes | report + evidence anchoring |

## ML and AI

Anomaly detection: rule-based ceiling check (Elections Act s.98) + statistical outlier detection on disclosure amounts. No GPU required. Model card not required (rule-based only).

## Dependencies

- Modules: M12 (candidate identity), M23 (finance complaint source)
- Platform: P05 (audit), P06 (HSM)
- Packages: eel-crypto-rs, eel-outbox-rs, eel-web-astro, eel-web-py

## Feature checklist

Candidate registration and disclosure:

FR-M22-001: Candidate registration: every gazetted candidate must register within 7 days of gazettement; name, constituency, party, and ECFA 2013 s.22 spending-ceiling tier auto-populated from M12 CRMS.
FR-M22-002: Weekly disclosure upload wizard: (a) bank statement PDF/OFX, (b) M-Pesa Paybill activity CSV, (c) itemised expense receipts PDF per ECFA 2013 s.22 format.
FR-M22-003: OCR plus structured parser extracts totals from uploaded documents; candidate confirms extracted totals before submission; the Commission officer can override with audit trail.
FR-M22-004: Party finance officer can file on behalf of multiple candidates under same login; separate submission record with candidate-level idempotency key per candidate.
FR-M22-005: Disclosure status lifecycle: DRAFT, SUBMITTED, UNDER_REVIEW, ACCEPTED, QUERIED; every transition audit-logged.
FR-M22-006: Compliance rate tracking: percentage of candidates with complete weekly disclosure; automated non-compliance reminder at T+3 past deadline; formal notice at T+7.

Ceiling enforcement:

FR-M22-007: Spending ceiling enforcement: compute cumulative declared spending per candidate per contest; compare to ECFA 2013 s.22 ceiling for that contest tier.
FR-M22-008: Amber flag at 80% ceiling; red breach flag at 100%; red breach triggers the Commission notification and ODPP referral queue entry within 60 s.
FR-M22-009: Ceiling computation method: in-kind contribution valuation rule confirmed by the Commission legal and stored as versioned configuration; computation logged for auditability.
FR-M22-010: Compliance rate report: per-candidate, per-party, and aggregate filing rates; weekly summary delivered to Registrar of Political Parties.

CBK/FRC integration and anomaly detection:

FR-M22-011: Daily aggregated cash-out-volume data per constituency received from CBK/FRC under MoU (no individual transactions; no personal data); adapter version-negotiates data format.
FR-M22-012: Aggregated cash-movement choropleth map on public dashboard; updated daily; z-score from 30-day baseline; constituency colour-coded by deviation.
FR-M22-013: Spike flag: >2 standard-deviation spike from baseline during campaign period; FRC SAR alert draft generated within 30 min.
FR-M22-014: POCAMLA 2009 pattern detection: bulk cash-out near polling stations, rapid round-trip transactions, structuring patterns; SAR draft auto-generated for FRC analyst review before submission.
FR-M22-015: Foreign-money detection: inbound cross-border transfers flagged on candidate bank statement; ECFA 2013 ss.32-34 prohibition cross-referenced; auto-referral to CBK/FRC.
FR-M22-016: Anomaly adjudication workflow: the Commission officer reviews anomaly flags; accepts, dismisses, or escalates; all decisions audit-logged with actor, timestamp, and reason.
FR-M22-017: CBK/FRC feed unavailability: heat map shows last-known data with staleness warning; SAR pipeline queued; alert to the Commission finance desk if lag >48 h.
FR-M22-018: Bank and Sacco disclosure integration: aggregate financial-institution disclosures at constituency level for election-cycle window; cross-reference against candidate declared filings.

Whistleblower and evidence:

FR-M22-019: Whistleblower tip line: anonymous submission via USSD, WhatsApp, web, or short-code; accepts photo, geolocation, and statement; Whistleblower Protection Act 2022.
FR-M22-020: Whistleblower identity protection: phone stored hashed (SHA-256) only; no plain-text identity stored; Tor-accessible endpoint; legal-privilege annotation on tip records.
FR-M22-021: Verified tips routed to DCI and ODPP within 6 h via secure evidence package endpoint.
FR-M22-022: Evidence package assembly: HSM-signed PDF + JSON; candidate identity, disclosure extracts, bank statement references (not raw statements), computed totals, statutory ceiling value, excess amount, referral basis, timestamp chain.
FR-M22-023: Evidence package hash anchored in P05 Merkle chain; inclusion proof verifiable by courts.
FR-M22-024: the Commission Chair HSM signature required before evidence package routed to ODPP or DCI; single officer cannot complete routing.

Public reporting:

FR-M22-025: Public spending dashboard: per-candidate spending bar vs ceiling; constituency comparison view; weekly trend chart; downloadable CSV + JSON dataset; no raw bank statements or individual transactions published.
FR-M22-026: Post-election analytics: per-candidate cost-per-vote regression once tallies published; aggregate research dataset (no individual voter data); published at T+30.
FR-M22-027: ECFA prosecution registry: public-facing list of Election Offences Act s.12 cases; outcome; judgment date; updated within 14 days of each judgment.
FR-M22-028: Statutory post-election campaign finance report: published and anchored to P05 Merkle chain.

DPA and access control:

FR-M22-029: DPA 2019: raw banking data not retained beyond cross-reference window; bank statements encrypted at rest (AES-256); retention 7 years post-election per ECFA; DPIA registered with ODPC.
FR-M22-030: No individual bank transaction data visible to any party other than the filing candidate and the Commission Compliance Officer under a logged, purpose-coded access event.
FR-M22-031: RBAC: candidate portal, party treasurer, the Commission finance desk, FRC analyst, ODPP prosecutor access tiers; SpiceDB enforced; geographic scoping per constituency.
FR-M22-032: Audit trail: every the Commission officer action on a candidate record logged with actor, action, timestamp, IP, and purpose code.
FR-M22-033: WCAG 2.2 AA for candidate portal, the Commission back-office, and public dashboard; EN + SW primary; CN, Arabic, French for internal cross-reference views.

## Acceptance criteria

AC-M22-01: Ceiling breach flag generated and ODPP referral queue entry created within 60 s of disclosure submission crossing 100% of the ECFA ceiling (integration test; boundary at exactly 100%).
AC-M22-02: CBK/FRC constituency spike >2 standard deviations from baseline triggers FRC SAR alert draft within 30 min of data receipt (integration test).
AC-M22-03: Verified whistleblower tip routed to DCI and ODPP within 6 h; HSM-signed evidence package with Merkle proof (process audit + integration test).
AC-M22-04: Evidence package chain of custody verifiable via P05 Merkle inclusion proof; verification succeeds on CLI tool without network (integration test).
AC-M22-05: the Commission Chair HSM signature required; evidence package route to ODPP/DCI blocked without it; single-officer attempt returns 403 (security test).
AC-M22-06: No individual bank transaction data accessible via any public or party-portal API; bank statement files encrypted at rest (security review + penetration test).
AC-M22-07: Raw banking data purged after cross-reference window; retention audit confirms no records beyond window (data-retention audit; automated daily check).
AC-M22-08: Public aggregate shows ceiling status and spending percentage without raw transaction detail; OCR-extracted totals not exposed individually (security review + UAT).
AC-M22-09: OCR extraction completes within 5 min of upload for standard bank statement and Paybill CSV formats; the Commission officer override available if extraction fails (performance test).
AC-M22-10: Duplicate disclosure submission same idempotency key returns cached response; ceiling total not double-counted (integration test).
AC-M22-11: ECFA prosecution registry updated within 14 days of each judgment; automated alert fires if update is late (process audit).
AC-M22-12: 99.9% availability during campaign finance reporting cycle T-90 to T+30 (cycle-critical SLO per appendix; burn-rate alert fires before budget exhausted).
AC-M22-13: WCAG 2.2 AA: axe-core passes on candidate portal and public dashboard; manual screen-reader verification with NVDA and ChromeVox (accessibility audit).

## Applicable edge cases

EC-CON-02: Candidate portal on 3G (500 kbps, 300 ms RTT); disclosure upload with CBOR+zstd; p99 <=1 s under 5,000 concurrent users tested on throttled emulator.
EC-CON-06: CBK/FRC API unavailable; heat map shows last-known data with staleness warning; SAR alert pipeline queued; the Commission finance desk alerted after 48 h.
EC-TIM-04: Reporting deadline OPA hard-close at transaction time; in-flight disclosures that started before deadline complete; late submissions rejected with statutory reason.
EC-TIM-09: Disclosure window freeze race at T-90/T-30 milestone; OPA hard-close on retroactive period writes; concurrent writers handled all-or-nothing.
EC-DAT-01: Duplicate disclosure submission same idempotency key and body; cached response returned; ceiling total not double-counted; no re-execution.
EC-DAT-02: Same disclosure key, different body (e.g., candidate correcting a draft); rejected with 422 idempotency-key-reuse until prior submission is explicitly voided.
EC-DAT-03: Out-of-order CBK daily aggregate in stream; buffered until gap filled; DLQ if permanent gap after timeout.
EC-DAT-05: Poison disclosure (negative expenditure amount or amount exceeding plausible bound); rejected with 422; reason logged; candidate notified.
EC-DAT-06: CBK feed adapter schema v1/v2 coexist during MoU-driven format upgrade; both accepted; compat check enforced.
EC-DAT-07: Partial batch party-officer filing failure (multi-candidate upload, 3 of 20 fail); per-candidate report returned; valid submissions committed; failed ones returned with reason.
EC-DAT-08: 10,000 concurrent candidate uploads on weekly deadline; streaming OCR ingest; checkpoint-resumable after kill; bounded memory.
EC-DAT-09: Unicode candidate names and party names; NFC normalisation applied; ICU collation; no truncation on public dashboard or compliance console.
EC-DAT-10: Zero declared spending is valid; renders "KES 0 (0% of ceiling)"; no divide-by-zero in ceiling-pct-used computation.
EC-DAT-11: OCR-extracted total differs significantly from declared total (discrepancy_flag=true); the Commission officer notified; candidate prompted to confirm or correct; both values logged.
EC-DAT-13: Soft-deleted (withdrawn) candidate in search; tombstone honoured; aggregate excluded from public dashboard; raw file purge scheduled.
EC-IDN-02: JWT expires mid-disclosure upload; in-flight request completes (document already received); next request returns 401 with silent-refresh hint.
EC-IDN-10: SpiceDB down; fail closed on anomaly adjudication writes and evidence package routing; public dashboard read-only CDN tier continues.
EC-SEC-05: Injection on disclosure form fields (description, notes, unstructured evidence text); parameterised queries only; Semgrep compile-time and ZAP dynamic scan required.
EC-SEC-06: SSRF via supporting document URL field (external bank-statement link); allowlist enforced; private IP ranges blocked; DNS-rebinding guard active.
EC-SEC-10: Candidate enumeration via spending dashboard; per-candidate data published only after filing deadline; constant-time response shape; no existence leakage before publication.
EC-SEC-16: Secrets or PII in disclosure free-text refused at ingest; CBK cross-reference input data redacted from logs and traces; log-scanner fixture in CI.
EC-SCL-01: 100,000 concurrent public dashboard users; >=95% served from CDN edge; origin p99 <=2 s (k6 + Gatling 5 regions).
EC-SCL-04: Consumer lag >60 s on eel.finance.disclosure.v1 topic; backpressure activated; page fired; no OOM condition.
EC-LEG-10: Daraja duplicate CBK callback on SAR payment; single payment processed; idempotent refund on duplicate; FRC callback log verified.
EC-HUM-06: Screen readers: candidate portal upload wizard and public spending dashboard fully navigable via keyboard only; WCAG 2.2 AA verified with axe-core and manual NVDA test.
EC-HUM-08: 11 languages: constituency names, ceiling labels, compliance alert banners, and dashboard legends localised; no truncation on long-string locales; 150% pseudo-locale tested in CI.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | ceiling check, anomaly rules, CBK cross-ref parser, evidence builder, purge scheduler | nextest | M22-UNIT- |
| property | ceiling breach always flagged; evidence hash stable | proptest | M22-PROP- |
| contract | M12 candidate identity (consumer); M23 grievance signal (consumer) | Pact | M22-PACT- |
| integration | CBK stub; M12 double; SoftHSM evidence sign; tlog anchor | nextest + testcontainers | M22-INT- |
| e2e | disclosure submit -> ceiling check -> CBK cross-ref -> anomaly flag -> adjudicate -> evidence | Playwright | M22-E2E- |
| security | injection on disclosure; SSRF; PII-strip regression; OWASP ZAP | ZAP + Semgrep | M22-SEC- |
| load | campaign finance period close; concurrent disclosures | k6 | M22-LOAD- |

## Runbooks and dashboards

- docs/runbooks/finance-transparency-degradation.md (create): CBK unavailable; cross-reference lag; HSM fail; FRC handoff failure.
- platform/observability/transparency-svc/grafana-finance.json: disclosure rate, ceiling breach count, CBK cross-ref lag, anomaly flag queue depth.
- platform/observability/transparency-svc/slo-alerts.yaml: 99.9% burn rate; CBK lag > 48 h; ceiling breach unacknowledged > 4 h.

## Open questions

- CBK/FRC MoU: the data-sharing agreement has not been signed; legal must confirm scope, data minimisation requirements, and liability before CBK cross-reference is enabled.
- Ceiling computation method: the Elections Act s.98 does not specify exactly how in-kind contributions are valued; the Commission legal must confirm the valuation method before the ceiling check is implemented.
