-- Diaspora voting schema (WP-5.8, M24).
-- Requires PostGIS for custody_events GPS point storage.

CREATE EXTENSION IF NOT EXISTS postgis;

-- Embassy or consulate polling stations abroad.
CREATE TABLE embassy_stations (
    id              UUID PRIMARY KEY,
    country_iso     CHAR(2)     NOT NULL,
    city            TEXT        NOT NULL,
    timezone        TEXT        NOT NULL,
    address         TEXT        NOT NULL,
    station_code    TEXT        NOT NULL UNIQUE,
    kits_deployed   INT         NOT NULL DEFAULT 0,
    opened_at       TIMESTAMPTZ,
    closed_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Frozen snapshots of eligible diaspora voters at a station.
CREATE TABLE diaspora_rolls (
    id                  UUID PRIMARY KEY,
    embassy_station_id  UUID        NOT NULL REFERENCES embassy_stations(id),
    snapshot_taken_at   TIMESTAMPTZ NOT NULL,
    voter_count         BIGINT      NOT NULL DEFAULT 0,
    hash                TEXT        NOT NULL,
    frozen              BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Form 34A diaspora results per station.
CREATE TABLE forms_34a_diaspora (
    id                      UUID PRIMARY KEY,
    embassy_station_id      UUID        NOT NULL REFERENCES embassy_stations(id),
    form_id                 TEXT        NOT NULL UNIQUE,
    results                 JSONB       NOT NULL DEFAULT '{}',
    rejected                BIGINT      NOT NULL DEFAULT 0,
    disputed                BIGINT      NOT NULL DEFAULT 0,
    transmitted_at          TIMESTAMPTZ,
    hsm_signature           TEXT,
    transmission_batch_id   UUID,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Physical ballot pouches for air transport to Nairobi.
CREATE TABLE pouch_custodies (
    id                  UUID PRIMARY KEY,
    form_ids            UUID[]      NOT NULL DEFAULT '{}',
    sealed_at           TIMESTAMPTZ,
    seal_id             TEXT,
    received_at_nairobi TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Individual custody handover events in the pouch chain.
-- gps_location stores the WGS-84 point where the transfer happened.
CREATE TABLE custody_events (
    id              BIGSERIAL   PRIMARY KEY,
    pouch_id        UUID        NOT NULL REFERENCES pouch_custodies(id),
    transferred_at  TIMESTAMPTZ NOT NULL,
    from_person     TEXT        NOT NULL,
    to_person       TEXT        NOT NULL,
    location        TEXT        NOT NULL,
    gps_location    GEOMETRY(Point, 4326),
    signature       TEXT        NOT NULL,
    seq             INT         NOT NULL,
    CONSTRAINT unique_pouch_seq UNIQUE (pouch_id, seq)
);

CREATE INDEX idx_custody_events_pouch ON custody_events(pouch_id, seq);
CREATE INDEX idx_forms34a_station    ON forms_34a_diaspora(embassy_station_id);
CREATE INDEX idx_rolls_station        ON diaspora_rolls(embassy_station_id);
