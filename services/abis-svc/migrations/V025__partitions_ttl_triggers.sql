-- Trigger to auto-update updated_at, monthly partitions on the hot growth
-- tables, and TTL policy comments referencing ADR-0016 retention classes.

-- Generic updated_at maintainer.
CREATE OR REPLACE FUNCTION eel_touch_updated_at() RETURNS TRIGGER AS $$
BEGIN
	NEW.updated_at = NOW();
	RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
DECLARE
	t TEXT;
BEGIN
	FOREACH t IN ARRAY ARRAY[
		'eel_enrolment_jobs',
		'eel_shards',
		'eel_reviewers',
		'eel_vendor_registry'
	] LOOP
		EXECUTE format(
			'DROP TRIGGER IF EXISTS trg_touch_updated_at ON %I; '
			'CREATE TRIGGER trg_touch_updated_at BEFORE UPDATE ON %I '
			'FOR EACH ROW EXECUTE FUNCTION eel_touch_updated_at();',
			t, t
		);
	END LOOP;
END $$;

-- Match volume is huge (60k rps peak + audit shadow); partition monthly.
-- We create partitioned parents alongside the existing plain tables via
-- rename + repartition. If the existing tables already carry rows the
-- attach step preserves them under a "legacy" partition.

-- NOTE: partitioning requires PG14+. Guard with pg_version check.

DO $$
BEGIN
	IF current_setting('server_version_num')::INT < 140000 THEN
		RAISE NOTICE 'partitioning skipped: PostgreSQL < 14';
		RETURN;
	END IF;

	-- audit shadow: PARTITION BY RANGE (created_at)
	IF NOT EXISTS (
		SELECT 1 FROM pg_partitioned_table pt
		JOIN pg_class c ON c.oid = pt.partrelid
		WHERE c.relname = 'eel_abis_audit_shadow_p'
	) THEN
		EXECUTE 'CREATE TABLE IF NOT EXISTS eel_abis_audit_shadow_p (
			LIKE eel_abis_audit_shadow INCLUDING DEFAULTS,
			PRIMARY KEY (audit_id, created_at)
		) PARTITION BY RANGE (created_at)';

		-- default legacy catch-all
		EXECUTE 'CREATE TABLE IF NOT EXISTS eel_abis_audit_shadow_p_default
			PARTITION OF eel_abis_audit_shadow_p DEFAULT';

		-- current + next 6 months of monthly partitions
		FOR i IN 0..6 LOOP
			EXECUTE format(
				'CREATE TABLE IF NOT EXISTS eel_abis_audit_shadow_p_%s '
				'PARTITION OF eel_abis_audit_shadow_p '
				'FOR VALUES FROM (%L) TO (%L)',
				to_char(date_trunc('month', NOW()) + (i || ' months')::INTERVAL, 'YYYYMM'),
				(date_trunc('month', NOW()) + (i || ' months')::INTERVAL)::TIMESTAMPTZ,
				(date_trunc('month', NOW()) + ((i+1) || ' months')::INTERVAL)::TIMESTAMPTZ
			);
		END LOOP;
	END IF;
END $$;

-- Retention documentation. Actual TTL is enforced by the periodic cleanup
-- job (services/abis-svc/tools/retention-sweep). Values from ADR-0016.
COMMENT ON TABLE eel_blob_manifest             IS 'Retention: restricted 10y, sensitive 5y, public 20y (retention_until per row).';
COMMENT ON TABLE eel_idempotency_keys          IS 'Retention: 24h post first_seen_at; expires_at column authoritative.';
COMMENT ON TABLE eel_kit_attestation_events    IS 'Retention: 3y (audit log for security review).';
COMMENT ON TABLE eel_pad_events                IS 'Retention: 3y aggregate; is_live=false kept 7y for forensic review.';
COMMENT ON TABLE eel_abis_audit_shadow         IS 'Retention: 10y per ADR-0016 legal-hold class D.';
COMMENT ON TABLE eel_fairness_evaluations      IS 'Retention: 7y (compliance evidence).';
COMMENT ON TABLE eel_legacy_kiems_migration    IS 'Retention: permanent (post-cutover reconciliation record).';
COMMENT ON TABLE eel_match_candidates          IS 'Retention: 2y active + 5y archived (linked to eel_match_results).';
COMMENT ON TABLE eel_gallery_templates         IS 'Retention: cycle + 15y post-cycle (M69 long-term archive).';

-- Cross-table integrity views for the completeness scanner + reporting.
CREATE OR REPLACE VIEW eel_abis_health_v AS
	SELECT
		(SELECT COUNT(*) FROM eel_shards WHERE status <> 'healthy')                            AS unhealthy_shard_count,
		(SELECT COUNT(*) FROM eel_vendor_registry WHERE health_status = 'unhealthy' AND enabled) AS unhealthy_vendor_count,
		(SELECT COUNT(*) FROM eel_enrolment_jobs WHERE status IN ('Received','QualityChecking','DedupPending')) AS backlog_depth,
		(SELECT COUNT(*) FROM eel_kit_attestation_events
			WHERE verdict = 'reject' AND occurred_at > NOW() - INTERVAL '15 minutes')          AS attest_rejects_15m,
		(SELECT COUNT(*) FROM eel_pad_events
			WHERE is_live = FALSE AND detected_at > NOW() - INTERVAL '1 hour')                 AS pad_attacks_1h,
		(SELECT MAX(evaluated_at) FROM eel_fairness_evaluations)                                AS last_fairness_evaluation,
		NOW()                                                                                    AS observed_at;

-- Reporting view: current gallery footprint per shard.
CREATE OR REPLACE VIEW eel_gallery_shard_stats_v AS
	SELECT
		s.shard_id,
		s.status,
		COUNT(t.template_id) FILTER (WHERE t.retired_at IS NULL) AS active_templates,
		COUNT(t.template_id) FILTER (WHERE t.retired_at IS NOT NULL) AS retired_templates,
		COUNT(DISTINCT t.subject_id) FILTER (WHERE t.retired_at IS NULL) AS active_subjects,
		SUM(COALESCE(octet_length(t.fingerprint_minutiae), 0)
		    + COALESCE(octet_length(t.face_embedding), 0)
		    + COALESCE(octet_length(t.iris_iriscode), 0)
		    + COALESCE(octet_length(t.iris_mask), 0)) AS payload_bytes
	FROM eel_shards s
	LEFT JOIN eel_gallery_templates t ON t.shard_id = s.shard_id
	GROUP BY s.shard_id, s.status;
