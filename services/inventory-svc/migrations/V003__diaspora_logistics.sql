-- M65: diaspora ballot physical logistics (WP-4.3)
-- Vienna Convention: M65 records only sealed handoff; no content inspection (FR-M65-057)

-- FR-M65-006: container UID format KE27-DIA-{POST:5s}-{SEQ:04d}
CREATE TABLE IF NOT EXISTS ballot_containers (
    id                      TEXT PRIMARY KEY,
    container_code          TEXT NOT NULL UNIQUE,
    station_id              TEXT NOT NULL,
    contents_manifest_url   TEXT,
    seal_id                 TEXT,
    status                  TEXT NOT NULL DEFAULT 'packed'
        CHECK (status IN ('packed','customs_cleared','in_transit','received','reconciled','quarantined')),
    packed_at               TIMESTAMPTZ
);

-- FR-M65-007: packing event signed by Embassy Election Officer and IAB-appointed observer
CREATE TABLE IF NOT EXISTS container_custody_events (
    id              TEXT PRIMARY KEY,
    container_id    TEXT NOT NULL REFERENCES ballot_containers(id),
    event_type      TEXT NOT NULL
        CHECK (event_type IN ('packed','customs_cleared','dispatched','received','seal_broken')),
    actor_id        TEXT NOT NULL,
    location        TEXT NOT NULL,
    photo_hash      TEXT,
    timestamp       TIMESTAMPTZ NOT NULL
);

-- FR-M65-016: KRA pre-clearance at least 6 hours before estimated arrival
CREATE TABLE IF NOT EXISTS preclearance_manifests (
    id              TEXT PRIMARY KEY,
    container_id    TEXT NOT NULL REFERENCES ballot_containers(id),
    mfa_ref         TEXT,
    kra_ref         TEXT,
    airline_awb     TEXT,
    status          TEXT NOT NULL DEFAULT 'submitted'
        CHECK (status IN ('submitted','approved','rejected','deferred')),
    submitted_at    TIMESTAMPTZ NOT NULL,
    expected_arrival_at TIMESTAMPTZ
);

-- FR-M65-011: routing policy per post (diplomatic pouch / airline cargo / charter)
CREATE TABLE IF NOT EXISTS hub_routes (
    id              TEXT PRIMARY KEY,
    post_code       TEXT NOT NULL,
    hub_code        TEXT NOT NULL,
    routing_policy  TEXT NOT NULL
        CHECK (routing_policy IN ('diplomatic_pouch','airline_cargo','charter_flight')),
    approved_at     TIMESTAMPTZ,
    locked          BOOLEAN NOT NULL DEFAULT false,
    approved_by     TEXT
);

-- FR-M65-031: reconcile received count against packing manifest
CREATE TABLE IF NOT EXISTS arrival_records (
    id              TEXT PRIMARY KEY,
    container_id    TEXT NOT NULL REFERENCES ballot_containers(id),
    expected_count  BIGINT NOT NULL,
    received_count  BIGINT NOT NULL,
    discrepancy     BIGINT NOT NULL,
    resolved_at     TIMESTAMPTZ
);

-- Audit shadow (FR-M65-049: container UIDs and seal numbers classified restricted)
CREATE TABLE IF NOT EXISTS ballot_containers_audit (
    audit_id        BIGSERIAL PRIMARY KEY,
    container_id    TEXT NOT NULL,
    actor_id        TEXT NOT NULL,
    action          TEXT NOT NULL,
    ts              TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outbox
CREATE TABLE IF NOT EXISTS diaspora_logistics_outbox (
    id              BIGSERIAL PRIMARY KEY,
    event_type      TEXT NOT NULL,
    aggregate_id    TEXT NOT NULL,
    payload         JSONB NOT NULL,
    published       BOOLEAN NOT NULL DEFAULT false,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_container_custody_container  ON container_custody_events(container_id);
CREATE INDEX IF NOT EXISTS idx_container_custody_ts         ON container_custody_events(timestamp);
CREATE INDEX IF NOT EXISTS idx_preclearance_container        ON preclearance_manifests(container_id);
CREATE INDEX IF NOT EXISTS idx_hub_routes_post               ON hub_routes(post_code);
CREATE INDEX IF NOT EXISTS idx_diaspora_outbox_pending       ON diaspora_logistics_outbox(id) WHERE NOT published;
