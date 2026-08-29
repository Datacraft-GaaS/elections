-- M20 custody API surface — postgres persistence for public contract (no in-memory)
-- Each AppState SharedMap -> PgPool + Store trait (TD-03-P0-01). Reversible.

CREATE TABLE IF NOT EXISTS custody_api_assets (
    asset_id                UUID PRIMARY KEY,
    asset_type              TEXT NOT NULL CHECK (asset_type IN ('kit','ballot','seal','consumable','vehicle','device')),
    serial_number           TEXT NOT NULL UNIQUE,
    current_custodian_id    TEXT NOT NULL,
    current_location        TEXT NOT NULL,
    status                  TEXT NOT NULL DEFAULT 'in_custody' CHECK (status IN ('in_custody','in_transit','in_warehouse','deployed','returned','destroyed')),
    condition               TEXT NOT NULL DEFAULT 'good',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS custody_api_transfers (
    transfer_id         UUID PRIMARY KEY,
    asset_id            UUID NOT NULL REFERENCES custody_api_assets(asset_id) ON DELETE CASCADE,
    from_custodian_id   TEXT NOT NULL,
    to_custodian_id     TEXT NOT NULL,
    from_location       TEXT NOT NULL,
    to_location         TEXT NOT NULL,
    reason              TEXT,
    transferred_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    officer_id          TEXT NOT NULL,
    witness_id          TEXT
);

CREATE TABLE IF NOT EXISTS custody_api_seals (
    seal_id         UUID PRIMARY KEY,
    asset_id        UUID NOT NULL REFERENCES custody_api_assets(asset_id) ON DELETE CASCADE,
    seal_number     TEXT NOT NULL UNIQUE,
    sealed_by_id    TEXT NOT NULL,
    sealed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    status          TEXT NOT NULL DEFAULT 'intact' CHECK (status IN ('intact','broken'))
);

CREATE TABLE IF NOT EXISTS custody_api_seal_breaks (
    break_id        UUID PRIMARY KEY,
    seal_id         UUID NOT NULL REFERENCES custody_api_seals(seal_id) ON DELETE CASCADE,
    broken_by_id    TEXT NOT NULL,
    countersign_id  TEXT,
    reason          TEXT NOT NULL,
    broken_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS custody_api_reserves (
    reserve_id      UUID PRIMARY KEY,
    asset_type      TEXT NOT NULL,
    count           INTEGER NOT NULL DEFAULT 0,
    location        TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS custody_api_reconciles (
    reconcile_id    UUID PRIMARY KEY,
    station_code    TEXT NOT NULL,
    election_id     TEXT NOT NULL,
    assets_expected INTEGER NOT NULL,
    assets_found    INTEGER NOT NULL,
    discrepancies   JSONB NOT NULL DEFAULT '[]'::jsonb,
    status          TEXT NOT NULL,
    officer_id      TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_custody_api_transfers_asset ON custody_api_transfers(asset_id);
CREATE INDEX IF NOT EXISTS idx_custody_api_seals_asset ON custody_api_seals(asset_id);
CREATE INDEX IF NOT EXISTS idx_custody_api_seal_breaks_seal ON custody_api_seal_breaks(seal_id);

-- Down (reversible): keep data, but allow rollback in dev
-- To rollback: DROP TABLE IF EXISTS custody_api_reconciles, custody_api_reserves, custody_api_seal_breaks, custody_api_seals, custody_api_transfers, custody_api_assets CASCADE;
