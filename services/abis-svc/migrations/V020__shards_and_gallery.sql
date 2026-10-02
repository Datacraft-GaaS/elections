-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- Sharding metadata + canonical gallery templates + HNSW snapshots
-- + demographic cohorts + match candidate ranks.
-- Enables the matcher to serve 1:N without joins across shards.

CREATE TABLE IF NOT EXISTS eel_shards (
	shard_id            SMALLINT    PRIMARY KEY CHECK (shard_id BETWEEN 0 AND 255),
	primary_node        TEXT        NOT NULL,
	replica_nodes       TEXT[]      NOT NULL DEFAULT '{}',
	key_range_lo        BYTEA       NOT NULL,
	key_range_hi        BYTEA       NOT NULL,
	status              TEXT        NOT NULL DEFAULT 'healthy'
	                                CHECK (status IN ('healthy','degraded','rebalancing','offline')),
	template_count      BIGINT      NOT NULL DEFAULT 0,
	size_bytes          BIGINT      NOT NULL DEFAULT 0,
	last_rebalanced_at  TIMESTAMPTZ,
	last_snapshot_at    TIMESTAMPTZ,
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_eel_shards_status ON eel_shards(status) WHERE status <> 'healthy';

-- Canonical operational gallery. Modality-specific columns so the matcher
-- can pull directly without JSON parsing. Encrypted at rest via KEK bindings.
CREATE TABLE IF NOT EXISTS eel_gallery_templates (
	template_id         UUID        PRIMARY KEY,
	subject_id          UUID        NOT NULL,
	shard_id            SMALLINT    NOT NULL REFERENCES eel_shards(shard_id),
	modality            TEXT        NOT NULL CHECK (modality IN ('fingerprint','face','iris')),
	finger_position     SMALLINT    CHECK (finger_position BETWEEN 1 AND 10),
	quality_score       REAL        NOT NULL CHECK (quality_score BETWEEN 0.0 AND 1.0),
	-- Modality-specific opaque bytes; parser picks by modality column.
	fingerprint_minutiae BYTEA,     -- ISO/IEC 19794-2 CBEFF-wrapped
	face_embedding      BYTEA,      -- 512 * 4 = 2048 bytes float32 L2-normalised
	iris_iriscode       BYTEA,      -- 256 bytes (2048 bits)
	iris_mask           BYTEA,      -- 256 bytes mask bitmap
	kek_version         INT         NOT NULL,
	blob_ref            TEXT        NOT NULL,     -- MinIO URI to fully-encrypted source
	blob_sha256         BYTEA       NOT NULL,
	enrolled_at         TIMESTAMPTZ NOT NULL,
	enrolment_decision_id UUID      REFERENCES eel_enrolment_decisions(id),
	retired_at          TIMESTAMPTZ,
	retirement_reason   TEXT
);

CREATE INDEX IF NOT EXISTS idx_eel_gallery_shard_modality
	ON eel_gallery_templates (shard_id, modality)
	WHERE retired_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_gallery_subject
	ON eel_gallery_templates (subject_id)
	WHERE retired_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_gallery_shard_quality
	ON eel_gallery_templates (shard_id, quality_score DESC)
	WHERE retired_at IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_gallery_subject_modality_finger
	ON eel_gallery_templates (subject_id, modality, COALESCE(finger_position, 0))
	WHERE retired_at IS NULL;

-- HNSW index checkpoints per shard for crash recovery.
CREATE TABLE IF NOT EXISTS eel_hnsw_snapshots (
	snapshot_id         UUID        PRIMARY KEY,
	shard_id            SMALLINT    NOT NULL REFERENCES eel_shards(shard_id),
	modality            TEXT        NOT NULL,
	taken_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	size_bytes          BIGINT      NOT NULL,
	template_count      BIGINT      NOT NULL,
	minio_ref           TEXT        NOT NULL,
	sha256              BYTEA       NOT NULL,
	hnsw_params         JSONB       NOT NULL,   -- M, ef_construction, ef_search
	restored_at         TIMESTAMPTZ,
	restored_by         TEXT
);
CREATE INDEX IF NOT EXISTS idx_eel_hnsw_snapshots_shard_taken
	ON eel_hnsw_snapshots (shard_id, taken_at DESC);

-- Demographic Bloom cohorts for pre-match pruning.
CREATE TABLE IF NOT EXISTS eel_demographic_cohorts (
	cohort_id           UUID        PRIMARY KEY,
	county_code         TEXT        NOT NULL,
	sex                 CHAR(1)     NOT NULL CHECK (sex IN ('M','F','X')),
	age_bucket          TEXT        NOT NULL CHECK (age_bucket IN ('18_25','26_40','41_60','61_plus')),
	bloom_bits          BYTEA       NOT NULL,
	bloom_hash_count    SMALLINT    NOT NULL,
	member_count        BIGINT      NOT NULL DEFAULT 0,
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_demographic_cohort
	ON eel_demographic_cohorts (county_code, sex, age_bucket);

-- Per-match ranked candidate list; one row per candidate returned to the caller.
CREATE TABLE IF NOT EXISTS eel_match_candidates (
	candidate_id        UUID        PRIMARY KEY,
	result_id           TEXT        NOT NULL REFERENCES eel_match_results(result_id),
	rank                SMALLINT    NOT NULL CHECK (rank BETWEEN 1 AND 1000),
	subject_id          UUID        NOT NULL,
	shard_id            SMALLINT    NOT NULL REFERENCES eel_shards(shard_id),
	score_fused         REAL        NOT NULL,
	score_fingerprint   REAL,
	score_face          REAL,
	score_iris          REAL,
	confidence          REAL        NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_eel_match_candidates_result_rank
	ON eel_match_candidates (result_id, rank);
CREATE INDEX IF NOT EXISTS idx_eel_match_candidates_subject
	ON eel_match_candidates (subject_id);
