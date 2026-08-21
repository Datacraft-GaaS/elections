-- Audit shadow table (immutable history) per ADR-0013
-- Trigger-populated on every voter mutation.

CREATE TABLE IF NOT EXISTS audit_voters_history (
    history_id    TEXT        NOT NULL DEFAULT gen_random_uuid()::TEXT PRIMARY KEY,
    voter_id      TEXT        NOT NULL,
    county_code   CHAR(3)     NOT NULL,
    version_from  BIGINT      NOT NULL,
    version_to    BIGINT,
    operation     TEXT        NOT NULL, -- INSERT | UPDATE | DELETE
    changed_by    TEXT        NOT NULL,
    changed_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    row_snapshot  JSONB       NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_hist_voter_id ON audit_voters_history (voter_id);
CREATE INDEX IF NOT EXISTS idx_hist_changed  ON audit_voters_history (changed_at DESC);

-- Trigger to append to history on every voter change
CREATE OR REPLACE FUNCTION audit_voters_trigger_fn()
RETURNS TRIGGER LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO audit_voters_history (voter_id, county_code, version_from, operation, changed_by, row_snapshot)
    VALUES (
        COALESCE(OLD.voter_id, NEW.voter_id),
        COALESCE(OLD.county_code, NEW.county_code),
        COALESCE(OLD.version, 0),
        TG_OP,
        current_setting('app.current_user', TRUE),
        row_to_json(OLD)::JSONB
    );
    RETURN NEW;
END;
$$;

CREATE TRIGGER audit_voters_trigger
    AFTER INSERT OR UPDATE OR DELETE ON voters
    FOR EACH ROW EXECUTE FUNCTION audit_voters_trigger_fn();
