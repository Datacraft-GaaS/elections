-- Operational excellence: threshold tuning + model canary + drift +
-- twin registry + duplicate clusters + rectification + bulk ops.

CREATE TABLE IF NOT EXISTS eel_threshold_profiles (
	profile_id        UUID        PRIMARY KEY,
	name              TEXT        NOT NULL,
	modality          TEXT        NOT NULL,
	fmr_target        REAL        NOT NULL,
	fnmr_target       REAL        NOT NULL,
	score_threshold   REAL        NOT NULL,
	age_band          TEXT,                          -- optional per-age tuning
	sex               CHAR(1),
	county_code       TEXT,
	tuned_from        DATE        NOT NULL,
	tuned_to          DATE        NOT NULL,
	sample_count      BIGINT      NOT NULL,
	confidence_iv_low REAL,
	confidence_iv_hi  REAL,
	active            BOOLEAN     NOT NULL DEFAULT FALSE,
	activated_at      TIMESTAMPTZ,
	retired_at        TIMESTAMPTZ,
	created_by        UUID        NOT NULL,
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_threshold_profile_active
	ON eel_threshold_profiles (modality, COALESCE(age_band,''), COALESCE(sex,'-'), COALESCE(county_code,'-'))
	WHERE active = TRUE;

CREATE TABLE IF NOT EXISTS eel_model_canaries (
	canary_id         UUID        PRIMARY KEY,
	from_model_card   TEXT        NOT NULL REFERENCES eel_model_cards(model_card_version),
	to_model_card     TEXT        NOT NULL REFERENCES eel_model_cards(model_card_version),
	traffic_pct       SMALLINT    NOT NULL CHECK (traffic_pct BETWEEN 0 AND 100),
	started_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	promoted_at       TIMESTAMPTZ,
	rolled_back_at    TIMESTAMPTZ,
	rollback_reason   TEXT,
	metrics_snapshot  JSONB       NOT NULL DEFAULT '{}'
);
CREATE INDEX IF NOT EXISTS idx_eel_model_canaries_active
	ON eel_model_canaries (started_at DESC)
	WHERE promoted_at IS NULL AND rolled_back_at IS NULL;

CREATE TABLE IF NOT EXISTS eel_duplicate_clusters (
	cluster_id        UUID        PRIMARY KEY,
	discovered_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	cluster_size      INT         NOT NULL CHECK (cluster_size >= 2),
	member_subject_ids UUID[]     NOT NULL,
	primary_subject_id UUID,
	confidence        REAL        NOT NULL,
	resolution        TEXT        CHECK (resolution IN ('pending','confirmed','split','merged','false_alarm')),
	resolved_at       TIMESTAMPTZ,
	resolved_by       UUID
);
CREATE INDEX IF NOT EXISTS idx_eel_dup_clusters_pending
	ON eel_duplicate_clusters (discovered_at DESC)
	WHERE resolution = 'pending' OR resolution IS NULL;

CREATE TABLE IF NOT EXISTS eel_twin_registry (
	twin_group_id     UUID        PRIMARY KEY,
	subject_ids       UUID[]      NOT NULL CHECK (cardinality(subject_ids) >= 2),
	twin_type         TEXT        NOT NULL CHECK (twin_type IN ('identical','fraternal','sibling','declared_similar')),
	verification_source TEXT      NOT NULL CHECK (verification_source IN ('civil_registry','declared','adjudicator','face_iris_correlation')),
	created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	verified_at       TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS eel_rectification_requests (
	request_id        UUID        PRIMARY KEY,
	subject_id        UUID        NOT NULL,
	request_type      TEXT        NOT NULL CHECK (request_type IN ('biometric_recapture','demographic_correction','template_removal','name_change','dob_correction','gender_correction')),
	reason            TEXT        NOT NULL,
	submitted_by      UUID        NOT NULL,
	submitted_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	evidence_refs     JSONB       NOT NULL DEFAULT '[]',
	status            TEXT        NOT NULL DEFAULT 'submitted'
	                              CHECK (status IN ('submitted','triaged','evidence_pending','approved','rejected','completed','withdrawn')),
	assigned_to       UUID,
	decided_at        TIMESTAMPTZ,
	decision_note     TEXT,
	sla_deadline_at   TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days'),
	completed_at      TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_eel_rectification_sla
	ON eel_rectification_requests (sla_deadline_at)
	WHERE status NOT IN ('completed','rejected','withdrawn');

CREATE TABLE IF NOT EXISTS eel_bulk_operations (
	op_id             UUID        PRIMARY KEY,
	op_type           TEXT        NOT NULL CHECK (op_type IN ('batch_enrol','batch_dedup','batch_delete','batch_export','batch_rewrap','batch_watchlist_import','batch_retire')),
	total_items       BIGINT      NOT NULL,
	processed         BIGINT      NOT NULL DEFAULT 0,
	succeeded         BIGINT      NOT NULL DEFAULT 0,
	failed            BIGINT      NOT NULL DEFAULT 0,
	skipped           BIGINT      NOT NULL DEFAULT 0,
	status            TEXT        NOT NULL DEFAULT 'queued'
	                              CHECK (status IN ('queued','running','paused','complete','failed','cancelled')),
	requested_by      UUID        NOT NULL,
	requested_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	started_at        TIMESTAMPTZ,
	completed_at      TIMESTAMPTZ,
	progress_report   JSONB       NOT NULL DEFAULT '{}',
	failure_manifest  TEXT                          -- MinIO ref to failure CSV
);

CREATE TABLE IF NOT EXISTS eel_explainability_records (
	record_id         UUID        PRIMARY KEY,
	match_result_id   TEXT        REFERENCES eel_match_results(result_id),
	explanation_type  TEXT        NOT NULL CHECK (explanation_type IN ('minutiae_overlay','embedding_saliency','shap','lime','attention_map','similarity_reasoning')),
	produced_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	blob_ref          TEXT        NOT NULL,
	sha256            BYTEA       NOT NULL,
	produced_for      TEXT        NOT NULL CHECK (produced_for IN ('adjudicator','court','audit','subject_access_request'))
);
CREATE INDEX IF NOT EXISTS idx_eel_explainability_result
	ON eel_explainability_records (match_result_id);
