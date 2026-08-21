# M35 cvr-continuous-voter-register

| field | value |
|---|---|
| M-number | M35 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | L |
| Availability class | cycle-critical 99.9% |
| Event-sourced | no |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M35-cvr-continuous-voter-register.md |

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
| GET /v1/cvr/events | 2 | JWT+SpiceDB (IEBC_OFFICER) | n/a | triage queue; proposed removals |
| PATCH /v1/cvr/events/{id} | 2 | JWT+SpiceDB (IEBC_OFFICER) | yes | approve / reject / hold event |
| GET /v1/cvr/snapshots | 2 | JWT+SpiceDB (IEBC_ADMIN) | n/a | list register snapshots |
| POST /v1/cvr/snapshots | 2 | JWT+SpiceDB (IEBC_COMMISSIONER) + HSM | yes | create and sign new snapshot; 2/day |
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

1. FR-M35-001: CRS death-feed ingestion; daily batch (SFTP + PGP-encrypted, schema-validated); match against voter register by National ID + biometric reference; generate proposed-removal records with confidence score
2. FR-M35-002: NRB ID-update ingestion; weekly batch; name/ID-number corrections; apply after officer review; update ABIS reference via M02
3. FR-M35-003: KRA-PIN dormancy signal; quarterly flag of PINs dormant >= 7 years; cross-reference with voter register; create "dormancy review" record; officer must confirm before any removal
4. FR-M35-004: family short-code intake; *400# USSD; family member enters ID number of deceased; provisional record created; requires CRS confirmation within 90 days; rate-limit 5/MSISDN/month
5. FR-M35-005: officer triage queue; cases ranked by confidence score; actions: approve, reject, hold with mandatory basis code; all actions logged with actor + timestamp
6. FR-M35-006: biometric cross-check before removal; any proposed removal triggers ABIS soft-hold so no new registration can reuse biometric until resolved
7. FR-M35-007: correction workflow for NRB-sourced changes; name, ID number, date of birth; full before-and-after audit trail; voter notified by SMS/post where contact exists
8. FR-M35-008: quarterly gazette; IEBC Commissioner HSM-signs gazette notice PDF; anchored in P05 Merkle chain; published on M11 public portal
9. FR-M35-009: register version snapshots; signed snapshot of root hash before any batch commit; reproducible for audit or court submission
10. FR-M35-010: voter self-check via M13 Public Verifier App by National ID hash; returns current status and last-updated date; no internal flags exposed
11. FR-M35-011: bulk-correction audit; every batch import produces reconciliation report (received, matched, unmatched, flagged, actioned); logged to IAB dashboard
12. FR-M35-012: multi-language notifications; EN + SW + Kikuyu + Luo + Kamba as detected from voter's registered county
13. CRS batch processed within 4 hours for up to 2,000 records; officer triage p99 <= 500 ms under 200 concurrent officers; snapshot creation <= 60 s for 22M entries
14. DPA 2019 lawful basis: s.30(1)(b) legal obligation (Elections Act s.6A/s.8A); DPIA registered with ODPC; CRS death data purged after removal confirmed; audit trails retained 7 years
15. Batch files validated against PGP signature of sending agency before parsing; raw CRS/NRB data not stored beyond processing window
16. Register freeze at T-40: no new deregistrations processed from T-40 to T+7

## Acceptance criteria

1. AC-M35-01: 100% CRS batches processed within 4 hours; ingestion timestamp delta
2. AC-M35-02: officer triage queue backlog <= 500 pending events at any time (non-election); queue depth monitor
3. AC-M35-03: register snapshot created in <= 60 seconds; snapshot creation log
4. AC-M35-04: false-positive proposed removals <= 0.1% of CRS matches; post-quarter audit by IEBC + CRS
5. AC-M35-05: citizen self-check availability >= 99.5%; uptime monitor
6. AC-M35-06: 100% gazette notices anchored in Merkle chain; Merkle reconciliation report

## Applicable edge cases

**connectivity**
- EC-CON-03: CRS SFTP unreachable; retry with exponential backoff; alert after 3 failed attempts; do not block officer queue

**data**
- EC-DAT-11: CRS death record ID number has transcription error matching live voter; confidence score below auto-approve threshold; officer reviews; rejection basis logged
- EC-DAT-15: KRA dormancy flag for voter already in CRS proposed-removal queue; KRA flag subordinated; deduplication layer

**security**
- EC-SEC-01: unsigned batch submitted; rejected before parsing; alert to Datacraft SOC
- EC-SEC-03: *400# rate-limit exceeded per MSISDN; intake blocked; operator notified

**legal**
- EC-LEG-04: Elections Act s.6A continuous registration; elections Act s.8A register correction; gazette notice is constructive notice where voter unreachable

**operations**
- EC-OPS-02: NRB batch arrives 24 hours late; queue does not stall; next batch processed on arrival; alert after 72-hour delay
- EC-OPS-07: snapshot latency exceeds 60 s at 22M entries; pre-computed incremental Merkle tree updated continuously; full recompute only on gazette

**scale**
- EC-SCL-03: 10,000 concurrent deregistration cases during post-census catch-up; worker pool auto-scales; KRA deduplication prevents double processing

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | CRS matching algorithm, confidence-score calculator, ID-correction deduplication, dormancy-flag cross-reference, snapshot root-hash computation | nextest | M35-UNIT- |
| property | snapshot root hash deterministic on same input set | proptest | M35-PROP- |
| contract | CRS/NRB/KRA SFTP batch ingest, ABIS soft-hold, M02 voter record update, M36 case creation | Pact | M35-PACT- |
| integration | end-to-end SFTP batch; ABIS soft-hold trigger; Merkle anchor; gazette publication | testcontainers | M35-INT- |
| e2e | simulate CRS batch with known test records; officer approval; gazette inclusion; Merkle proof; snapshot reproducibility | nextest | M35-E2E- |
| security | unsigned batch rejected; triage queue access without IEBC_OFFICER role denied; PII purge after processing window | ZAP + cargo-fuzz | M35-SEC- |
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

- CRS, NRB, KRA MoUs: IEBC must negotiate post-award; Datacraft delivers technical spec; timeline risk if MoUs delayed past 2026-12-01.
- *400# USSD short-code: IEBC must reserve and test with Safaricom / Africa's Talking; process begins at contract start.
- Register freeze boundary at T-40: confirm boundary with IEBC Elections Director; any exceptions (e.g. court-ordered reinstatements) need a protocol.
- Snapshot latency at 22M entries: incremental Merkle tree implementation must be benchmarked against production data volume before G3 gate.
