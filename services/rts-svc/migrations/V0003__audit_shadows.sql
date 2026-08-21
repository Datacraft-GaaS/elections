-- Audit shadow tables (trigger-populated) per ADR-0013
CREATE SCHEMA IF NOT EXISTS audit;

CREATE TABLE IF NOT EXISTS audit.form34a_view_history (
    LIKE form34a_view INCLUDING ALL,
    _op   TEXT NOT NULL,
    _at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    _by   TEXT
);

CREATE OR REPLACE FUNCTION audit.log_form34a_changes() RETURNS trigger AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        INSERT INTO audit.form34a_view_history SELECT OLD.*, 'D', now(), current_user;
    ELSE
        INSERT INTO audit.form34a_view_history SELECT NEW.*, TG_OP, now(), current_user;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER form34a_audit_trigger
AFTER INSERT OR UPDATE OR DELETE ON form34a_view
FOR EACH ROW EXECUTE FUNCTION audit.log_form34a_changes();
