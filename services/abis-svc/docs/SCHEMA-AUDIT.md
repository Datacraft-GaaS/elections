# ABIS schema audit

Date: 2026-08-21

## Coverage

| Concern | Table | Migration | Notes |
|---|---|---|---|
| Template metadata | eel_template_index | V001 | Legacy pointer table; superseded operationally by eel_gallery_templates but kept for backward compat. FK to attestations wired in V026. |
| Canonical gallery | eel_gallery_templates | V020 | Modality-specific columns (fingerprint_minutiae, face_embedding, iris_iriscode, iris_mask); shard-partitioned via shard_id FK. |
| Match requests | eel_match_requests | V002 + V026 | Correlation IDs and blob-manifest FK added in V026. |
| Match results summary | eel_match_results | V003 | Outcome + confidence. |
| Ranked candidates | eel_match_candidates | V020 | One row per rank (1..k) with per-modality subscores; enables audit + adjudication drill-down. |
| Adjudication cases (legacy) | eel_adjudication_cases | V004 + V026 | Legacy schema; investigator FK to eel_reviewers wired in V026. |
| Adjudication cases (v2) | eel_adjudication_cases_v2 | V012 (in flight) | Agent-B replacement with full FSM. |
| Reviewer actions | eel_reviewer_actions | V013 (in flight) | Append-only with hash chain. |
| Panel votes | eel_panel_votes | V014 (in flight) | Tier-3 M-of-N. |
| Reviewer registry | eel_reviewers | V023 | LMS-certified adjudicators; hardware key optional for high tier. |
| Reviewer specialisations | eel_reviewer_specialisations | V023 | Per-modality proficiency 1..5 for routing. |
| Enclave attestations | eel_enclave_attestations | V005 | TDX/SEV-SNP verdicts. Additional per-report detail in V015 (in flight, agent C). |
| Enrolment jobs | eel_enrolment_jobs | V009 + V026 | UUID7 PK; FSM + orchestrator lease; correlation IDs + kit FK in V026. |
| Enrolment decisions | eel_enrolment_decisions | V010 | Gallery placement + merge lineage. |
| Quality reports | eel_quality_reports | V011 | Per-modality quality scores + retake reasons. |
| Shard topology | eel_shards | V020 | 0..255 with primary + replicas + status + rebalance metadata. |
| HNSW snapshots | eel_hnsw_snapshots | V020 | Per-shard checkpoints for crash recovery. |
| Demographic cohorts | eel_demographic_cohorts | V020 | Bloom filters per (county, sex, age_bucket) for pre-match pruning. |
| Kit registry | eel_kits | V021 | Source of truth for kit identity + attestation profile + station assignment. |
| Kit attestation events | eel_kit_attestation_events | V021 | Append-only hash chain per kit + nonce-replay uniqueness. |
| Vendor router registry | eel_vendor_registry | V021 | Modalities supported + priority + circuit state + health. |
| KEK generations | eel_kek_generations | V022 | Per-cycle KEK versions issued by HSM signer. |
| Template KEK bindings | eel_template_kek_bindings | V022 | Enables online rewrap on KEK rotation. |
| Blob manifest | eel_blob_manifest | V022 | Every probe_ref anchored; per-blob KEK/DEK + retention_until. |
| PAD events | eel_pad_events | V022 | Presentation-attack detection outcomes with attack type + action. |
| Fairness evaluations | eel_fairness_evaluations | V024 | Per-slice FMR/FNMR trending vs baseline + breach severity. |
| Model cards | eel_model_cards | V024 | Cosign-signed model artefact metadata. |
| Legacy KIEMS migration | eel_legacy_kiems_migration | V024 | Legacy -> eel subject mapping for WP-5.10 cutover. |
| Drift alerts | eel_drift_alerts | V006 | Model drift over configured tolerance. |
| Outbox | eel_abis_outbox | V007 | Transactional event publishing. |
| Audit shadow | eel_abis_audit_shadow | V008 + V025 | Monthly partitioned in V025. |
| Idempotency store | eel_idempotency_keys | V023 | 24h TTL default. |
| Updated_at triggers | eel_touch_updated_at | V025 | Applied to shards, reviewers, vendor_registry, enrolment_jobs, template_index. |
| Health view | eel_abis_health_v | V025 | Roll-up for dashboards + completeness scanner. |
| Gallery shard stats view | eel_gallery_shard_stats_v | V025 | Payload byte accounting per shard. |

## Indexing highlights

- Hot path 1:N lookup: `idx_eel_gallery_shard_modality` (shard_id, modality) partial WHERE retired_at IS NULL.
- Hot path candidate ranking: `idx_eel_match_candidates_result_rank` (result_id, rank).
- Orchestrator poll: `idx_eel_enrolment_jobs_status_created` partial WHERE status NOT IN terminal set.
- Attestation replay defence: `uq_eel_kit_attest_nonce` UNIQUE (kit_id, nonce).
- Idempotency: `PRIMARY KEY (key_hash)` on eel_idempotency_keys.
- Retention sweeper: `idx_eel_blob_manifest_retention` (retention_until) WHERE deleted_at IS NULL.
- Fairness breach dashboard: `idx_eel_fairness_breach` (evaluated_at DESC) WHERE breach = TRUE.
- Vendor router: `idx_eel_vendor_registry_health` (health_status, priority) WHERE enabled.

## Foreign key graph (post V026)

```
eel_shards <- eel_gallery_templates <- eel_template_kek_bindings -> eel_kek_generations
                                    ^        ^
                                    |        `- eel_blob_manifest -> eel_kek_generations
                                    |
eel_enrolment_decisions -----------`
eel_enrolment_jobs -> eel_kits (V026 FK)
eel_template_index -> eel_enclave_attestations (V026 FK)
eel_match_requests -> eel_blob_manifest (V026 FK)
eel_match_results -> eel_match_requests
eel_match_candidates -> eel_match_results, eel_shards
eel_adjudication_cases -> eel_match_results, eel_reviewers (V026 FK)
eel_reviewer_specialisations -> eel_reviewers
eel_pad_events -> eel_enrolment_jobs, eel_kits
eel_kit_attestation_events -> eel_kits
eel_model_cards -> eel_vendor_registry
```

## Retention (ADR-0016) enforced by

- `retention_until` column on eel_blob_manifest (per-row, per-class).
- `expires_at` on eel_idempotency_keys.
- Partition drop on eel_abis_audit_shadow_p (monthly rollover).
- TABLE COMMENTs on every retention-bearing table documenting the class + period; a background sweeper reads these and enforces via periodic deletes.

## Correlation

Every hot path table (match_requests, enrolment_jobs, adjudication_cases) now carries trace_id (16-byte W3C), span_id (8-byte), correlation_id (UUID), causation_id (UUID). Emitted events carry the same via CloudEvents eel_* extensions.

## Gaps intentionally left

- Fingerprint minutiae GIN index on quantised buckets: deferred until vendor SDK returns a stable quantisation; current design pre-filters via Bloom cohort + HNSW-ANN over face embeddings first, then exact fingerprint verify on the top-N, so the GIN would only help a fingerprint-only search which is not the primary flow.
- Face embedding pgvector column: intentionally BYTEA to keep the schema portable across PG installs; the operational index lives in the HNSW graph maintained in-process and periodically snapshotted to eel_hnsw_snapshots.
- Row-level security policies: to be added when Keycloak realm mapping to PG roles lands via eel-auth-rs (WP-1.3 downstream integration).

## In-flight collisions

Agent B is landing V012 - V014 (adjudication v2 tables). Agent C is landing V015 (attestation report detail). This audit's V020 - V026 sit above their range so no collision.
