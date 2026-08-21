# M48 recall

| field | value |
|---|---|
| M-number | M48 |
| Home service | governance-svc (ADR-0020) |
| Owning work package | WP-5.2 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M48-recall.md |

## Purpose

Manages the constitutional recall petition process under Article 104, including signatory collection and verification, threshold reporting, and the consequent by-election handoff to M50. Enforces the 30% registered-voter signature threshold and the 30-day collection window.

## Surfaces and languages

| surface | technology | path under services/governance-svc/ |
|---|---|---|
| Public recall petition status portal | Astro | astro/governance-public/ |
| Petition management admin | Django | django/governance-admin/ |
| Signatory OCR import | Python worker | py/recall-ocr/ |
| Internal API | Rust (axum) | crates/recall/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| RecallPetition | id, constituency_id, mp_id, election_cycle_id, grounds, status (draft/active/closed/upheld/dismissed), signatory_target, filed_at | C | T1-consensus |
| SignatoryRecord | id, petition_id, voter_id, ward, ocr_confidence, verified_at, verification_method | B | T1-consensus |
| ThresholdReport | id, petition_id, total_signatories, registered_voters, percentage, meets_threshold, generated_at | C | T1-audit |
| RecallElection | id, petition_id, by_election_id (FK to M50), created_at | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/recall/petitions | 1 signed-static | public | n/a | list active petitions |
| GET /v1/recall/petitions/{id} | 2 live-per-user | public | n/a | petition status and threshold progress |
| POST /v1/recall/petitions | 3 accredited | JWT+SpiceDB | yes | file new petition (Commissioner dual-sign) |
| POST /v1/recall/petitions/{id}/signatories/import | 3 accredited | JWT+SpiceDB | yes | bulk OCR import of signature sheets |
| GET /v1/recall/petitions/{id}/threshold-report | 3 accredited | JWT+SpiceDB | n/a | current threshold report |
| PUT /v1/recall/petitions/{id}/close | 3 accredited | JWT+SpiceDB | yes | close petition window |
| POST /v1/recall/petitions/{id}/election | 3 accredited | JWT+SpiceDB | yes | create recall election via M50 |

OpenAPI: packages/openapi-contracts/governance/recall.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.governance.recall_petition_filed.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_threshold_met.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.recall_dismissed.v1 | emits | eel.governance.recall.v1 | operational | no | at-least-once idempotent |
| eel.governance.by_election_created.v1 | consumes | eel.governance.by_election.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR voter register | internal | crates/recall/ | yes | circuit breaker; threshold computation deferred |
| M05/M06 BVI (recall election BVI mode) | internal | crates/recall/ | yes | advisory; by-election provisioning continues |
| M50 by-election-operations | internal | crates/recall/ | yes | hard dependency for recall election handoff |
| M08 RTS kit (recall BVI mode) | internal | crates/rts-device-api/ | yes | mode switch via signed config bundle |

## ML and AI

- OCR confidence scoring on scanned signature sheets via M28 model registry; images deleted within 1 s of extraction.

## Dependencies

- Modules: M02 (voter register for threshold), M05/M06 (identification for recall election), M08 (kit mode switch), M50 (by-election creation)
- Platform: P01 (SpiceDB authz), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

1. FR-M48-001: File a recall petition with grounds, constituency reference, and Commissioner dual-sign.
2. FR-M48-002: Validate that constituency exists and MP is currently holding office.
3. FR-M48-003: Set 30-day collection window on petition activation.
4. FR-M48-004: Import bulk signatory sheets via OCR with per-row confidence score.
5. FR-M48-005: Verify signatory voter IDs against M02 register extract.
6. FR-M48-006: Reject duplicate voter ID in same petition.
7. FR-M48-007: Compute running total of verified signatories toward 30% threshold.
8. FR-M48-008: Generate threshold report with percentage, registered voter count, and ward breakdown.
9. FR-M48-009: Alert Commission when threshold is met.
10. FR-M48-010: Close petition window automatically at 30-day expiry.
11. FR-M48-011: Dismiss petition when window closes below threshold.
12. FR-M48-012: Uphold petition and create recall election via M50 when threshold met.
13. FR-M48-013: Public portal displays petition status and progress bar without PII.
14. FR-M48-014: Public portal lists all active and closed petitions.
15. FR-M48-015: Admin lists signatories with filter by ward and verification status.
16. FR-M48-016: Admin can manually flag a disputed signatory for adjudication.
17. FR-M48-017: OCR import worker processes batches asynchronously and reports per-row results.
18. FR-M48-018: OCR image deleted from storage within 1 s of data extraction (PII minimisation).
19. FR-M48-019: Petition closed status is immutable; corrections via compensating events only.
20. FR-M48-020: Recall election creation event consumed by M50; handoff idempotent.
21. FR-M48-021: Support concurrent recall petitions in different constituencies.
22. FR-M48-022: Audit trail of all petition status transitions with actor and timestamp.
23. FR-M48-023: i18n in 11 languages on public portal.
24. FR-M48-024: WCAG 2.2 AA on all public surfaces.
25. FR-M48-025: Threshold computation uses integer arithmetic (no floating point).
26. FR-M48-026: BVI recall election mode signalled via signed kit config bundle to M08.
27. FR-M48-027: Export signatory verification report as PDF for Commission resolution.
28. FR-M48-028: Petition filing window and threshold rules governed by Elections Act s.104 rule engine.

## Acceptance criteria

1. AC-M48-01: Threshold report computed correctly for boundary values (exactly 30%, 29.99%, 30.01%); boundary test passes.
2. AC-M48-02: Duplicate voter ID in same petition rejected with reason; test.
3. AC-M48-03: OCR image deleted within 1 s of extraction; integration test measures deletion time.
4. AC-M48-04: Petition window closes automatically at expiry with no manual intervention; scheduler test.
5. AC-M48-05: Recall election creation in M50 is idempotent; double-fire test.
6. AC-M48-06: Petition status transitions follow valid FSM; invalid transitions rejected; property test.
7. AC-M48-07: API p99 latency under nominal load within routine 99.5% SLA.
8. AC-M48-08: All public petition data contains no PII; OpenSearch audit confirms.
9. AC-M48-09: 11-locale snapshots pass; no hardcoded strings.

## Applicable edge cases

**Legal:**
- EC-LEG-02: court-ordered postponement of recall election; signed calendar amendment propagated.
- EC-TIM-08: 30-day petition window T+30 midnight EAT statutory; boundary test +-1 s.

**Data:**
- EC-DAT-01: duplicate signatory import; idempotent; no double-count.
- EC-DAT-07: partial batch failure in OCR import; per-row report; valid rows committed.
- EC-DAT-08: large signatory set (constituency up to 300k voters); streaming, checkpoint-resumable.
- EC-DAT-09: Unicode voter names on signatory sheets; NFC normalisation.

**Security:**
- EC-SEC-05: injection on grounds or petition fields.
- EC-SEC-10: voter enumeration via signatory list; redacted PII on public surface.
- EC-SEC-16: voter IDs must not appear in logs.

**Human:**
- EC-HUM-08: 11 languages; no truncation.
- EC-HUM-12: OCR on degraded signature sheets; confidence gate forces re-entry.

**ML:**
- EC-ML-03: torn/wet signature sheets; quality gate routes to manual.

**Scale:**
- EC-SCL-07: concurrent recall operations across constituencies within SLA.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | FSM transitions, threshold arithmetic, signatory dedup | nextest | M48-UNIT- |
| property | Threshold computation at boundary; signatory import idempotency | proptest | M48-PROP- |
| contract | M02 register extract; M50 by-election creation | Pact | M48-PACT- |
| integration | Full petition lifecycle with testcontainers | nextest + testcontainers | M48-INT- |
| e2e | Public portal petition status; admin import and close flows | Playwright | M48-E2E- |
| fuzz | OCR import payload; petition grounds field | cargo-fuzz | M48-FUZZ- |
| security | ZAP on public endpoints; PII log scanner | ZAP + Semgrep | M48-SEC- |
| load | Signatory import batch at peak | k6 | M48-LOAD- |
| i18n | 11 locales + pseudo | custom + Playwright | M48-I18N- |
| a11y | axe on public and admin surfaces | axe-core + Playwright | M48-A11Y- |

Tags: `#[trace("FR-M48-001","AC-M48-01","EC-TIM-08")]`

## Runbooks and dashboards

- docs/runbooks/governance-svc-degradation.md (extend with recall section)
- docs/runbooks/recall-petition-ocr-failure.md (new)
- platform/observability/governance-svc/recall-dashboard.json
- platform/observability/governance-svc/slo-alerts.yaml

## Open questions

- Article 104(3) interpretation: whether 30% threshold is of registered voters at last election or current register; legal opinion required before threshold rule engine is finalised.
- Whether recall petitions can overlap in time for the same MP; rule engine must handle edge case.
