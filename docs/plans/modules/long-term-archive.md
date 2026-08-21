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

1. FR-M69-001: Create archive collection per election cycle with source module list.
2. FR-M69-002: Ingest BagIt packages from each source module; validate fixity on ingest.
3. FR-M69-003: Checkpoint-resumable ingest; kill-and-resume produces no duplicates.
4. FR-M69-004: Seal collection after all ingest jobs complete; no further ingest allowed.
5. FR-M69-005: Mint DataCite DOI for sealed collection.
6. FR-M69-006: Periodic preservation attestation with fixity re-check.
7. FR-M69-007: Alert on fixity failure (bit-rot detection).
8. FR-M69-008: Submit collection metadata to KNADS.
9. FR-M69-009: Grant time-boxed research access with institution, purpose, and access level.
10. FR-M69-010: Revoke research access on expiry or on request.
11. FR-M69-011: Destruction certificate on schedule per KNADS retention policy; dual witness.
12. FR-M69-012: Public archive catalogue with collection metadata, DOI, and fixity hash.
13. FR-M69-013: Format migration job to convert obsolete formats; re-attest after migration.
14. FR-M69-014: Collection destroyed status permanently visible; no metadata purge.
15. FR-M69-015: Admin list of collections with filter by election, status, source module.
16. FR-M69-016: Audit trail of all collection state transitions.
17. FR-M69-017: Export collection manifest as signed PDF for OAG audit.
18. FR-M69-018: i18n on public catalogue in 11 languages.
19. FR-M69-019: WCAG 2.2 AA on public catalogue.
20. FR-M69-020: Ingest from M14 bundle produces bit-identical BagIt bag across x86_64 and aarch64.

## Acceptance criteria

1. AC-M69-01: Ingest checkpoint-kill-resume produces no duplicate items; kill-resume integration test (EC-DAT-08).
2. AC-M69-02: BagIt fixity validated on ingest and on periodic attestation; bit-flip injection test.
3. AC-M69-03: DataCite DOI minted within 5 min of seal; circuit breaker queues on unavailability; test.
4. AC-M69-04: Research access revoked on expiry; time-boxed URL expired; test.
5. AC-M69-05: Destruction certificate requires dual witness approval; SpiceDB policy test.
6. AC-M69-06: Format migration re-attestation confirms bit-identical reproduction; test.
7. AC-M69-07: API p99 within routine 99.5% SLA.
8. AC-M69-08: KNADS submission retried on failure; circuit breaker triggers after three attempts.

## Applicable edge cases

**Data:**
- EC-DAT-08: 22-30M row ingest from BVR/RTS; streaming; checkpoint-resumable; bounded memory.
- EC-DAT-07: partial batch failure in ingest; per-row report; valid items committed.
- EC-DAT-13: destroyed collection in catalogue; metadata retained; content URL gone.

**Security:**
- EC-SEC-07: path traversal on bag manifest URL; sanitised.
- EC-SEC-04: KEK rotation mid-ingest; online re-wrap; old readable until done.
- EC-SEC-16: researcher PII (institution, purpose) not in analytics logs.

**Connectivity:**
- EC-CON-08: DataCite/KNADS DNS poisoned; hardcoded fallback; circuit breaker.
- EC-OPS-08: disk full during ingest; bounded queues; 507; alert; no corruption.

**Time:**
- EC-TIM-06: ingest event time < causation time; accept; audit skew annotation.

**Scale:**
- EC-SCL-06: tiering lag on cold replay from MinIO; ≤10 MB/s throttle; snapshot-then-tail.

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
