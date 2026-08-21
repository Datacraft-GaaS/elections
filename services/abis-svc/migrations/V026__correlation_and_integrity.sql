-- Backfill correlation-ID columns on hot tables and add missing FKs that
-- weren't safe to add at V001-V008 creation time (referenced tables landed
-- in V020-V024).

-- Correlation IDs on match_requests + enrolment_jobs + adjudication_cases.
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.columns
		WHERE table_name = 'eel_match_requests' AND column_name = 'trace_id'
	) THEN
		ALTER TABLE eel_match_requests
			ADD COLUMN trace_id       BYTEA,                      -- 16-byte W3C trace-id
			ADD COLUMN span_id        BYTEA,                      -- 8-byte W3C span-id
			ADD COLUMN correlation_id UUID,
			ADD COLUMN causation_id   UUID;
		CREATE INDEX idx_eel_match_requests_correlation
			ON eel_match_requests (correlation_id);
	END IF;

	IF NOT EXISTS (
		SELECT 1 FROM information_schema.columns
		WHERE table_name = 'eel_enrolment_jobs' AND column_name = 'trace_id'
	) THEN
		ALTER TABLE eel_enrolment_jobs
			ADD COLUMN trace_id       BYTEA,
			ADD COLUMN span_id        BYTEA,
			ADD COLUMN correlation_id UUID,
			ADD COLUMN causation_id   UUID;
		CREATE INDEX idx_eel_enrolment_jobs_correlation
			ON eel_enrolment_jobs (correlation_id);
	END IF;

	IF NOT EXISTS (
		SELECT 1 FROM information_schema.columns
		WHERE table_name = 'eel_adjudication_cases' AND column_name = 'trace_id'
	) THEN
		ALTER TABLE eel_adjudication_cases
			ADD COLUMN trace_id       BYTEA,
			ADD COLUMN span_id        BYTEA,
			ADD COLUMN correlation_id UUID,
			ADD COLUMN causation_id   UUID;
		CREATE INDEX idx_eel_adjudication_correlation
			ON eel_adjudication_cases (correlation_id);
	END IF;
END $$;

-- Wire enrolment_jobs.captured_by_kit_id -> eel_kits(kit_id) now that the
-- kits table exists (V021).
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.table_constraints
		WHERE constraint_name = 'fk_eel_enrolment_jobs_kit'
	) THEN
		ALTER TABLE eel_enrolment_jobs
			ADD CONSTRAINT fk_eel_enrolment_jobs_kit
			FOREIGN KEY (captured_by_kit_id) REFERENCES eel_kits(kit_id)
			ON DELETE SET NULL
			DEFERRABLE INITIALLY DEFERRED;
	END IF;
END $$;

-- Wire template_index.enclave_attest_id -> eel_enclave_attestations(enclave_id).
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.table_constraints
		WHERE constraint_name = 'fk_eel_template_index_attest'
	) THEN
		ALTER TABLE eel_template_index
			ADD CONSTRAINT fk_eel_template_index_attest
			FOREIGN KEY (enclave_attest_id) REFERENCES eel_enclave_attestations(enclave_id)
			ON DELETE SET NULL
			DEFERRABLE INITIALLY DEFERRED;
	END IF;
END $$;

-- Wire match_requests.probe_ref -> eel_blob_manifest(blob_ref) so probe
-- refs cannot dangle. Deferred so an outbox batch that inserts both rows
-- in the same tx passes.
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.table_constraints
		WHERE constraint_name = 'fk_eel_match_requests_probe_blob'
	) THEN
		ALTER TABLE eel_match_requests
			ADD CONSTRAINT fk_eel_match_requests_probe_blob
			FOREIGN KEY (probe_ref) REFERENCES eel_blob_manifest(blob_ref)
			DEFERRABLE INITIALLY DEFERRED;
	END IF;
END $$;

-- Add updated_at to eel_template_index for trigger + auditability.
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.columns
		WHERE table_name = 'eel_template_index' AND column_name = 'updated_at'
	) THEN
		ALTER TABLE eel_template_index
			ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();
		CREATE TRIGGER trg_touch_updated_at BEFORE UPDATE ON eel_template_index
			FOR EACH ROW EXECUTE FUNCTION eel_touch_updated_at();
	END IF;
END $$;

-- Ensure adjudication_cases has a reviewer FK once eel_reviewers exists.
DO $$
BEGIN
	IF NOT EXISTS (
		SELECT 1 FROM information_schema.table_constraints
		WHERE constraint_name = 'fk_eel_adjudication_investigator'
	) THEN
		ALTER TABLE eel_adjudication_cases
			ADD CONSTRAINT fk_eel_adjudication_investigator
			FOREIGN KEY (investigator_id) REFERENCES eel_reviewers(reviewer_id)
			ON DELETE SET NULL
			DEFERRABLE INITIALLY DEFERRED;
	END IF;
END $$;
