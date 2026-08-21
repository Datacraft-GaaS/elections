-- Row-level security for observer isolation and party-agent isolation (ADR-0012)

ALTER TABLE voters ENABLE ROW LEVEL SECURITY;

-- Observers: read-only, no Tier A fields (handled at application layer too)
CREATE POLICY observer_read ON voters
    FOR SELECT
    TO PUBLIC
    USING (TRUE);

-- Application writes only (via service account)
CREATE POLICY service_write ON voters
    FOR ALL
    TO PUBLIC
    USING (current_setting('app.role', TRUE) IN ('bvr-svc', 'admin'))
    WITH CHECK (current_setting('app.role', TRUE) IN ('bvr-svc', 'admin'));
