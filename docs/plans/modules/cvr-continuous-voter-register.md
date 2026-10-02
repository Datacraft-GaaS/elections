# M35 cvr-continuous-voter-register

| field | value |
|---|---|
| M-number | M35 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M35-cvr-continuous-voter-register.md |

## Purpose

Maintain the voter register as a living record between election cycles by ingesting death notifications from CRS (Civil Registration Service), identity updates from NRB (National Registration Bureau), and KRA-PIN dormancy signals; processing family short-code (*400#) provisional death reports requiring CRS confirmation within 90 days; synchronising updates with the ABIS biometric store (M03) and BVR back-end (M02); and publishing signed quarterly gazette notices of removals anchored in the Merkle chain; distinct from M01 (initial first-time enrolment).

## Surfaces and languages

| surface | technology | path |
|---|---|---|
| Officer Triage Console | Phoenix LiveView | services/bvr-svc/ex/cvr_web/ |
| Commissioner Gazette Panel | Django | services/bvr-svc/django/gazette_admin/ |
| Batch Ingestion Dashboard | Django | services/bvr-svc/django/gazette_admin/ |
| Public gazette and self-check | Astro | services/bvr-svc/astro/cvr-public-site/ |
| API | Rust (axum) | services/bvr-svc/crates/bvr-api/ |

Languages: EN + SW; citizen short-code and self-check in EN + SW + Kikuyu + Luo + Kamba.

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VoterRegisterEvent | id, voter_id, event_type (DEATH_PROPOSED/DEATH_CONFIRMED/ID_CORRECTION/DORMANCY_FLAG/FAMILY_NOTIFICATION/REMOVAL_APPROVED/REMOVAL_REJECTED/REINSTATEMENT), source (CRS/NRB/KRA/FAMILY_SHORTCODE/OFFICER_MANUAL), confidence_score, batch_id, officer_id, decision (APPROVED/REJECTED/PENDING/HELD), decision_basis, created_at, resolved_at | A | T1-audit |
| RegisterSnapshot | id, snapshot_at, total_registered, root_hash (SHA-256 sorted voter ID list), merkle_root, gazette_notice_id, signed_by, signed_at | A | T1-audit |
| BatchIngestionReport | id, source (CRS/NRB/KRA), received_at, record_count, matched, unmatched, flagged, actioned, errors, report_url | B | T1-consensus |

## APIs

OpenAPI file: `packages/openapi-contracts/bvr/cvr.yaml`

| method + path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| POST /v1/cvr/batches/crs | internal gRPC | SVID mTLS (CRS_SYSTEM) | yes | ingest CRS death-notification batch; 5/day |
| POST /v1/cvr/batches/nrb | internal gRPC | SVID mTLS (NRB_SYSTEM) | yes | ingest NRB ID-correction batch; 5/day |
| POST /v1/cvr/batches/kra | internal gRPC | SVID mTLS (KRA_SYSTEM) | yes | ingest KRA dormancy flags; 5/quarter |
| GET /v1/cvr/events | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | n/a | triage queue; proposed removals |
| PATCH /v1/cvr/events/{id} | 2 | JWT+SpiceDB (COMMISSION_OFFICER) | yes | approve / reject / hold event |
| GET /v1/cvr/snapshots | 2 | JWT+SpiceDB (COMMISSION_ADMIN) | n/a | list register snapshots |
| POST /v1/cvr/snapshots | 2 | JWT+SpiceDB (COMMISSION_COMMISSIONER) + HSM | yes | create and sign new snapshot; 2/day |
| GET /v1/cvr/snapshots/{id}/proof | 2 | JWT+SpiceDB (COURT_REGISTRAR) | n/a | Merkle proof for snapshot |
| POST /v1/cvr/family-notifications | internal | USSD_GATEWAY secret | yes | *400# family death report intake; 1000/hr |
| GET /v1/cvr/status/{national_id_hash} | 1 | public + rate limit | n/a | citizen self-check: status + last updated; no internal flags; 30/hr |

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.removal_proposed.v1 | emits | eel.bvr.cvr.v1 | operational | yes | at-least-once idempotent |
| eel.bvr.removal_approved.v1 | emits | eel.bvr.cvr.v1 | operational | yes | exactly-once |
| eel.bvr.removal_rejected.v1 | emits | eel.bvr.cvr.v1 | operational | yes | exactly-once |
| eel.bvr.id_correction_applied.v1 | emits | eel.bvr.cvr.v1 | operational | yes | exactly-once |
| eel.bvr.gazette_published.v1 | emits | eel.bvr.gazette.v1 | operational | yes | exactly-once |
| eel.bvr.register_snapshot.v1 | emits | eel.bvr.snapshot.v1 | event-source | yes | exactly-once |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M02 BVR Back-End | internal | integrations/bvr/ | yes | voter record corrections; circuit breaker |
| M03 ABIS | internal | integrations/abis/ | yes | biometric soft-hold on proposed removals; circuit breaker |
| M13 Public Verifier App | internal | integrations/verifier/ | yes | citizen self-check surface |
| M15 IAB Dashboard | internal | integrations/iab/ | yes | aggregate churn metrics; batch reconciliation |
| M36 Voter Deregistration Appeal | internal | integrations/deregistration/ | yes | escalation path for approved removals |
| M40 Election Court IT Interface | internal | integrations/court/ | yes | signed snapshots for petitioners |
| P05 Merkle Chain | P05 | integrations/merkle/ | yes | gazette notice and snapshot anchoring |
| P06 HSM | P06 | integrations/hsm/ | yes | Commissioner signing |
| CRS Civil Registration Service | external (MoU) | integrations/crs/ | yes (stub) | SFTP batch daily; PGP-signed; retry with alert |
| NRB National Registration Bureau | external (MoU) | integrations/nrb/ | yes (stub) | SFTP batch weekly |
| KRA | external (MoU) | integrations/kra/ | yes (stub) | SFTP batch quarterly |
| Safaricom / Africa's Talking | I08 | integrations/ussd-gateway/ | yes | *400# USSD intake |

## ML and AI

- Confidence-score matching for CRS death records against voter register: deterministic rule-based (National ID + biometric reference cross-check); no ML model.

## Dependencies

- Modules: M02, M03, M13, M15, M36, M40
- Platform: P05, P06
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs

## Feature checklist

Batch ingestion:

1. FR-M35-001: CRS death-feed ingestion via daily SFTP; PGP-encrypted; schema-validated against registered schema before any parsing; file rejected and SOC alerted if PGP signature is invalid or absent
2. FR-M35-002: NRB ID-update ingestion via weekly SFTP; PGP-signed; fields covered include name, ID number, and date of birth; applied after officer review
3. FR-M35-003: KRA-PIN dormancy signal ingestion via quarterly SFTP; PINs dormant >= 7 years flagged; creates dormancy-review record requiring officer confirmation before any removal proceeds
4. FR-M35-004: family short-code intake via *400# USSD; family member enters deceased voter's ID number; provisional record created; SMS acknowledgment sent within 60 seconds
5. FR-M35-005: idempotent batch ingestion; same idempotency key + body returns cached 200 without reprocessing; same key + different body returns 422 idempotency-key-reuse; no duplicate events created
6. FR-M35-006: agency rate limits enforced: CRS 5 batches/day, NRB 5 batches/day, KRA 5 batches/quarter; excess rejected with 429 and advisory payload
7. FR-M35-007: concurrent batch processing from all three agencies without queue starvation; each agency uses an independent worker queue with bounded depth

Matching and confidence scoring:

8. FR-M35-008: National ID + biometric reference cross-match performed against voter register for every CRS death record; both fields required for confident match
9. FR-M35-009: confidence score (0.0-1.0) computed per proposed removal using deterministic rule-based engine; high-confidence records surfaced first in triage queue
10. FR-M35-010: auto-approve threshold configurable by the Commission policy; records below threshold mandatory for officer review before any action
11. FR-M35-011: deduplication: if voter already has a pending CRS event, incoming KRA dormancy flag is subordinated; no double triage event created; single queue entry maintained

Family notification workflow:

12. FR-M35-012: *400# MSISDN rate-limit enforced at 5 reports per MSISDN per calendar month; excess rejected with advisory SMS to caller
13. FR-M35-013: family provisional record requires CRS confirmation within 90 days; auto-expires to UNCONFIRMED_EXPIRED state with no removal if CRS confirmation not received within window
14. FR-M35-014: ABIS soft-hold triggered on voter's biometric template for any proposed removal from any source (CRS, NRB, KRA, family, officer manual); no new registration may reuse that biometric until the record is resolved

Officer triage console:

15. FR-M35-015: officer triage queue ranked by confidence score descending; columns: masked voter ID, proposed event type, source, event age, ABIS soft-hold status
16. FR-M35-016: officer actions: APPROVE, REJECT, HOLD; each requires a mandatory basis code from a statutory code list; all actions logged immutably with actor, timestamp, and basis code
17. FR-M35-017: record detail screen shows before-and-after view of proposed change; redacted source document link (full document accessible to officer only); decision history per record
18. FR-M35-018: batch ingestion dashboard shows per-agency tiles with last batch received, record counts, error count, next expected batch; drill-through to per-batch reconciliation report
19. FR-M35-019: officer triage console p99 response <= 500 ms under 200 concurrent officers; measured by synthetic load test at OAT gate
20. FR-M35-020: officer APPROVE of a proposed removal triggers creation of a DeregistrationCase in M36 within 60 seconds via internal gRPC call

Correction workflow:

21. FR-M35-021: NRB-sourced corrections applied to name, ID number, and date of birth fields in voter record after officer APPROVE decision via M02 BVR Back-End
22. FR-M35-022: full before-and-after audit trail for every correction stored immutably; retained 7 years per Elections Act s.8A
23. FR-M35-023: voter notification via SMS and postal mail for every applied correction where contact data exists; notification includes corrected fields and effective date
24. FR-M35-024: M03 ABIS biometric reference updated via M02 after NRB-sourced correction applied; ABIS soft-hold lifted on resolution

Gazette publication:

25. FR-M35-025: quarterly gazette notice PDF prepared from approved removals this quarter; Commissioner HSM-signs using P06 HSM credential with dual-control quorum
26. FR-M35-026: gazette notice PDF anchored in P05 Merkle chain at publication; Merkle inclusion proof stored with notice record; immutable after anchoring
27. FR-M35-027: gazette notice published on M11 public portal within 24 hours of Commissioner signing; publication event emits eel.bvr.gazette_published.v1 with exactly-once delivery

Register snapshots:

28. FR-M35-028: signed register snapshot created before any batch commit; snapshot record includes total_registered, root_hash (SHA-256 of sorted voter ID list), and merkle_root
29. FR-M35-029: snapshot creation completes in <= 60 seconds for register of up to 22M entries; uses incrementally maintained Merkle tree; full recompute only at gazette
30. FR-M35-030: any snapshot reproducible for audit or court submission; Merkle proof retrievable by COURT_REGISTRAR role via GET /v1/cvr/snapshots/{id}/proof
31. FR-M35-031: Commissioner HSM-signs each new snapshot at creation time; unsigned or improperly signed snapshot rejected by P05 before anchoring
32. FR-M35-032: snapshot list API paginated and rate-limited at 20 requests/hour for the Commission Admin role
33. FR-M35-033: court Merkle proof available within 2 hours of authenticated court-registrar request; measured end-to-end from API call to proof delivery

Batch reporting:

34. FR-M35-034: reconciliation report generated for every batch import (CRS, NRB, KRA); fields: records received, matched, unmatched, flagged, actioned, errors; report URL stored on BatchIngestionReport record
35. FR-M35-035: reconciliation reports pushed to M15 IAB Dashboard as aggregate churn metrics (records added, corrected, removed this quarter) without exposing individual records

Citizen self-check:

36. FR-M35-036: citizen queries own registration status using National ID hash at GET /v1/cvr/status/{national_id_hash}; response includes status badge (Active/Pending Review/Removed) and last-updated date; no internal flags or officer-action details exposed
37. FR-M35-037: self-check rate-limited at 30 requests/hour per IP; excess returns 429 with retry-after header
38. FR-M35-038: self-check surface WCAG 2.2 AA compliant; axe-core zero critical errors; tested at each release
39. FR-M35-039: self-check available in EN + SW + Kikuyu + Luo + Kamba via language selector defaulting to county-detected language

Notifications and language:

40. FR-M35-040: SMS and USSD communications sent in voter's county-detected language; supported: EN, SW, Kikuyu, Luo, Kamba; fallback to SW if county undetected
41. FR-M35-041: all notification strings ICU-plural-correct; no truncation for any supported locale at 150% pseudo-length; font fallback declared for all scripts

Register freeze:

42. FR-M35-042: register freeze applied at T-40 days before election day; no new deregistrations processed from T-40 through T+7
43. FR-M35-043: freeze enforced as OPA hard-close at transaction time; concurrent writes at the freeze boundary are all-or-nothing; no partial commits
44. FR-M35-044: court-ordered reinstatements during freeze require Commissioner HSM-signed exception record and dual-officer sign-off; exception logged to P05 Merkle chain

Security and compliance:

45. FR-M35-045: PGP signature of each batch file validated against the sending agency's registered public key before parsing; raw batch file not stored beyond processing window
46. FR-M35-046: DPA 2019 s.30(1)(b) lawful basis recorded for every processing action; DPIA registered with ODPC; basis code attached to every VoterRegisterEvent
47. FR-M35-047: CRS raw death-notification data purged after removal confirmed; audit trail of purge event retained 7 years; ODPC-accessible log confirms purge timestamp
48. FR-M35-048: all inputs to batch ingestion and API endpoints use parameterized queries only; Semgrep and sqlx compile-time checks enforced in CI; ZAP fuzz corpus validates
49. FR-M35-049: no voter PII (National ID, name, biometric hash) in telemetry traces or logs; Tier A/B fields redacted by telemetry middleware before export; CI log-scanner gate blocks any violation

Observability and operations:

50. FR-M35-050: OpenTelemetry traces exported to central collector; Grafana dashboard named `eel-iems-35`; alert ruleset `iems-35-alerts` in Alertmanager; both required at OAT gate per FAT-07
51. FR-M35-051: SFTP retry with exponential backoff on connection failure; alert after 3 consecutive failed attempts per agency; on-call SRE notified; officer triage queue not blocked by SFTP failure
52. FR-M35-052: module availability >= 99.9% in T-90 to T+7 window (cycle-critical class); >= 99.5% during non-election periods; SLO alert fires within 5 minutes of threshold breach

## Acceptance criteria

1. AC-M35-01: 100% of CRS batches processed end-to-end within 4 hours of SFTP receipt; measured by ingestion-to-reconciliation-report timestamp delta; no batch outstanding > 4 hours in any 30-day window
2. AC-M35-02: officer triage queue backlog <= 500 pending events at any time during non-election periods; measured by queue-depth monitor on eel-iems-35 dashboard; sustained breach triggers P2 page
3. AC-M35-03: register snapshot creation completes in <= 60 seconds for a register of up to 22M entries; measured by snapshot creation log; verified at OAT with production-scale synthetic dataset
4. AC-M35-04: false-positive proposed removals <= 0.1% of total CRS matches per quarter; measured by post-quarter audit comparing proposed removals to confirmed CRS deaths; sign-off from the Commission Registration Director and CRS
5. AC-M35-05: citizen self-check endpoint available >= 99.5% on a 30-day rolling window; SLO alert fires within 5 minutes if availability drops below threshold
6. AC-M35-06: 100% of gazette notices anchored in P05 Merkle chain before M11 publication; verified by Merkle reconciliation report generated after each gazette run; zero gaps tolerated
7. AC-M35-07: officer triage console p99 response <= 500 ms under 200 concurrent officers; verified by k6 load test at OAT gate; p99 must not exceed 550 ms at 3x load
8. AC-M35-08: *400# MSISDN rate-limit enforced at <= 5 reports per MSISDN per calendar month; verified by automated test injecting a 6th report from the same MSISDN; 6th must return advisory rejection with no provisional record created
9. AC-M35-09: every PGP-unsigned or signature-mismatched batch rejected before parsing; zero exceptions tolerated; verified by security test suite submitting corrupted and unsigned batches from CRS, NRB, and KRA stubs
10. AC-M35-10: CRS raw death-notification data purged within the processing window; audit log entries confirm purge with timestamp; verified by ODPC audit trail review at each quarterly deregistration cycle
11. AC-M35-11: module availability >= 99.9% in T-90 to T+7 window; <= 43.8 minutes total downtime per 30-day rolling window in that period; SLO alert fires within 5 minutes of threshold breach
12. AC-M35-12: Merkle proof for any snapshot retrievable via court-registrar API within 2 hours of authenticated request; verified by e2e test creating snapshot and timing proof retrieval under simulated court load
13. AC-M35-13: WCAG 2.2 AA on citizen self-check screen confirmed by axe-core automated scan (zero critical errors) and manual screen-reader review; verified at each release
14. AC-M35-14: multi-language notifications delivered in correct county-detected language in >= 99% of integration test cases across all five supported locales
15. AC-M35-15: reconciliation report generated for every batch import within 30 minutes of batch completion; measured by report_created_at minus batch_received_at; verified at OAT with CRS, NRB, and KRA stubs

## Applicable edge cases

- EC-DAT-01: duplicate CRS batch submission with same idempotency key returns cached 200 response without reprocessing; no duplicate proposed-removal records created
- EC-DAT-02: same CRS batch idempotency key submitted with different body returns 422 idempotency-key-reuse; original batch preserved; alert logged
- EC-DAT-06: CRS batch contains events with schema v1 and v2 in same file; both accepted after Karapace compat check; dual-emit to downstream topics
- EC-DAT-07: partial CRS batch failure (e.g. 30 malformed records out of 2,000); malformed rows written to per-row error report; valid records committed; batch not rolled back
- EC-DAT-08: register snapshot at 22-30M entries streams with bounded memory; checkpoint-resumable if interrupted; verified by kill-resume harness
- EC-DAT-09: voter names with multi-codepoint Unicode characters (Somali, Arabic script) stored as NFC; ICU collation applied in triage queue; grapheme-safe; no cluster split
- EC-DAT-11: CRS death record ID number has transcription error matching live voter; confidence score below auto-approve threshold; officer reviews; rejection basis logged; voter not removed
- EC-DAT-13: soft-deleted (REMOVED) voter record does not appear as Active in self-check surface; tombstone honoured in projection layer
- EC-DAT-14: oversized event (>1 MB) produced during gazette publication; outbox rejects inline payload; blob stored in MinIO with SHA-256 hash reference; downstream consumers retrieve from MinIO
- EC-DAT-15: KRA dormancy flag arrives for voter already in CRS proposed-removal queue; KRA flag subordinated; deduplication layer prevents double triage event; single queue entry maintained
- EC-SEC-01: unsigned or signature-mismatched batch submitted by agency; rejected before parsing; alert generated to SOC; no data ingested
- EC-SEC-03: HSM unavailable during Commissioner gazette signing; gazette signing pauses and queues; ingest continues; degraded-mode alert raised; no snapshots signed until HSM restored
- EC-SEC-05: SQL injection attempted via malformed batch field or API query parameter; parameterized queries and sqlx compile-time checks prevent execution; fuzz corpus confirms
- EC-SEC-16: officer or telemetry path attempts to log voter PII (National ID, name, biometric hash); Tier A/B redaction middleware strips fields before export; CI log-scanner gate blocks violations
- EC-IDN-02: JWT expires mid-officer triage request; in-flight action completes with token valid at dispatch; next request returns 401 with silent-refresh header; no data loss
- EC-IDN-10: SpiceDB unavailable during officer approval write; write fails closed; 503 problem+json returned; no partial state persisted
- EC-SCL-03: hot partition on eel.bvr.cvr.v1 topic due to skewed voter_id distribution; stream-id spread applied; lag alert fires; HPA scales consumers
- EC-SCL-04: Kafka consumer lag on eel.bvr.cvr.v1 exceeds 60 seconds; page fires to SRE; backpressure applied to upstream producers; no OOM condition
- EC-TIM-06: batch event timestamped before causation time (retroactive CRS death certificate); accepted; skew anomaly logged; projections remain consistent independent of wall-clock order
- EC-TIM-09: register freeze race at T-40: two concurrent write transactions attempt to approve removal at exact freeze boundary; OPA hard-close applied at transaction time; one succeeds, one rejected; all-or-nothing semantics verified by TLA+ model
- EC-CON-03: CRS SFTP unreachable; retry with exponential backoff; alert after 3 consecutive failures; officer triage queue and other agency batches unaffected
- EC-CON-04: site partition between primary and mirror; Patroni fencing prevents split-brain on register writes; MirrorMaker lag alert at 5 min; RPO <= 5 min verified by Chaos Mesh
- EC-LEG-03: s.44A invoked during election period; dual-officer S44AEvent required; no removal approvals processed during invocation window; ECC alert raised
- EC-OPS-07: NRB batch arrives 24 hours late; queue does not stall; next batch processed on arrival; alert raised after 72-hour delay; SRE escalation path triggered
- EC-OPS-08: disk full or bucket quota reached during batch ingestion; 507 returned to agency; bounded queue maintained; alert raised; no data corruption

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | CRS matching algorithm, confidence-score calculator, ID-correction deduplication, dormancy-flag cross-reference, snapshot root-hash computation | nextest | M35-UNIT- |
| property | snapshot root hash deterministic on same input set | proptest | M35-PROP- |
| contract | CRS/NRB/KRA SFTP batch ingest, ABIS soft-hold, M02 voter record update, M36 case creation | Pact | M35-PACT- |
| integration | end-to-end SFTP batch; ABIS soft-hold trigger; Merkle anchor; gazette publication | testcontainers | M35-INT- |
| e2e | simulate CRS batch with known test records; officer approval; gazette inclusion; Merkle proof; snapshot reproducibility | nextest | M35-E2E- |
| security | unsigned batch rejected; triage queue access without COMMISSION_OFFICER role denied; PII purge after processing window | ZAP + cargo-fuzz | M35-SEC- |
| a11y | WCAG 2.2 AA on citizen self-check screen | axe-core + manual | M35-A11Y- |
| load | 2,000 CRS records within 4-hour SLA under normal officer queue load | k6 | M35-LOAD- |
| chaos | CRS SFTP unreachable: retry + alert; NRB batch late: queue unblocked | Chaos Mesh | M35-CHAOS- |

Tags: `#[trace("FR-M35-005","AC-M35-04","EC-DAT-11")]`

## Runbooks and dashboards

- `docs/runbooks/bvr-svc-degradation.md` - extend with CVR batch ingestion failure paths
- `docs/runbooks/crs-batch-failure.md` - new runbook; SFTP retry; 72-hour alert escalation; manual batch upload procedure
- `docs/runbooks/gazette-publication.md` - new runbook; Commissioner HSM ceremony; Merkle anchor verification; M11 publication steps
- Grafana dashboard: `platform/observability/bvr-svc/cvr-dashboard.json` (named `eel-iems-35`)
- SLO alerts: `platform/observability/bvr-svc/cvr-slo-alerts.yaml` (alert ruleset `iems-35-alerts`; batch 4 h, snapshot 60 s)

## Open questions

- CRS, NRB, KRA MoUs: the Commission must negotiate post-award; Datacraft delivers technical spec; timeline risk if MoUs delayed past 2026-12-01.
- *400# USSD short-code: the Commission must reserve and test with Safaricom / Africa's Talking; process begins at contract start.
- Register freeze boundary at T-40: confirm boundary with the Commission Elections Director; any exceptions (e.g. court-ordered reinstatements) need a protocol.
- Snapshot latency at 22M entries: incremental Merkle tree implementation must be benchmarked against production data volume before G3 gate.
