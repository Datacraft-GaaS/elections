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

-- Reports catalogue + generation runs + rendered artefacts + delivery.
-- Covers operational, statistical, regulatory, legal, public, financial,
-- and executive report classes for the ABIS service.

CREATE TABLE IF NOT EXISTS eel_report_definitions (
	definition_id       UUID        PRIMARY KEY,
	slug                TEXT        NOT NULL,          -- 'daily-enrolment', 'fairness-quarterly'
	title               TEXT        NOT NULL,
	report_class        TEXT        NOT NULL CHECK (report_class IN (
	                                    'operational','statistical','regulatory','legal',
	                                    'public','financial','executive','ad_hoc'
	                                )),
	audience            TEXT[]      NOT NULL,          -- {'commission_staff','iab','odpc','public','court'}
	data_class          TEXT        NOT NULL CHECK (data_class IN ('restricted','sensitive','public')),
	sql_template_ref    TEXT,                          -- filesystem path to SQL template
	renderer_pipeline   TEXT[]      NOT NULL,          -- {'json','csv','parquet','pdf','html','png'}
	requires_hsm_sign   BOOLEAN     NOT NULL DEFAULT FALSE,
	requires_tlog_anchor BOOLEAN    NOT NULL DEFAULT FALSE,
	default_schedule    TEXT,                          -- cron string; null = on demand
	timezone            TEXT        NOT NULL DEFAULT 'Africa/Nairobi',
	retention_years     SMALLINT    NOT NULL DEFAULT 7,
	template_version    TEXT        NOT NULL,
	cosign_signature    BYTEA,                         -- signature over the template
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	retired_at          TIMESTAMPTZ,
	retirement_reason   TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_definitions_slug_active
	ON eel_report_definitions (slug)
	WHERE retired_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_eel_report_definitions_class
	ON eel_report_definitions (report_class)
	WHERE retired_at IS NULL;

-- Generation run: one per (definition, window). Idempotent by
-- (definition_id, window_from, window_to, requested_by).
CREATE TABLE IF NOT EXISTS eel_report_runs (
	run_id              UUID        PRIMARY KEY,
	definition_id       UUID        NOT NULL REFERENCES eel_report_definitions(definition_id),
	slug                TEXT        NOT NULL,
	window_from         TIMESTAMPTZ NOT NULL,
	window_to           TIMESTAMPTZ NOT NULL,
	status              TEXT        NOT NULL DEFAULT 'queued'
	                                CHECK (status IN (
	                                    'queued','extracting','rendering','signing',
	                                    'anchoring','delivering','complete','failed','cancelled'
	                                )),
	requested_by        TEXT        NOT NULL,
	requested_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	started_at          TIMESTAMPTZ,
	completed_at        TIMESTAMPTZ,
	rows_extracted      BIGINT,
	extraction_ms       INT,
	render_ms           INT,
	total_ms            INT,
	failure_reason      TEXT,
	correlation_id      UUID,
	trace_id            BYTEA
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_runs_definition_window_requester
	ON eel_report_runs (definition_id, window_from, window_to, requested_by);
CREATE INDEX IF NOT EXISTS idx_eel_report_runs_status_requested
	ON eel_report_runs (status, requested_at)
	WHERE status IN ('queued','extracting','rendering','signing','anchoring','delivering');
CREATE INDEX IF NOT EXISTS idx_eel_report_runs_slug_time
	ON eel_report_runs (slug, requested_at DESC);

-- Rendered artefacts. One row per output format per run.
CREATE TABLE IF NOT EXISTS eel_report_artifacts (
	artifact_id         UUID        PRIMARY KEY,
	run_id              UUID        NOT NULL REFERENCES eel_report_runs(run_id),
	format              TEXT        NOT NULL CHECK (format IN (
	                                    'json','csv','parquet','pdf','html','png','xlsx','svg'
	                                )),
	blob_ref            TEXT        NOT NULL,
	sha256              BYTEA       NOT NULL,
	size_bytes          BIGINT      NOT NULL CHECK (size_bytes >= 0),
	page_count          INT,                           -- PDFs
	row_count           BIGINT,                        -- csv, parquet, xlsx
	hsm_signature       BYTEA,                         -- optional per-artefact HSM sig
	hsm_key_role        TEXT,
	rendered_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_eel_report_artifacts_run
	ON eel_report_artifacts (run_id);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_artifacts_run_format
	ON eel_report_artifacts (run_id, format);

-- Transparency-log anchor for signed reports. Anchoring is not always
-- required (see eel_report_definitions.requires_tlog_anchor).
CREATE TABLE IF NOT EXISTS eel_report_tlog_anchors (
	anchor_id           UUID        PRIMARY KEY,
	run_id              UUID        NOT NULL REFERENCES eel_report_runs(run_id),
	artifact_id         UUID        NOT NULL REFERENCES eel_report_artifacts(artifact_id),
	tlog_leaf_index     BIGINT      NOT NULL,
	tlog_root_hash      BYTEA       NOT NULL,
	inclusion_proof     BYTEA       NOT NULL,
	sth_size            BIGINT      NOT NULL,
	witness_cosigs      JSONB       NOT NULL DEFAULT '[]',
	anchored_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_tlog_anchors_leaf
	ON eel_report_tlog_anchors (tlog_leaf_index);

-- Subscribers (email/webhook/kafka/sftp/gazette).
CREATE TABLE IF NOT EXISTS eel_report_subscribers (
	subscriber_id       UUID        PRIMARY KEY,
	name                TEXT        NOT NULL,
	channel             TEXT        NOT NULL CHECK (channel IN (
	                                    'email','webhook','kafka','sftp','gov_printer','kbc_sftp','api_pull'
	                                )),
	endpoint            TEXT        NOT NULL,          -- URL / SFTP host / kafka topic / email address (hashed)
	endpoint_hash       BYTEA       NOT NULL,          -- SHA-256 of endpoint (PII protection for emails)
	audience            TEXT        NOT NULL,          -- 'commission_staff','iab','odpc','court','public'
	auth_ref            TEXT,                          -- OpenBao path to credentials
	pgp_pubkey_ref      TEXT,                          -- optional signed delivery
	preferred_format    TEXT        NOT NULL DEFAULT 'pdf',
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	verified_at         TIMESTAMPTZ,
	suspended_at        TIMESTAMPTZ,
	suspension_reason   TEXT
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_subscribers_endpoint_channel
	ON eel_report_subscribers (endpoint_hash, channel)
	WHERE suspended_at IS NULL;

-- Which subscribers get which reports.
CREATE TABLE IF NOT EXISTS eel_report_subscriptions (
	subscription_id     UUID        PRIMARY KEY,
	subscriber_id       UUID        NOT NULL REFERENCES eel_report_subscribers(subscriber_id),
	definition_id       UUID        NOT NULL REFERENCES eel_report_definitions(definition_id),
	format              TEXT        NOT NULL,
	filter              JSONB       NOT NULL DEFAULT '{}',    -- e.g. {county: "047"}
	created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	cancelled_at        TIMESTAMPTZ
);
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_report_subscription_active
	ON eel_report_subscriptions (subscriber_id, definition_id, format)
	WHERE cancelled_at IS NULL;

-- Delivery attempts, with retry + backoff.
CREATE TABLE IF NOT EXISTS eel_report_deliveries (
	delivery_id         UUID        PRIMARY KEY,
	run_id              UUID        NOT NULL REFERENCES eel_report_runs(run_id),
	subscription_id     UUID        NOT NULL REFERENCES eel_report_subscriptions(subscription_id),
	artifact_id         UUID        NOT NULL REFERENCES eel_report_artifacts(artifact_id),
	status              TEXT        NOT NULL DEFAULT 'pending'
	                                CHECK (status IN ('pending','sending','delivered','failed','abandoned')),
	attempt_count       SMALLINT    NOT NULL DEFAULT 0,
	max_attempts        SMALLINT    NOT NULL DEFAULT 5,
	next_attempt_at     TIMESTAMPTZ,
	last_attempt_at     TIMESTAMPTZ,
	delivered_at        TIMESTAMPTZ,
	last_error          TEXT,
	remote_receipt_id   TEXT,                          -- e.g. Gov Printer submission id
	remote_receipt_at   TIMESTAMPTZ
);
CREATE INDEX IF NOT EXISTS idx_eel_report_deliveries_status_next
	ON eel_report_deliveries (status, next_attempt_at)
	WHERE status IN ('pending','sending','failed');
CREATE INDEX IF NOT EXISTS idx_eel_report_deliveries_run
	ON eel_report_deliveries (run_id);

-- Access log for pull-based consumption of published reports.
CREATE TABLE IF NOT EXISTS eel_report_access_log (
	access_id           UUID        PRIMARY KEY,
	artifact_id         UUID        NOT NULL REFERENCES eel_report_artifacts(artifact_id),
	subject_id          TEXT,                          -- authenticated caller if any
	ip_prefix           TEXT,                          -- /24 v4 or /48 v6 (privacy)
	user_agent_hash     BYTEA,
	accessed_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
	byte_range          TEXT
);
CREATE INDEX IF NOT EXISTS idx_eel_report_access_time
	ON eel_report_access_log (accessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_eel_report_access_artifact
	ON eel_report_access_log (artifact_id);
