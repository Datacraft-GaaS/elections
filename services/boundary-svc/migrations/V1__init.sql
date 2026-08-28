-- boundary-svc bootstrap migration (Refinery managed).
-- PostGIS extension must be enabled on the target database.

CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ---------------------------------------------------------------------------
-- Core tables
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS boundaries (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    kind           TEXT        NOT NULL CHECK (kind IN ('county','constituency','ward','polling_station')),
    code           TEXT        NOT NULL,
    name           TEXT        NOT NULL,
    geom           GEOMETRY(Geometry, 4326) NOT NULL,
    effective_from DATE        NOT NULL,
    effective_to   DATE,
    version        INT         NOT NULL DEFAULT 1,
    status         TEXT        NOT NULL DEFAULT 'active' CHECK (status IN ('active','superseded','withdrawn')),
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT boundaries_code_version_uq UNIQUE (code, version)
);

CREATE INDEX ON boundaries (kind);
CREATE INDEX ON boundaries (code);
CREATE INDEX ON boundaries USING GIST (geom);

CREATE TABLE IF NOT EXISTS boundary_changes (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    boundary_id   UUID        NOT NULL REFERENCES boundaries (id),
    change_type   TEXT        NOT NULL CHECK (change_type IN ('created','boundary_updated','name_changed','deactivated')),
    old_version   INT,
    new_version   INT         NOT NULL,
    gazette_ref   TEXT,
    submitted_by  TEXT        NOT NULL,
    decided_by    TEXT,
    decided_at    TIMESTAMPTZ,
    status        TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX ON boundary_changes (boundary_id);
CREATE INDEX ON boundary_changes (status);

CREATE TABLE IF NOT EXISTS boundary_submissions (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    submitter_class  TEXT        NOT NULL CHECK (submitter_class IN ('public','political-party','civil-society')),
    payload_ref      TEXT        NOT NULL,
    submitted_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    status           TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','under_review','approved','rejected'))
);

CREATE INDEX ON boundary_submissions (status);

CREATE TABLE IF NOT EXISTS fairness_metrics (
    id                   UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    boundary_kind        TEXT         NOT NULL CHECK (boundary_kind IN ('county','constituency','ward','polling_station')),
    metric_name          TEXT         NOT NULL CHECK (metric_name IN ('compactness','population-equality','VRA-analog')),
    value                DOUBLE PRECISION NOT NULL,
    computed_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    methodology_version  TEXT         NOT NULL
);

CREATE INDEX ON fairness_metrics (boundary_kind, metric_name);
CREATE INDEX ON fairness_metrics (computed_at DESC);

CREATE TABLE IF NOT EXISTS geofence_polygons (
    id             UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    station_code   TEXT        NOT NULL,
    polygon        GEOMETRY(Polygon, 4326) NOT NULL,
    signed_by      TEXT        NOT NULL,
    signature      BYTEA       NOT NULL,
    effective_from DATE        NOT NULL,
    CONSTRAINT geofence_polygons_station_effective_uq UNIQUE (station_code, effective_from),
    CONSTRAINT geofence_signature_nonempty CHECK (length(signature) > 0)
);

CREATE INDEX ON geofence_polygons (station_code);
CREATE INDEX ON geofence_polygons USING GIST (polygon);

CREATE TABLE IF NOT EXISTS shapefile_imports (
    id           UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    source       TEXT        NOT NULL CHECK (source IN ('NLC','KNBS')),
    version      TEXT        NOT NULL,
    file_ref     TEXT        NOT NULL,
    imported_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    imported_by  TEXT        NOT NULL,
    verified     BOOLEAN     NOT NULL DEFAULT false,
    CONSTRAINT shapefile_imports_file_ref_uq UNIQUE (file_ref)
);

-- ---------------------------------------------------------------------------
-- Outbox (consumed by eel-outbox-rs)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS boundary_outbox (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    aggregate_id  TEXT NOT NULL,
    event_type    TEXT NOT NULL,
    payload       JSONB NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at  TIMESTAMPTZ
);

CREATE INDEX ON boundary_outbox (published_at) WHERE published_at IS NULL;

-- ---------------------------------------------------------------------------
-- Audit shadow log (populated by trigger from eel-observability-rs)
-- ---------------------------------------------------------------------------

CREATE TABLE IF NOT EXISTS audit_log (
    id          BIGSERIAL    PRIMARY KEY,
    table_name  TEXT         NOT NULL,
    operation   TEXT         NOT NULL,
    row_id      TEXT,
    changed_by  TEXT,
    changed_at  TIMESTAMPTZ  NOT NULL DEFAULT now(),
    diff        JSONB
);
