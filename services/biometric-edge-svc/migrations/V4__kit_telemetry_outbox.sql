-- Migration V4: outbox table for kit events (WP-3.8)
-- Follows eel-outbox-rs schema convention.
CREATE TABLE edge_outbox (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type   TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload      JSONB NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at TIMESTAMPTZ,
    attempts     SMALLINT NOT NULL DEFAULT 0
);

CREATE INDEX edge_outbox_unpublished
    ON edge_outbox (created_at)
    WHERE published_at IS NULL;

COMMENT ON TABLE edge_outbox IS
    'Transactional outbox for edge events: kit_heartbeat.v1, kit_distress.v1, kit_ota_delivered.v1, kit_bvr_enrolled.v1 (WP-3.8).';

-- Audit shadow table for immutable append-only audit log.
CREATE TABLE edge_audit (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type   TEXT NOT NULL,
    aggregate_id TEXT NOT NULL,
    payload      JSONB NOT NULL,
    actor_svid   TEXT,
    recorded_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

COMMENT ON TABLE edge_audit IS 'Immutable audit shadow for edge API events (WP-3.8).';
