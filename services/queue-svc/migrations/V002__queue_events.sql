CREATE TABLE eel_queue_events (
    id          UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type  TEXT        NOT NULL,
    station_code TEXT       NOT NULL,
    payload     JSONB       NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_eel_queue_events_station ON eel_queue_events(station_code, occurred_at DESC);
CREATE INDEX idx_eel_queue_events_type    ON eel_queue_events(event_type, occurred_at DESC);
