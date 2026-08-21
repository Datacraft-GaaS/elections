# M59 oag-interface

| field | value |
|---|---|
| M-number | M59 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.1 (docs/plans/06-wave-5-governance.md) |
| Size | M |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M59-oag-interface.md |

## Purpose

Provides the Office of the Auditor-General with scoped, time-boxed audit engagement access to financial, inventory, payroll, and archive data packages, with evidence scope management and response tracking.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| OAG auditor portal | Django | django/governance-admin/ (OAG role) |
| Internal API | Rust (axum) | crates/oag/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| AuditEngagement | id, engagement_ref, financial_year, scope_description, lead_auditor, status (open/evidence-gathering/responded/closed), opened_at, closed_at | C | T1-consensus |
| EvidenceScope | id, engagement_id, module_ref, data_type, time_range_from, time_range_to, approved_by, approved_at | C | T1-consensus |
| EvidencePackage | id, scope_id, package_url, hash, generated_at, expires_at | C | T1-consensus |
| AuditResponse | id, engagement_id, finding_ref, response_text, responded_by, responded_at | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/oag/engagements | 3 accredited | JWT+SpiceDB | n/a | list audit engagements |
| POST /v1/oag/engagements | 3 accredited | JWT+SpiceDB | yes | create engagement |
| POST /v1/oag/engagements/{id}/scopes | 3 accredited | JWT+SpiceDB | yes | add evidence scope with approval |
| POST /v1/oag/engagements/{id}/scopes/{scope_id}/package | 3 accredited | JWT+SpiceDB | yes | generate evidence package |
| GET /v1/oag/engagements/{id}/scopes/{scope_id}/package | 3 accredited | JWT+SpiceDB | n/a | download evidence package (time-boxed URL) |
| POST /v1/oag/engagements/{id}/responses | 3 accredited | JWT+SpiceDB | yes | submit management response |
| PUT /v1/oag/engagements/{id}/close | 3 accredited | JWT+SpiceDB | yes | close engagement |

OpenAPI: packages/openapi-contracts/governance/oag-interface.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.audit_engagement_opened.v1 | emits | eel.governance.audit.v1 | operational | no | at-least-once idempotent |
| eel.governance.evidence_package_generated.v1 | emits | eel.governance.audit.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M54 budget-expenditure | internal | crates/oag/ | yes | advisory; financial data extract |
| M14 reproducibility-bundle | internal | crates/oag/ | yes | advisory; bundle package |
| M20 inventory-management | internal | crates/oag/ | yes | advisory; inventory extract |
| M55 hr-payroll | internal | crates/oag/ | yes | advisory; payroll extract |
| M69 long-term-archive | internal | crates/oag/ | yes | advisory; archive package |

## ML and AI

none

## Dependencies

- Modules: M54, M14, M20, M55, M69
- Platform: P01 (SpiceDB scoped access), P04 (Kafka outbox), MinIO (evidence packages)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py

## Feature checklist

Audit Engagement Management:
1. FR-M59-001: Create audit engagement with OAG reference, financial year, audit type (annual, special-election, follow-up), and engagement letter reference. (spec FR-M59-01)
2. FR-M59-002: Track engagement through notified, scope-agreed, evidence-production, management-response, report-published, and closed states. (spec FR-M59-01)
3. FR-M59-003: Register published OAG report with reference, publication date, and key findings within 5 business days of publication. (spec FR-M59-06)
4. FR-M59-004: Admin list of engagements with status badges and days-to-statutory-deadline counter; alert on overdue items.

Scope Negotiation:
5. FR-M59-005: Co-author scope document with OAG Audit Manager in a shared workspace. (spec FR-M59-02)
6. FR-M59-006: Define evidence scope per module with data type (kit-inventory, RTS metadata, BRN counts, payroll, financial), time range, and approval. (spec FR-M59-02)
7. FR-M59-007: Lock scope after dual sign-off (Commission CFO and OAG Audit Manager); scope changes require new dual sign-off. (spec FR-M59-02)
8. FR-M59-008: Enforce scope boundary at API level; out-of-scope data type request returns 403 with logged access-attempt event. (spec NFR-M59-03)

Evidence Production:
9. FR-M59-009: Generate evidence package as signed MinIO bundle; SHA-256 hash stored in transparency log. (spec FR-M59-03)
10. FR-M59-010: Provide time-boxed signed download URL for evidence package; expires <= 48 hours. (spec FR-M59-03)
11. FR-M59-011: Revoke package URL before expiry if scope changes; revocation logged in P05.
12. FR-M59-012: Alert when package URL approaches expiry without a confirmed download.
13. FR-M59-013: Pull evidence from M54 (budget-expenditure), M20 (inventory), M55 (HR payroll), M14 (reproducibility bundle), and M69 (archive) within agreed scope. (spec FR-M59-03)
14. FR-M59-014: Evidence extraction for up to 10M records completes within 30 minutes; extraction status available via polling endpoint. (spec NFR-M59-01)
15. FR-M59-015: Log each evidence extract with scope reference, extraction timestamp, record count, SHA-256 hash, and DPA purpose code. (spec FR-M59-07)
16. FR-M59-016: Record OAG acknowledgement of received evidence package with timestamp; follow up if no acknowledgement within 5 business days.

Query Management:
17. FR-M59-017: Accept OAG audit queries with OAG reference, query text, implicated modules, and response deadline. (spec FR-M59-04)
18. FR-M59-018: Assign audit query to module owner; track response deadline with T-7 alert.
19. FR-M59-019: Submit signed management response to OAG query; sign with CEO HSM token. (spec FR-M59-04)
20. FR-M59-020: Retain full query and response history permanently. (spec FR-M59-04)

Management Response:
21. FR-M59-021: Draft management response collaboratively with version control and diff view. (spec FR-M59-05)
22. FR-M59-022: CEO signs final management response; HSM timestamp recorded; hash in P05 Merkle chain. (spec FR-M59-05)
23. FR-M59-023: Alert CEO at T-14 and T-7 from management-response statutory deadline (Public Audit Act 2015 s.32: 30 days). (spec FR-M59-05, NFR-M59-06)
24. FR-M59-024: Submit signed management response within 30-day statutory deadline; deadline tracked from draft-finding receipt date.

Follow-Up and Reporting:
25. FR-M59-025: Log each OAG finding with severity (material, significant, minor), Commission response (accepted, partially-accepted, rejected with reason), owner, and target date. (spec FR-M59-08)
26. FR-M59-026: Track follow-up action items through open, in-progress, completed, and overdue states. (spec FR-M59-08)
27. FR-M59-027: Notify OAG Audit Manager on follow-up item closure with completion evidence reference. (spec FR-M59-08)
28. FR-M59-028: Generate and deliver follow-up status report to OAG at 6-month intervals. (spec FR-M59-08)

Audit and Compliance:
29. FR-M59-029: Log every OAG access event (data category, record count, timestamp, OAG user, scope reference) in P05. (spec FR-M59-07)
30. FR-M59-030: Generate OAG access report for any period via `/v1/oag-access-log`; row count verifiable against P05 log.
31. FR-M59-031: Issue time-limited OAG credentials scoped exactly to the agreed audit scope; credentials expire at engagement close. (spec NFR-M59-03)
32. FR-M59-032: OAG auditor portal shows scoped data only; no cross-engagement data leakage; isolation integration test required.
33. FR-M59-033: All evidence packages hashed; hash stored in transparency log; verifiable by Commission DPO.
34. FR-M59-034: DPIA filed with ODPC before any election-special audit involving personal data; registration tracked in engagement record. (spec NFR-M59-05)
35. FR-M59-035: OAG auditor portal meets WCAG 2.2 AA; axe-core verification required. (spec NFR-M59-07)

## Acceptance criteria

AC-M59-01: Evidence package URL expires after <= 48 hours; access denied after expiry with 403; test with synthetic URL and server-side time advance.
AC-M59-02: Request for data type not in agreed scope returns 403; API gateway enforces scope guard per data category; test for each data type variant.
AC-M59-03: Scope change requires new dual sign-off (Commission CFO and OAG Audit Manager); single-party sign returns 403; SpiceDB policy test.
AC-M59-04: Evidence package SHA-256 hash matches content on download; tamper detected and access denied; integrity test with flipped byte.
AC-M59-05: OAG auditor credentials cannot access data from a different engagement; cross-engagement request returns 403; isolation integration test.
AC-M59-06: API p99 < 2 s; evidence extraction for 10M records completes within 30 minutes; load and timing tests measured under representative synthetic load.
AC-M59-07: Management response submitted within 30-day statutory deadline for 100% of engagements; T-14 and T-7 alerts confirmed by scheduler test with synthetic draft-finding events.
AC-M59-08: Every OAG access event logged in P05 with data category, record count, OAG user, scope reference, and DPA purpose code; log completeness test over a full evidence-production cycle.
AC-M59-09: OAG credentials expire at engagement close; post-close API access returns 401; credential expiry integration test.
AC-M59-10: Evidence package delivered to OAG within 5 business days of scope lock; timer test with synthetic scope-lock event confirms delivery before deadline.
AC-M59-11: Follow-up action items closed by target date >= 90%; overdue items surfaced in dashboard with Alertmanager alert; dashboard integration test.
AC-M59-12: OAG auditor portal passes WCAG 2.2 AA; axe-core zero critical violations; manual test documented.
AC-M59-13: Portal availability >= 99.5%; SLO alert configured in iems-59-alerts; verified over 90-day synthetic measurement window.
AC-M59-14: Follow-up status report generated and delivered to OAG at 6-month intervals; automated report scheduler test with synthetic 6-month trigger.

## Applicable edge cases

EC-DAT-01: duplicate evidence package generation request with same scope and key returns cached 200; no second extraction triggered.
EC-DAT-02: duplicate request with same key but different scope body returns 422 idempotency-key-reuse; request rejected.
EC-DAT-08: evidence extraction of up to 10M financial and inventory records streams with checkpoint-resumable cursor; bounded memory enforced; kill-resume test verifies no data loss.
EC-DAT-09: OAG finding descriptions containing Unicode text stored in NFC; display tested in EN.
EC-DAT-14: oversized evidence extract exceeding 1 MB outbox limit redirected to MinIO blob with hash reference; outbox not used for large payloads.
EC-DAT-07: partial batch failure during multi-module evidence extraction (e.g., M54 succeeds, M20 fails); per-module error report returned; successful extracts committed; failed modules retried with backoff.
EC-IDN-02: JWT expires mid evidence-package download; in-flight download completes; next request returns 401 and triggers silent refresh.
EC-IDN-09: dual-control scope approver credential revoked between first and second sign-off; second sign re-validates both at commit; 403 returned.
EC-IDN-10: SpiceDB unavailable; scope-write operations fail closed; engagement reads allowed with stale-cache label; 503 returned.
EC-IDN-11: realm key rotated during active OAG session; JWKS refetched on kid miss; session not invalidated unexpectedly.
EC-SEC-05: injection attempted on scope description, query text, and finding description fields; parameterised queries only; ZAP and Semgrep verified.
EC-SEC-07: path traversal on evidence package download URL; sanitised; fuzz extractor with ../  corpus tested.
EC-SEC-16: evidence package contents, OAG query text, and audit finding detail absent from traces and application logs; log-scanner CI fixture enforced.
EC-SEC-06: SSRF via evidence package pre-signed URL field; allowlist enforced; private IP ranges blocked.
EC-SEC-03: HSM unavailable; management-response signing pauses; draft preserved; signing queue accumulates; resumes on HSM recovery; no response lost.
EC-CON-05: flapping MinIO during evidence package generation; generation retried with backoff; no partial packages stored; only complete packages delivered.
EC-CON-11: Kafka ISR shrinks during evidence_package_generated.v1 emit; producer blocks; outbox accumulates; no event lost on ISR recovery.
EC-SCL-04: consumer lag on eel.governance.audit.v1 exceeds 60 s; Alertmanager fires iems-59-alerts; backpressure applied; no OOM.
EC-SCL-08: Valkey eviction loses idempotency cache for package generation; PostgreSQL-durable store used as source of truth; Valkey is cache only.
EC-TIM-06: OAG query timestamp earlier than engagement creation time; audit skew anomaly logged; query accepted and not rejected.
EC-TIM-04: evidence package timestamp stored in UTC; EAT display for Commission users computed correctly; DST-safe conversion test.
EC-OPS-10: PITR restore during active audit engagement; drill conducted; RPO and RTO measured; restore event logged to transparency log.
EC-LEG-05: post-election special audit covers a presidential run-off cycle; evidence scope covers both election rounds; time-range validated against run-off declaration date.
EC-HUM-08: OAG auditor portal renders without truncation in EN; pseudo-localisation at 150% tested for future locale additions.
EC-OPS-07: OAG lead auditor credential reassigned mid-engagement due to staff change; old credential revoked; new credential scoped to same engagement; exactly-once credential guarantee enforced.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Scope guard, URL expiry, package hash | nextest | M59-UNIT- |
| property | Scope isolation invariant; URL expiry boundary | proptest | M59-PROP- |
| contract | M54 financial extract; M69 archive package | Pact | M59-PACT- |
| integration | Full engagement lifecycle; scope and package | nextest + testcontainers | M59-INT- |
| e2e | OAG portal; scope creation; package download | Playwright | M59-E2E- |
| security | ZAP; path traversal; log scanner | ZAP + Semgrep | M59-SEC- |

Tags: `#[trace("FR-M59-004","AC-M59-01","EC-SEC-07")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with OAG section)
- platform/observability/governance-svc/oag-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Scope creep risk: OAG may request data beyond initially agreed scope; escalation process and technical boundary enforcement to be agreed with IEBC legal.
- Whether evidence packages must be retained for a minimum period post-engagement; retention policy to align with KNADS requirements.
