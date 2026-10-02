# M53 voter-id-card

| field | value |
|---|---|
| M-number | M53 |
| Home service | bvr-svc (ADR-0020) |
| Owning work package | WP-4.10 (docs/plans/05-wave-4-operations.md) |
| Size | XL (flagged) |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/the Commission/sw/modules/M53-voter-id-card.md |

## Purpose

Feature-flagged module for voter ID card issuance: manages card batch production orders to the NRB card manufacturer, collection records, blacklist maintenance, and card deactivation lifecycle integrated with the BVR voter register.

## Surfaces and languages

| surface | technology | path under services/bvr-svc/ |
|---|---|---|
| Public card collection status | Astro / Django | astro/bvr-public/ |
| Admin card batch management | Django | django/bvr-admin/ |
| Internal API | Rust (axum) | crates/voter-card/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| VoterCard | id, voter_id, card_number, batch_id, status (ordered/printed/dispatched/collected/deactivated/blacklisted), issued_at | B | T1-consensus |
| CardBatch | id, election_id, constituency_id, order_count, manufacturer_ref, status, ordered_at, received_at | C | T1-consensus |
| CollectionRecord | id, card_id, voter_id, collected_at, station_id, officer_id | B | T1-audit |
| BlacklistEntry | id, card_number, reason, blacklisted_at, blacklist_source | C | T1-consensus |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/voter-cards/status | 2 live-per-user | JWT | n/a | voter's own card status |
| POST /v1/voter-cards/batches | 3 accredited | JWT+SpiceDB | yes | create card batch order |
| GET /v1/voter-cards/batches/{id} | 3 accredited | JWT+SpiceDB | n/a | batch status |
| POST /v1/voter-cards/collections | 3 accredited | JWT+SpiceDB | yes | record card collection |
| POST /v1/voter-cards/blacklist | 3 accredited | JWT+SpiceDB | yes | add card to blacklist |
| DELETE /v1/voter-cards/blacklist/{card_number} | 3 accredited | JWT+SpiceDB | yes | remove from blacklist |
| GET /v1/voter-cards/blacklist | 3 accredited | JWT+SpiceDB | n/a | blacklist query (BVI integration) |
| PUT /v1/voter-cards/{id}/deactivate | 3 accredited | JWT+SpiceDB | yes | deactivate card |
| GET /v1/voter-cards/blacklist/export | 4 bulk | JWT+SpiceDB | n/a | blacklist export for BVI kits |

OpenAPI: packages/openapi-contracts/bvr/voter-id-card.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.bvr.voter_card_issued.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_card_blacklisted.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_card_deactivated.v1 | emits | eel.bvr.voter_card.v1 | operational | no | at-least-once idempotent |
| eel.bvr.voter_register_snapshot_ready.v1 | consumes | eel.bvr.voter_register.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| NRB card manufacturer | card-manufacturer | integrations/card-manufacturer/ | yes | circuit breaker; order queued |
| M06 BVI backend (blacklist sync) | internal | crates/voter-card/ | yes | advisory; BVI kits poll blacklist |
| M04 VPRS (pre-registration link) | internal | crates/voter-card/ | yes | advisory |
| M02 BVR backend (voter register) | internal | crates/voter-card/ | yes | hard dependency for voter validation |

## ML and AI

none

## Dependencies

- Modules: M02 (voter register), M04 (pre-registration), M06 (BVI blacklist sync)
- Platform: P01 (SpiceDB), P04 (Kafka outbox)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-crypto-rs, eel-web-py

## Feature checklist

Feature flag and module lifecycle:

FR-M53-001: Feature flag gates entire module; all card endpoints return 404 when flag is inactive; flag state controlled via Commission resolution event (spec FR-M53-001).
FR-M53-002: Feature flag state change logged with timestamp, resolution reference, and operator ID; audit event emitted to eel.bvr.voter_card.v1.
FR-M53-003: Module health endpoint reports flag state; eel-iems-53 Grafana dashboard reflects flag status in real time.

Card batch production:

FR-M53-004: Create card production batch linked to election ID and constituency code; batch ID is UUID7 (spec FR-M53-004).
FR-M53-005: Validate constituency exists in M04 register before batch creation; unknown constituency returns 422.
FR-M53-006: Submit batch order to NRB card manufacturer API via mTLS-authenticated channel; TLS 1.3 minimum; no downgrade permitted (spec FR-M53-004).
FR-M53-007: Batch order payload encrypted with manufacturer public key before transmission; plaintext personalisation data never stored at intermediate parties.
FR-M53-008: Track batch status through full lifecycle: ORDERED, PRINTING, QUALITY_CHECK, DISPATCHED, RECEIVED, COMPLETE (spec FR-M53-005).
FR-M53-009: Receive batch receipt from manufacturer with estimated delivery date; store manufacturer reference on batch record.
FR-M53-010: Alert operations when batch status has not advanced from DISPATCHED within 7 days; alert emitted to eel-iems-53-alerts.
FR-M53-011: Support batch cancellation with mandatory reason code; cancelled batches retained in list with CANCELLED status.
FR-M53-012: Batch order idempotent on manufacturer reference; re-submit of same reference returns existing receipt without creating duplicate batch.

Card personalisation and assignment:

FR-M53-013: Validate voter exists in M02 voter register before assigning card serial; absent voter returns 422 (spec FR-M53-002).
FR-M53-014: Card personalisation data package includes voter ID hash (not raw voter ID), constituency code, polling station code, and card serial; no biometric template included (spec FR-M53-002, FR-M53-003).
FR-M53-015: QR payload generated as signed hash of personalisation fields using P06 HSM key; signature verifiable offline (spec FR-M53-002).
FR-M53-016: RFID chip payload option (feature-flagged per tender scope): same fields as QR payload, HSM-signed; no biometric template on chip (spec FR-M53-003).
FR-M53-017: Assign card serial to voter idempotently; second assignment of same serial for same voter returns existing card record.
FR-M53-018: Card serial format configurable per manufacturer specification; sequential batch-scoped identifier.
FR-M53-019: Card number hashed in all application log output; raw serial stored only in encrypted audit store.

Card collection workflow:

FR-M53-020: Record card collection at constituency station: officer ID, station ID, collected_at timestamp (spec FR-M53-006).
FR-M53-021: Require biometric confirmation event reference from M01 or M08 before setting card status to COLLECTED; confirmation reference stored on collection record (spec FR-M53-006).
FR-M53-022: Prevent duplicate collection for same card serial; second attempt returns 409 with existing collection record reference (spec FR-M53-006).
FR-M53-023: Voter receives SMS confirmation on card collection with timestamp and officer reference within 60 s.
FR-M53-024: Collection record includes voter identity confirmation method: BIOMETRIC, ID_ONLY, or ASSISTED.
FR-M53-025: Batch collection mode: officer marks an entire batch received and distributed; triggers individual card status updates to DELIVERED.

Blacklist management:

FR-M53-026: Add card to blacklist with reason code (LOST, STOLEN, DUPLICATE, DEACTIVATED) and source officer ID (spec FR-M53-009).
FR-M53-027: Blacklist addition for STOLEN reason requires dual-officer confirmation; LOST requires single officer.
FR-M53-028: Remove card from blacklist requires dual-officer approval and mandatory reason; audit event emitted (spec FR-M53-010).
FR-M53-029: Blacklist exported as signed bundle via GET /v1/voter-cards/blacklist/export; bundle includes version counter and P06 HSM signature (spec FR-M53-008).
FR-M53-030: BVI kits verify blacklist bundle signature before applying; unsigned or invalid-signature bundles rejected (spec FR-M53-008).
FR-M53-031: Blacklist propagated to all BVI polling station kit caches within 30 minutes of blacklist event; propagation lag monitored (spec FR-M53-011, NFR-M53-008).
FR-M53-032: Blacklist query endpoint returns card status in constant-time response regardless of presence; voter enumeration prevented.

Card deactivation and replacement:

FR-M53-033: Deactivate card when voter deregistration event received from M04; status set to DEACTIVATED; event emitted to topic (spec FR-M53-012).
FR-M53-034: Deactivate card on death notification from NRB integration; deactivation event anchored to P05 audit chain.
FR-M53-035: Replacement workflow: voter reports lost card; officer verifies biometrically; old serial blacklisted as LOST immediately; new card batch order created for single replacement card (spec FR-M53-007).
FR-M53-036: Replacement card record carries replacement_for_serial field linking to original card serial.
FR-M53-037: Voter may have only one ACTIVE card at a time; system blocks second active card issuance with 409.
FR-M53-038: Replacement turnaround target 5 working days; alert emitted if breach of SLA.

BVI and polling-day integration:

FR-M53-039: Polling station card verification: M08 kit sends card serial to BVI backend; BVI checks blacklist and voter eligibility; response p99 <= 2 s (spec FR-M53-008, NFR-M53-002).
FR-M53-040: Offline fallback: QR payload contains sufficient fields for local eligibility check when central server unreachable; kit uses locally cached blacklist bundle (spec FR-M53-009, NFR-M53-007).
FR-M53-041: Biometric-fallback mode: voter arrives without card; Returning Officer authorises biometric-only verification; fallback event logged with authorisation reference (spec FR-M53-009).
FR-M53-042: All card issuance, blacklist, and deactivation events emitted to eel.bvr.voter_card.v1 topic.
FR-M53-043: Kit blacklist cache refreshed from export endpoint at polling-day start and every 30 minutes during polling period.

Statistics and reporting:

FR-M53-044: Card statistics endpoint: cards ordered, printed, dispatched, collected, active, blacklisted per constituency (spec FR-M53-011).
FR-M53-045: Coverage rate metric: active cards divided by registered voters per constituency; reported as percentage.
FR-M53-046: Low-uptake alert: flag constituencies where collection rate is below 70% with 14 days remaining before polling day.
FR-M53-047: Export collection records as CSV per batch or per constituency for manual reconciliation.
FR-M53-048: Admin batch list filterable by constituency, status, and election; cursor-paginated.
FR-M53-049: Admin card list filterable by batch, status, and blacklisted flag; cursor-paginated.

Public voter-facing portal:

FR-M53-050: Public card status portal: voter enters voter number; response is one of ORDERED, DISPATCHED, READY_FOR_COLLECTION, COLLECTED, BLACKLISTED, DEACTIVATED with collection point address (spec FR-M53-013).
FR-M53-051: Public portal does not reveal voter biographic data; response is status code and collection point only.
FR-M53-052: Public portal supports 11 display languages; language detected from browser locale; manual override available.
FR-M53-053: Public portal WCAG 2.2 AA compliant; keyboard-navigable; full task completion confirmed by screen reader.
FR-M53-054: Voter enumeration protection on public portal: constant-time response for existing and non-existing voter numbers; delta p95 < 5 ms.
FR-M53-055: Rate limiting on public portal: maximum 10 requests per voter number per hour; 429 returned on excess.

Security and data protection:

FR-M53-056: All card personalisation data packages encrypted with manufacturer public key before transmission; no plaintext PII transmitted or stored at third-party (spec NFR-M53-004).
FR-M53-057: No biometric templates on card (QR payload or RFID chip); DPA 2019 data minimisation enforced; automated manifest scan confirms pre-transmission (spec NFR-M53-003).
FR-M53-058: Card data package manifest signed by Commission DPO before batch transmission is authorised.
FR-M53-059: mTLS channel to NRB manufacturer; certificate pinned; any TLS version below 1.3 refused by server.
FR-M53-060: Voter ID excluded from analytics events and OpenTelemetry trace attributes; Semgrep rule enforced in CI pipeline.
FR-M53-061: Card serial hashed in all application log lines; raw serial accessible only in encrypted audit store.
FR-M53-062: Parameterised queries on all card serial and voter ID inputs; SQL injection impossible by construction; Semgrep compile-time check.
FR-M53-063: SpiceDB authorisation checked on all write endpoints; unauthenticated or unauthorised requests return 401 or 403 with problem+json body.

Audit and compliance:

FR-M53-064: Audit trail of all card status transitions: previous status, new status, actor, timestamp, and reason (spec FR-M53-016).
FR-M53-065: Card issuance and blacklist events anchored to P05 Merkle audit chain.
FR-M53-066: Decommission purge: all card personalisation files purged per DPA 2019 s.41 schedule; disposal certificate generated.
FR-M53-067: Card data retained until next election cycle or 7 years maximum; automated purge job with alerting on overdue purge (spec NFR-M53-003).

Operational and monitoring:

FR-M53-068: Module Grafana dashboard eel-iems-53 with panels: batch status timeline, collection rate by constituency, blacklist size, BVI verification latency p99.
FR-M53-069: Alert when BVI blacklist export endpoint p99 exceeds 500 ms for 5 consecutive minutes.
FR-M53-070: Alert when blacklist propagation lag to any kit cache exceeds 35 minutes.
FR-M53-071: OpenTelemetry traces exported for all card batch and verification operations; trace ID returned in API response headers.
FR-M53-072: Circuit breaker on NRB manufacturer channel: open after 5 consecutive failures; batch queued in outbox; alert emitted; circuit half-open retry after 60 s.

## Acceptance criteria

AC-M53-01: Feature flag disables all card endpoints when inactive; all requests return 404; verified by automated contract test at FAT gate.
AC-M53-02: Duplicate card collection for same serial rejected with 409; idempotency verified by property test over 10k random serials.
AC-M53-03: Blacklist export bundle signed by P06 HSM and verifiable by BVI kit; Pact contract test against BVI double.
AC-M53-04: Blacklist propagated to all BVI kit caches within 30 minutes of blacklist event; integration test with 65k-kit simulator confirms lag <= 30 min.
AC-M53-05: Batch order idempotent on manufacturer reference; second submission returns same receipt with no duplicate batch; unit test.
AC-M53-06: Voter absent from M02 register rejected with 422 and descriptive problem+json body; integration test.
AC-M53-07: Voter ID absent from all log output of card endpoints; CI log-scanner fixture confirms.
AC-M53-08: Card serial hashed in all log lines; raw serial never emitted; Semgrep rule passes in CI.
AC-M53-09: API p99 latency <= 500 ms at routine 99.5% availability class under 5x load; k6 load test at pre-prod scale.
AC-M53-10: Public portal voter enumeration response time delta < 5 ms between existing and non-existing voter number at p95; timing oracle test.
AC-M53-11: No biometric template present in card personalisation data package before transmission; automated manifest scan passes.
AC-M53-12: Blacklist export endpoint p99 <= 500 ms under 65k concurrent kit poll load; load test with Gatling simulator.
AC-M53-13: BVI card verification response p99 <= 2 s including network round-trip to central server; kit telemetry confirmed in integration test (spec NFR-M53-002).
AC-M53-14: Full 22M-record card personalisation batch generation completes within 72 h on reference hardware; performance test (spec NFR-M53-001).
AC-M53-15: Manufacturer channel enforces TLS 1.3 minimum; TLS 1.2 refused; testssl.sh scan passes.
AC-M53-16: WCAG 2.2 AA on public card status portal; axe-core zero critical violations; manual screen-reader full-task-completion test passes.
AC-M53-17: 11 languages rendered without truncation on public portal; 150% pseudo-locale test and all 11-locale snapshots pass.
AC-M53-18: Card data retention automated purge job removes records beyond 7-year maximum; tested with synthetic aged records; disposal certificate generated.
AC-M53-19: SpiceDB authorisation enforced on all write endpoints; unauthenticated write returns 401 problem+json; unauthorised role returns 403.
AC-M53-20: Offline QR verification on BVI kit returns correct ELIGIBLE or BLACKLISTED result without central server; kit simulator test with network blocked.

## Applicable edge cases

EC-DAT-01: Duplicate card collection submission for same serial; idempotent response returns existing collection record.
EC-DAT-02: Same batch order key with different constituency code; 422 idempotency-key-reuse returned; no second batch created.
EC-DAT-03: Out-of-order batch status events from manufacturer; buffered until sequence gap filled or pushed to DLQ.
EC-DAT-07: Partial batch failure (some cards fail printing); per-card status report generated; valid cards advanced to QUALITY_CHECK.
EC-DAT-08: 22M-record personalisation batch; streaming pipeline; checkpoint-resumable on failure; bounded memory; kill-resume test.
EC-DAT-09: Voter name contains Unicode characters (Swahili diacritics, Arabic script); NFC normalised; grapheme-safe truncation; 11-locale snapshots pass.
EC-DAT-13: Deactivated card appears in admin card list; DEACTIVATED tombstone honoured; card not treated as active for any purpose.
EC-SEC-05: SQL injection on card serial or voter ID query parameter; parameterised query blocks attempt; Semgrep compile-time check and ZAP scan enforced in CI.
EC-SEC-10: Voter enumeration via public card status endpoint; constant-time response regardless of voter existence; timing oracle p95 delta < 5 ms.
EC-SEC-11: Timing side-channel on QR signature verification; constant-time compare enforced; dudect-style analysis passes.
EC-SEC-16: Voter ID and card serial appear in no log line or OpenTelemetry trace attribute; log-scanner CI fixture and Semgrep rule both pass.
EC-IDN-02: JWT expires mid-card-collection request; in-flight request completes; next request returns 401 with silent-refresh hint in WWW-Authenticate header.
EC-IDN-09: Dual-control blacklist-removal second approver account revoked after first approval; both identities re-validated at commit; operation rejected if second approver is revoked.
EC-IDN-10: SpiceDB unavailable during card write request; fail closed; 503 problem+json returned; no partial write committed.
EC-CON-01: NRB manufacturer API offline for 7 days; batch order queued in outbox; transmitted on reconnect; sequence gap audit confirms no loss.
EC-CON-05: Flapping manufacturer API; exponential backoff with jitter; retry budget enforced; no duplicate batch submitted.
EC-CON-09: IPsec tunnel to manufacturer drops mid-batch transmission; transmission resumes at sequence; idempotent re-send; no double batch.
EC-SCL-04: Consumer lag on eel.bvr.voter_card.v1 topic exceeds 60 s; alert paged; backpressure applied to producers; no OOM on consumer.
EC-SCL-07: 65k BVI kit simultaneous polls of blacklist export endpoint; SLA met at 1x load; p99 regression <= 10% at 5x; k6 Gatling simulator.
EC-HUM-06: Screen reader navigates public card status portal; full task completion without mouse confirmed; WCAG 2.2 AA axe-core scan passes.
EC-HUM-08: 11 display languages on public portal including long Swahili strings; no truncation; ICU plurals applied; font fallback stack tested.
EC-OPS-01: Kit battery dies mid-card-verification at polling station; verification result journalled before committing; exact resume on restart; no double token issued.
EC-OPS-03: Card batch arrives with broken seal in transit; custody event emitted; batch quarantined; replacement batch order created automatically.
EC-LEG-02: Court-ordered election postponement; BVI kits refuse card verification sessions after HSM-signed calendar amendment received; portal notice displayed.
EC-TIM-07: Commission extends card collection deadline via signed resolution; collection window fence reopened; forged extension rejected by HSM signature check.
EC-ML-06: Unsigned or tampered blacklist bundle pushed to BVI kits; cosign verification fails; kit refuses to apply update and alerts operations.
EC-KIT-04: Feature-flag card-enabled mode switch not propagated to all kits; unacked kits listed in alert queue; signed bundle resent until acked.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Card FSM, feature flag gate, blacklist logic | nextest | M53-UNIT- |
| property | Collection idempotency; blacklist propagation timing | proptest | M53-PROP- |
| contract | Manufacturer double; M06 BVI blacklist export | Pact | M53-PACT- |
| integration | Full card lifecycle with testcontainers; blacklist sync | nextest + testcontainers | M53-INT- |
| e2e | Public status portal; admin batch and collection | Playwright | M53-E2E- |
| security | ZAP; log scanner; enumeration timing | ZAP + Semgrep | M53-SEC- |
| i18n | 11 locales | custom | M53-I18N- |
| a11y | axe on public portal | axe-core | M53-A11Y- |

Tags: `#[trace("FR-M53-001","AC-M53-01","EC-SEC-10")]`

## Runbooks and dashboards

- docs/runbooks/bvr-svc-degradation.md (extend with voter-card section)
- docs/runbooks/voter-card-blacklist-sync-failure.md (new)
- platform/observability/bvr-svc/voter-card-dashboard.json
- platform/observability/bvr-svc/slo-alerts.yaml

## Open questions

- Scope activation: Commission resolution and legal basis for voter ID cards not yet confirmed; module remains feature-flagged until decision.
- NRB card manufacturer API specification and SLA not received; double must gate development until live credentials available.
