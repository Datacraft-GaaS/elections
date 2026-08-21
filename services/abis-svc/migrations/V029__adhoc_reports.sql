-- Ad-hoc report drafts + versions + saved queries + shares.
-- Users can author, edit, preview, publish, and share ad-hoc reports without
-- touching the seeded catalogue.

CREATE TABLE IF NOT EXISTS eel_adhoc_reports (
	adhoc_id            UUID        PRIMARY KEY,
	slug                TEXT        NOT NULL,          -- user-chosen; unique per author + active
	title               TEXT        NOT NULL,
	description         TEXT,
	author_id           UUID        NOT NULL,          -- eel_reviewers.reviewer_id or staff subject
	data_class          TEXT        NOT NULL CHECK (data_class IN ('restricted','sensitive','public'))
	                                DEFAULT 'sensitive',
	visibility          TEXT        NOT NULL DEFAULT 'private'
	                                CHECK (visibility IN ('private','shared','team','org')),
	published_as_definition_id UUID REFERENCES eel_report_definitions(definition_id),
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	archived_at         TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_adhoc_reports_author_slug_active
	ON eel_adhoc_reports (author_id, slug)
	WHERE archived_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_adhoc_reports_visibility
	ON eel_adhoc_reports (visibility, updated_at DESC)
	WHERE archived_at IS NULL;

-- Every save creates a new version (append-only history for audit).
CREATE TABLE IF NOT EXISTS eel_adhoc_report_versions (
	version_id          UUID        PRIMARY KEY,
	adhoc_id            UUID        NOT NULL REFERENCES eel_adhoc_reports(adhoc_id),
	version_number      INT         NOT NULL,
	source_type         TEXT        NOT NULL CHECK (source_type IN ('sql','query_builder','markdown')),
	source              TEXT        NOT NULL,          -- raw SQL / builder JSON / MD template
	parameters          JSONB       NOT NULL DEFAULT '[]',
	                                -- [{name, type, default, required, prompt}]
	allowed_tables      TEXT[]      NOT NULL DEFAULT '{}',  -- allow-list guard
	column_projection   JSONB,                             -- optional friendly names + types
	render_pipeline     TEXT[]      NOT NULL DEFAULT ARRAY['pdf','docx','markdown','csv','json'],
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	created_by          UUID        NOT NULL,
	commit_message      TEXT,
	linted_at           TIMESTAMPTZ,
	lint_report         JSONB       NOT NULL DEFAULT '{}',
	safety_score        REAL        NOT NULL DEFAULT 0.0
	                                CHECK (safety_score BETWEEN 0.0 AND 1.0)
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_adhoc_versions_report_num
	ON eel_adhoc_report_versions (adhoc_id, version_number);
CREATE INDEX IF NOT EXISTS idx_eel_adhoc_versions_report
	ON eel_adhoc_report_versions (adhoc_id, created_at DESC);

-- Preview runs (bounded, sampled, unsigned). Distinct from full runs.
CREATE TABLE IF NOT EXISTS eel_adhoc_previews (
	preview_id          UUID        PRIMARY KEY,
	version_id          UUID        NOT NULL REFERENCES eel_adhoc_report_versions(version_id),
	parameter_values    JSONB       NOT NULL DEFAULT '{}',
	sample_limit        INT         NOT NULL DEFAULT 100 CHECK (sample_limit <= 1000),
	rows_returned       INT,
	elapsed_ms          INT,
	preview_format      TEXT        NOT NULL DEFAULT 'json',
	preview_bytes       BYTEA,                          -- capped at 1 MB
	requested_by        UUID        NOT NULL,
	requested_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	expires_at          TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '2 hours'),
	error_message       TEXT
);
CREATE INDEX IF NOT EXISTS idx_eel_adhoc_previews_expires
	ON eel_adhoc_previews (expires_at)
	WHERE preview_bytes IS NOT NULL;

-- Shares (grant read/edit to another subject).
CREATE TABLE IF NOT EXISTS eel_adhoc_shares (
	share_id            UUID        PRIMARY KEY,
	adhoc_id            UUID        NOT NULL REFERENCES eel_adhoc_reports(adhoc_id),
	shared_with         UUID        NOT NULL,
	permission          TEXT        NOT NULL CHECK (permission IN ('view','run','edit','admin')),
	granted_by          UUID        NOT NULL,
	granted_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	revoked_at          TIMESTAMPTZ,
	expires_at          TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_adhoc_share_active
	ON eel_adhoc_shares (adhoc_id, shared_with)
	WHERE revoked_at IS NULL;

-- Saved / reusable query fragments (per author).
CREATE TABLE IF NOT EXISTS eel_adhoc_snippets (
	snippet_id          UUID        PRIMARY KEY,
	author_id           UUID        NOT NULL,
	name                TEXT        NOT NULL,
	body                TEXT        NOT NULL,
	kind                TEXT        NOT NULL CHECK (kind IN ('cte','where','select','markdown_partial')),
	visibility          TEXT        NOT NULL DEFAULT 'private'
	                                CHECK (visibility IN ('private','shared','org')),
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_adhoc_snippets_author_name
	ON eel_adhoc_snippets (author_id, name);

-- Extend the format allow-list on the main runs / artifacts tables.
-- (V027 already includes docx? no — add docx + markdown formats.)
ALTER TABLE eel_report_artifacts
	DROP CONSTRAINT IF EXISTS eel_report_artifacts_format_check;
ALTER TABLE eel_report_artifacts
	ADD CONSTRAINT eel_report_artifacts_format_check
	CHECK (format IN ('json','csv','parquet','pdf','html','png','xlsx','svg','docx','markdown'));

-- View: joined convenience for the ad-hoc editor listing.
CREATE OR REPLACE VIEW eel_adhoc_reports_v AS
	SELECT
		r.adhoc_id, r.slug, r.title, r.author_id, r.visibility, r.data_class,
		r.published_as_definition_id, r.updated_at, r.archived_at,
		(SELECT MAX(version_number) FROM eel_adhoc_report_versions v WHERE v.adhoc_id = r.adhoc_id) AS latest_version_number,
		(SELECT COUNT(*) FROM eel_adhoc_shares s WHERE s.adhoc_id = r.adhoc_id AND s.revoked_at IS NULL) AS share_count
	FROM eel_adhoc_reports r;
