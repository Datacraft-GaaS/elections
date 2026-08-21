# M69 long-term-archive

| field | value |
|---|---|
| M-number | M69 |
| Home service | support-admin-svc (ADR-0020) |
| Owning work package | WP-5.6 (docs/plans/06-wave-5-governance.md) |
| Size | L |
| Availability class | routine 99.5% |
| Event-sourced | no (ADR-0013: only M08/M09/M10, M20, M51, M63) |
| Upstream spec | work_docs/01-Projects/IEBC/sw/modules/M69-long-term-archive.md |

## Purpose

Creates, preserves, and manages BagIt-format archive collections of election records from M14, M10, M04, M20, M22, M63, and M66, issues DataCite DOIs, maintains preservation attestations, provides time-boxed research access, and issues destruction certificates on schedule under KNADS policy.

## Surfaces and languages

| surface | technology | path under services/support-admin-svc/ |
|---|---|---|
| Archive admin console | Django | django/hr/ (shared admin, archive section) |
| Public archive catalogue | Astro | astro/archive-public/ |
| BagIt ingest worker | Python | py/bagit/ |
| Internal API | Rust (axum) | crates/archive/ |

## Domain model

| entity | key fields or notes | data class | store |
|---|---|---|---|
| ArchiveCollection | id, election_id, collection_name, source_modules (JSONB list), bag_checksum, bagit_version, doi, status (ingest/sealed/attested/public/destroyed), created_at | C | T1-consensus |
| IngestJob | id, collection_id, source_module, source_ref, items_expected, items_ingested, status (pending/running/complete/failed), started_at | C | T1-consensus |
| PreservationAttestation | id, collection_id, attested_by, fixity_check_result, fixity_check_at, attestation_hash | C | T1-audit |
| DestructionCertificate | id, collection_id, destruction_basis, destroyed_at, witnessed_by, certificate_hash | C | T1-audit |
| ResearchAccessGrant | id, collection_id, researcher_id, institution, purpose, access_level, expires_at, granted_by | C | T1-audit |

## APIs

| method+path | tier | auth | idempotent write? | notes |
|---|---|---|---|---|
| GET /v1/archive/collections | 1 signed-static | public | n/a | public archive catalogue |
| GET /v1/archive/collections/{id} | 1 signed-static | public | n/a | collection metadata, DOI, fixity |
| POST /v1/archive/collections | 3 accredited | JWT+SpiceDB | yes | create archive collection |
| POST /v1/archive/collections/{id}/ingest | 3 accredited | JWT+SpiceDB | yes | trigger ingest job for source module |
| GET /v1/archive/collections/{id}/ingest/{job_id} | 3 accredited | JWT+SpiceDB | n/a | ingest job status |
| PUT /v1/archive/collections/{id}/seal | 3 accredited | JWT+SpiceDB | yes | seal collection (no further ingest) |
| POST /v1/archive/collections/{id}/attest | 3 accredited | JWT+SpiceDB | yes | preservation attestation |
| POST /v1/archive/collections/{id}/access | 3 accredited | JWT+SpiceDB | yes | grant research access |
| POST /v1/archive/collections/{id}/destroy | 3 accredited | JWT+SpiceDB | yes | issue destruction certificate |
| GET /v1/archive/collections/{id}/format-migration | 3 accredited | JWT+SpiceDB | n/a | format migration status |
| POST /v1/archive/collections/{id}/format-migration | 3 accredited | JWT+SpiceDB | yes | trigger format migration job |

OpenAPI: packages/openapi-contracts/support-admin/long-term-archive.yaml

## Events

| event type | direction | topic | retention class | signed? | delivery |
|---|---|---|---|---|---|
| eel.archive.collection_sealed.v1 | emits | eel.archive.collection.v1 | operational | no | at-least-once idempotent |
| eel.archive.preservation_attested.v1 | emits | eel.archive.collection.v1 | operational | no | at-least-once idempotent |
| eel.archive.collection_destroyed.v1 | emits | eel.archive.collection.v1 | operational | no | at-least-once idempotent |
| eel.archive.research_access_granted.v1 | emits | eel.archive.collection.v1 | operational | no | at-least-once idempotent |

## Integrations

| system | I-code or name | adapter path | double available in dev? | failure behaviour |
|---|---|---|---|---|
| M14 reproducibility-bundle | internal | crates/archive/ | yes | hard dependency; bundle ingested |
| M10 rts-backend (tally data) | internal | crates/archive/ | yes | hard dependency; tally records ingested |
| M04 VPRS (pre-registration) | internal | crates/archive/ | yes | advisory |
| M20 inventory-management | internal | crates/archive/ | yes | advisory |
| M22 campaign-finance | internal | crates/archive/ | yes | advisory |
| M63 ballot-serial-chain | internal | crates/archive/ | yes | advisory |
| M66 election-financial-forensics | internal | crates/archive/ | yes | advisory |
| KNADS (Kenya National Archives) | knads | integrations/knads/ | yes | circuit breaker; submission queued |
| DataCite | datacite | integrations/datacite/ | yes | circuit breaker; DOI queued |

## ML and AI

none

## Dependencies

- Modules: M14, M10, M04, M20, M22, M63, M66
- Platform: P01 (SpiceDB), P04 (Kafka outbox), MinIO (bag storage)
- Packages: eel-outbox-rs, eel-resilience-rs, eel-web-py, eel-web-astro

## Feature checklist

Data source registry:

1. FR-M69-001: Register every operational data source eligible for archival: source system ID, data category, retention period (7/10/15 years), classification (public/restricted/restricted_judicial), PII flag, and lawful basis for retention. (FR-69.01)
2. FR-M69-002: Reject source registration missing lawful basis or retention period with 422 and eel_error_code: source_invalid; DPIA reference required for any PII-flagged source before ingest begins. (FR-69.01)

Ingestion pipeline:

3. FR-M69-003: Create archive collection per election cycle with ordered source module list; collection status follows FSM: ingest, sealed, attested, public, destroyed. (FR-69.02)
4. FR-M69-004: Accept BagIt-format ingest packages at defined milestones: M14 at T+4h, M10 at T+24h, M20 and M63 at T+30d, M22 and M66 at T+90d, M04 and P05 annually. (FR-69.02)
5. FR-M69-005: Validate BagIt fixity on ingest: compute SHA-256 of every file; compare against bag manifest; reject any manifest mismatch and fire a P1 alert. (FR-69.02)
6. FR-M69-006: Record per-ingest run metadata: record count, hash manifest, source system version, and ingest timestamp; store in IngestJob entity with status (pending/running/complete/failed). (FR-69.02)
7. FR-M69-007: Ingest jobs are checkpoint-resumable; kill-and-resume produces zero duplicate items; resume reads from last committed checkpoint at 1M-row granularity. (FR-M69-003)
8. FR-M69-008: Ingest pipeline streams data without loading full dataset into memory; maximum memory footprint <= 512 MB per ingest worker on 22-30M row datasets. (EC-DAT-08)
9. FR-M69-009: Partial batch failure produces per-row error report; valid rows committed; failed rows re-queued with exponential backoff; ingest job does not abort on individual row failure. (EC-DAT-07)
10. FR-M69-010: Ingest events with event_time preceding causation_time are accepted; record stored with an audit annotation noting the clock skew anomaly. (EC-TIM-06)
11. FR-M69-011: Duplicate ingest submission with same collection_id and manifest hash returns cached 202 with no re-execution of the fixity pipeline. (EC-DAT-01)
12. FR-M69-012: Duplicate ingest idempotency key reused with a different manifest body returns 422 eel_error_code: idempotency_key_reuse. (EC-DAT-02)

Sealing and format management:

13. FR-M69-013: Seal collection after all configured ingest jobs complete; sealed collection rejects further ingest with 423; eel.archive.collection_sealed.v1 emitted at-least-once. (FR-M69-004)
14. FR-M69-014: At sealing, render all structured data to open canonical formats: CSV and JSON-LD for structured data, PDF/A-3 for documents, FLAC for audio, BagIt container for delivery. (FR-69.03)
15. FR-M69-015: Format migration pipeline triggered at 5-year intervals or on Archivist request: render to new open canonical, verify byte-count and SHA-256 against pre-migration manifest, generate signed migration attestation. (FR-69.03)
16. FR-M69-016: Migration log retained alongside data; FormatMigration entity stores pre and post SHA-256 hashes and archivist UUID; KNADS counter-signature recorded. (FR-69.03)
17. FR-M69-017: BagIt ingest from M14 produces bit-identical BagIt bag across x86_64 and aarch64 platforms; verified by cross-platform ingest test in CI. (FR-M69-020)

KNADS deposit:

18. FR-M69-018: Deposit designated collections to KNADS via SFTP plus PGP within 90 days of election; KNADS accession number recorded; circuit breaker queues deposit on KNADS unavailability. (FR-69.04)
19. FR-M69-019: KNADS deposit receipt (signed PDF) stored in M69 attestation store; deposit status and accession number visible in Archivist dashboard. (FR-69.04)
20. FR-M69-020: KNADS submission retried with exponential backoff on transient failure; circuit breaker opens after three consecutive failures; P1 alert fired on circuit open; delivery resumes on recovery. (EC-CON-08)

Preservation attestation:

21. FR-M69-021: At each checkpoint (T+1y, T+5y, T+10y, T+15y): IEBC Archivist generates a preservation attestation signed by HSM key; KNADS counter-signature required before attestation is published. (FR-69.05)
22. FR-M69-022: Attestation content: manifest SHA-256, current format, storage locations, KNADS accession, and next migration date; published to M11 (public collections) and M15 IAB Dashboard. (FR-69.05)
23. FR-M69-023: Annual SHA-256 fixity re-verification of every archived file; P1 alert on any mismatch; alert fires within 24 hours of next scheduled verification cycle. (NFR-69.03)
24. FR-M69-024: All primary archive files stored in WORM object storage (MinIO object-lock, 15-year retention policy); any overwrite attempt rejected at the storage layer. (NFR-69.03)
25. FR-M69-025: Three-copy strategy enforced: Nairobi Tier III DC (primary WORM), KNADS physical site (deposit), offsite cold tape; storage locations recorded in ArchiveCollection.storage_locations. (NFR-69.01)

DOI and public access:

26. FR-M69-026: Mint DataCite DOI for each public collection version within 5 minutes of sealing; DOI resolves to M69 public landing page; circuit breaker queues on DataCite unavailability. (FR-69.07)
27. FR-M69-027: Public-access API (collection browser, dataset query, file download, citation generator) becomes available after embargo lifts: public at T+1y, restricted at T+5y after judicial close, PII never public. (FR-69.06)
28. FR-M69-028: Citation generator returns BibTeX, DataCite, and RIS formats on request; citation includes DOI, collection version, and attribution to the Commission. (FR-69.07)
29. FR-M69-029: Public API rate-limited to 10 requests per minute per authenticated researcher; anonymous access limited to collection metadata browse only; embargo enforced at data layer. (FR-69.06)
30. FR-M69-030: Cross-cycle link table maps constituency and station codes across 2017, 2022, and 2027 election cycles; DOI-linked per cycle; no internal system IDs exposed to researchers. (FR-69.10)

Research access:

31. FR-M69-031: Researcher submits access application: institution, research purpose, data categories, and IRB clearance reference; IEBC Research Committee decides within 30 days. (FR-69.09)
32. FR-M69-032: Approved researcher receives time-limited DUA URL; downloads are watermarked with researcher ID and access-grant UUID; access expires precisely at ResearchAccessGrant.expires_at. (FR-69.09)
33. FR-M69-033: Research access revoked automatically on expiry; download URL invalidated; revocation logged to P05; eel.archive.research_access_granted.v1 includes expires_at for downstream consumers. (FR-M69-010)
34. FR-M69-034: Denied application includes written reason; researcher may appeal to Commissioner; appeal decision recorded in audit trail. (FR-69.09)

Expiry and destruction:

35. FR-M69-035: At retention end, Archivist initiates review; PII collections: cryptographic erasure per DPA 2019 §41, destruction certificate, and ODPC notification within 30 days. (FR-69.08)
36. FR-M69-036: Destruction certificate requires dual-witness sign-off (IEBC Archivist plus KNADS counter); single-approver attempt returns 422 eel_error_code: dual_witness_required. (FR-M69-011)
37. FR-M69-037: Destroyed collection status permanently visible in the public catalogue; metadata retained; content URL returns 410 Gone; eel.archive.collection_destroyed.v1 emitted. (FR-M69-014)
38. FR-M69-038: Non-PII collections at retention end may be extended by IEBC resolution or transferred permanently to KNADS; transfer recorded in audit trail with authorising officer UUID. (FR-69.08)

Admin and public UI:

39. FR-M69-039: Admin console shows ingest job queue with per-collection progress, status, and estimated completion; filter by election cycle, status, and source module. (FR-M69-015)
40. FR-M69-040: Archivist dashboard shows fixity check schedule, upcoming attestation deadlines, format-migration queue, and expiry calendar with 90-day advance warnings. (FR-M69-016)
41. FR-M69-041: Public archive catalogue lists collections with classification badge, embargo status, DOI link, and download button (embargo-aware); WCAG 2.2 AA; EN and SW. (FR-M69-012 / NFR-69.06)
42. FR-M69-042: Attestation viewer shows per-collection attestation history, manifest hash, KNADS accession, and HSM signature verification link; accessible to IAB auditors and public. (FR-M69-006)
43. FR-M69-043: Signed PDF manifest export for Auditor-General per collection: record count, SHA-256 hash, storage locations, and full attestation chain. (FR-M69-017)

Compliance and internationalisation:

44. FR-M69-044: DPIA reference stored in DataSource entity; ingest blocked until DPIA record is present for any PII-flagged source; enforced by pre-ingest policy check. (NFR-69.04)
45. FR-M69-045: Public catalogue rendered in English and Swahili; academic API documentation in English; all API error messages include eel_error_code field. (FR-M69-018)
46. FR-M69-046: Public catalogue and researcher portal meet WCAG 2.2 AA; verified by automated axe-core scan before OAT gate. (NFR-69.06)

Observability and audit:

47. FR-M69-047: All collection state transitions emitted to P05 with eel_collection_id, eel_archivist_id, eel_source_module, and eel_cycle fields; queryable by IAB auditors. (NFR-M69-MONITORING)
48. FR-M69-048: OpenTelemetry traces exported to central collector; Grafana dashboard (eel-M69) and Alertmanager ruleset (iems-M69-alerts) present before OAT gate. (NFR-M69-MONITORING)
49. FR-M69-049: Kafka outbox (eel-outbox-rs) used for all eel.archive.* events; outbox retains events during KNADS or DataCite circuit-breaker open; events re-delivered on integration recovery. (EC-CON-11)
50. FR-M69-050: Debezium WAL slot lag monitored; Alertmanager alert fires at 60 s lag; disk guard prevents WAL bloat beyond configured limit; outbox retained during lag period with no loss. (EC-DAT-12)
51. FR-M69-051: Oversized event (bag manifest > 1 MB) rejected by outbox; oversized payload stored in MinIO; event carries content hash only; verified by fuzz-size unit test. (EC-DAT-14)

## Acceptance criteria

1. AC-M69-01: Ingest checkpoint-kill-resume produces zero duplicate items on a 30M-row synthetic dataset; kill-and-resume integration test exercises every 1M-row checkpoint boundary.
2. AC-M69-02: BagIt fixity validated on ingest; SHA-256 mismatch fires a P1 alert within 1 s of detection; annual re-verification completes within 48 hours for a 100 TB collection; bit-flip injection test confirms on single corrupted byte.
3. AC-M69-03: DataCite DOI minted within 5 minutes of collection seal; circuit breaker queues the mint on DataCite unavailability; DOI minted on recovery without operator intervention.
4. AC-M69-04: Research access URL and download token expire precisely at ResearchAccessGrant.expires_at; any request after expiry returns 403 with eel_error_code: access_expired; verified by time-boxed integration test.
5. AC-M69-05: Destruction certificate requires dual-witness sign-off; single-approver attempt returns 422 eel_error_code: dual_witness_required; SpiceDB policy evaluated at commit time to prevent race condition.
6. AC-M69-06: Format migration re-attestation confirms that all field-level values are preserved across format change; FormatMigration.record_count_match is true; SHA-256 of migrated content differs from pre-migration hash (encoding change) while field comparison confirms equivalence.
7. AC-M69-07: Public archive API (GET /v1/archive/collections) p99 <= 200 ms at routine steady-state load; service availability class: routine 99.5%.
8. AC-M69-08: KNADS submission retried with exponential backoff; circuit breaker opens after three consecutive failures and fires a P1 alert; deposit resumes on KNADS recovery without data loss.
9. AC-M69-09: WORM overwrite attempt rejected at the MinIO object-lock storage layer; integration test confirms PUT on a sealed object returns 403; object content unchanged after attempt.
10. AC-M69-10: PII-containing collection returns 403 to a public researcher before embargo lift; restricted-judicial researcher with valid active grant receives 200; embargo enforcement verified by integration test targeting the download endpoint.
11. AC-M69-11: Streaming ingest of 30M-row dataset stays within 512 MB memory footprint at peak; kill-resume resumes from last committed checkpoint without re-processing prior rows; verified by kill-resume load test.
12. AC-M69-12: Out-of-order ingest event (event_time < causation_time) accepted and stored with audit annotation; collection integrity check does not reject or reorder the event; verified by shuffled-replay property test.
13. AC-M69-13: Secrets (HSM key handles, KNADS PGP private key, researcher JWT, DataCite API token) produce zero hits in log-scanner fixture (Tier A/B patterns); CI gate blocks merge on any hit (EC-SEC-16).
14. AC-M69-14: SpiceDB unavailable during a destruction certificate write; write fails closed with 503 problem+json; public catalogue read remains available via cached responses; verified by PodChaos test (EC-IDN-10).
15. AC-M69-15: Injection attempted via researcher institution field, research purpose text, and ingest source_ref URL; all inputs parameterised; ZAP and Semgrep confirm zero raw interpolation (EC-SEC-05).
16. AC-M69-16: RTO for the archive service <= 1 hour; RPO <= 5 minutes; PITR drill documented in runbook before OAT gate; WORM primary remains readable throughout primary failure.
17. AC-M69-17: JWT expiry mid-ingest-trigger or mid-attestation returns 401 with silent refresh on next request; in-flight operation completes on the original token without data loss (EC-IDN-02).

## Applicable edge cases

All TIM rows (ingest timestamps, embargo windows, retention deadlines):

EC-TIM-01: ingest worker clock skew exceeds 5 minutes at job start; worker refuses to start ingest session until NTS or GNSS time is confirmed; time-fence FSM unit test with faked RTC.
EC-TIM-02: NTP/NTS unreachable for the full ingest day; last authoritative time plus monotonic clock used; ingest events tagged time_confidence=low; tolerance window applied before retention deadline enforcement.
EC-TIM-03: archive service backend clock jumps backward during a running ingest; HLC never regresses; skew alert fires; existing ingest job continues with HLC timestamps.
EC-TIM-04: DST or TZ change on Archivist workstation; all embargo lift dates and retention deadlines stored in UTC; display converted to the Archivist's gazetted time zone; no off-by-one on 7-year or 15-year expiry.
EC-TIM-05: leap-second timestamp (23:59:60) appears in an ingested audit-log extract; timestamp parser tolerates the value and normalises to 23:59:59 UTC without crash.
EC-TIM-06: ingest event_time precedes causation_time (out-of-order delivery from M10 or P05); record accepted and stored with audit annotation noting clock skew; projection logic does not depend on wall-clock order.
EC-TIM-07: s.14A extension shifts the election date, which changes the T+4h ingest milestone; ingest scheduler reloads milestones from the HSM-signed calendar amendment; forged extension without valid signature is rejected.
EC-TIM-08: petition window T+7 midnight boundary: petition-evidence retrieval requests arriving at T+6d 23h must be served within the 24-hour clock; embargo enforcement does not block judicial-tier access during active petition window.
EC-TIM-09: register freeze race at T-60 or T-30 causes a concurrent ingest job and a snapshot job to target the same collection; OPA hard-close prevents concurrent writes; all-or-nothing commit enforced; TLA+ freeze model verified.

All DAT rows (ingest data quality, idempotency, integrity):

EC-DAT-01: duplicate ingest submission with same collection_id and bag manifest SHA-256 returns cached 202 with no re-execution of fixity pipeline; auto-generated per-write endpoint test.
EC-DAT-02: duplicate ingest idempotency key reused with a different manifest body returns 422 eel_error_code: idempotency_key_reuse; prevents silent bag substitution.
EC-DAT-03: out-of-order ingest events from Kafka topic; ingest buffer waits until gap is filled or sends to DLQ; collection is not sealed while any expected source module's ingest is in-flight.
EC-DAT-04: replayed signed ingest event (counter-based): rejected on counter check; event published to eel.archive.tamper.v1; fuzz counter test confirms.
EC-DAT-05: poison ingest message (record count exceeds registered total for source): sent to DLQ with reason code; ingest partition continues; lag returns to 0 after DLQ drain.
EC-DAT-06: ArchiveCollection schema v1 (no doi field) and v2 coexist during migration; both accepted; dual-emit ensures KNADS and DataCite adapters receive correct shape.
EC-DAT-07: partial batch failure in a 30M-row ingest run; per-row error report produced; valid rows committed; failed rows re-queued; ingest job does not abort on individual row failure.
EC-DAT-08: 22-30M row ingest from M14 or M10; pipeline is streaming, checkpoint-resumable, and bounded to <= 512 MB memory; kill-resume integration test confirms no row loss or duplication.
EC-DAT-09: Unicode constituency or station names (Somali, Gikuyu, Arabic) in ingested records are stored as NFC-normalised; ICU collation applied in cross-cycle link table; no grapheme truncation in signed PDF manifest.
EC-DAT-10: zero records in an ingested source module extract (e.g. no financial forensics panel for a by-election cycle); ingest job completes with record_count: 0; no division-by-zero in attestation report; renders "0 records" in admin dashboard.
EC-DAT-11: ingested tally record where spoilt votes exceed valid votes, or total exceeds registered voters; record flagged in per-row error report; DLQ entry created; collection not sealed until archivist reviews anomalies.
EC-DAT-12: Debezium slot lag or WAL bloat on the operational source database during an export; alert fires at 60 s lag; outbox retained with no loss; disk guard prevents WAL growth beyond configured limit.
EC-DAT-13: destroyed collection in public catalogue; metadata permanently retained; content URL returns 410 Gone; purge job does not remove the ArchiveCollection metadata record or audit entries.
EC-DAT-14: oversized bag manifest event exceeds 1 MB outbox limit; outbox rejects inline payload; manifest stored in MinIO; event carries SHA-256 hash reference only; fuzz-size unit test confirms.
EC-DAT-15: IPRS returns two matching person records during a voter-roll snapshot ingest (used in M04 extract); AdjudicationCase created; no auto-merge; ingest pauses for that record; IPRS stub duplicate test confirms.
EC-DAT-16: legacy ingest payload from a pre-CloudEvents M14 bundle (pre-schema upgrade); shim upgrades the envelope while preserving the original signature; Pact shim-to-archive golden field corpus verifies fidelity.

Mandatory minimums and other applicable rows:

EC-DAT-01: (covered above)
EC-DAT-02: (covered above)
EC-SEC-05: injection attempted via researcher institution field, research purpose text, ingest source_ref URL, and bag manifest file paths; all inputs parameterised; Semgrep and ZAP confirm zero raw interpolation.
EC-SEC-07: path traversal on bag manifest file path or MinIO object key (e.g. ../../etc prefix); object key sanitised to UUID-keyed path before storage; fuzz corpus covers ../ sequences.
EC-SEC-04: KEK rotation mid-ingest; online re-wrap with new KEK; old KEK remains readable until all in-flight encrypted objects are re-wrapped; audit event per re-wrapped object; no data loss.
EC-SEC-16: HSM key handle, KNADS PGP private key, researcher JWT, and DataCite API token must not appear in OpenTelemetry traces or application logs; log-scanner fixture in CI enforces zero Tier A/B matches.
EC-IDN-02: JWT expires mid-ingest-trigger or mid-attestation write; in-flight operation completes on existing token; next request receives 401 and client silently refreshes.
EC-IDN-10: SpiceDB unavailable during a destruction certificate write or research access grant; write fails closed with 503 problem+json; read-only public catalogue remains available via cached responses.
EC-SCL-04: consumer lag on eel.archive.collection.v1 topic exceeds 60 s; Alertmanager pages on-call; backpressure applied; no event loss; slow-consumer integration test confirms.
EC-CON-08: DataCite or KNADS DNS poisoned or unreachable; hardcoded fallback endpoints used; circuit breaker opens; P1 alert fires; DOI mint and deposit queued for recovery.
EC-CON-11: Kafka ISR shrinks during a long ingest run; producer blocks; outbox accumulates; P1 page; no event loss; two-broker kill test confirms outbox drains on recovery.
EC-OPS-08: MinIO bucket quota or disk quota reached during bag upload; service returns 507 with eel_error_code: storage_quota_exceeded; alert fires; no partial bag written; ingest job paused.
EC-SCL-06: tiering lag on cold replay of an archived collection from MinIO cold tier; throttled at <= 10 MB/s to protect downstream consumers; snapshot-then-tail pattern applied; tiered-container test confirms.

## Test plan

| layer | what is covered | harness | test-ID prefix |
|---|---|---|---|
| unit | Ingest FSM, fixity check, DOI mint, access expiry | nextest (pytest) | M69-UNIT- |
| property | Ingest idempotency; fixity determinism; checkpoint-resume | proptest | M69-PROP- |
| contract | KNADS double; DataCite double; M14 bundle; M10 tally | Pact | M69-PACT- |
| integration | Full archive lifecycle; ingest kill-resume; attestation; destroy | nextest + testcontainers | M69-INT- |
| e2e | Public catalogue; admin ingest; research access | Playwright | M69-E2E- |
| fuzz | Bag manifest payload; fixity hash fields | cargo-fuzz | M69-FUZZ- |
| load | 30M-row ingest at throttled rate | k6 | M69-LOAD- |
| security | ZAP; path traversal; log scanner; KEK rotation | ZAP + Semgrep | M69-SEC- |
| i18n | 11 locales | custom | M69-I18N- |
| a11y | axe on public catalogue | axe-core | M69-A11Y- |

Tags: `#[trace("FR-M69-003","AC-M69-01","EC-DAT-08")]`

## Runbooks and dashboards

- docs/runbooks/support-admin-svc-degradation.md (extend with archive section)
- docs/runbooks/archive-fixity-failure.md (new)
- docs/runbooks/archive-format-migration.md (new)
- platform/observability/support-admin-svc/archive-dashboard.json
- platform/observability/support-admin-svc/slo-alerts.yaml (fixity failure alert, ingest lag)

## Open questions

- KNADS API and MoU status; doubles gate development until formal agreement signed.
- DataCite DataCite membership: IEBC DataCite membership application status unknown; DOI minting blocked until confirmed.
- Retention schedule for each data class under KNADS policy not yet finalised; destruction certificate schedule and dual-witness requirement to be confirmed with legal team.
