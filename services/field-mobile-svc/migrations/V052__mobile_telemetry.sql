-- Copyright (c) 2026 Nyimbi Odero
--
-- Licensed under the Apache License, Version 2.0 (the "License");
-- you may not use this file except in compliance with the License.
-- You may obtain a copy of the License at
--
--     http://www.apache.org/licenses/LICENSE-2.0
--
-- Unless required by applicable law or agreed to in writing, software
-- distributed under the License is distributed on an "AS IS" BASIS,
-- WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
-- See the License for the specific language governing permissions and
-- limitations under the License.

-- V052 Mobile telemetry ingestion tables
-- Ref: docs/features/mobile-telemetry-and-observability.md sections 3,5,6,9
-- ADR-0016 data retention and append-only guarantees apply to signals, alerts, snapshots

-- ---------------------------------------------------------------------------
-- 1. eel_telemetry_devices
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_devices (
    device_id           UUID            NOT NULL DEFAULT gen_random_uuid(),
    kit_id              TEXT            NOT NULL,
    app_id              TEXT            NOT NULL,
    persona_role        TEXT            NOT NULL,
    spiffe_id           TEXT,
    hmac_key_hsm_ref    TEXT            NOT NULL,
    first_seen_at       TIMESTAMPTZ     NOT NULL DEFAULT now(),
    last_seen_at        TIMESTAMPTZ     NOT NULL DEFAULT now(),
    revoked_at          TIMESTAMPTZ,
    revocation_reason   TEXT,
    CONSTRAINT eel_telemetry_devices_pkey PRIMARY KEY (device_id),
    CONSTRAINT eel_telemetry_devices_kit_id_unique UNIQUE (kit_id),
    CONSTRAINT eel_telemetry_devices_kit_app_unique UNIQUE (kit_id, app_id)
);

COMMENT ON TABLE eel_telemetry_devices IS
    'ADR-0016: registered field mobile devices; revocation recorded in-place';

-- ---------------------------------------------------------------------------
-- 2. eel_telemetry_sessions
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_sessions (
    session_id      UUID            NOT NULL DEFAULT gen_random_uuid(),
    device_id       UUID            NOT NULL REFERENCES eel_telemetry_devices (device_id),
    actor_svid      TEXT            NOT NULL,
    entered_at      TIMESTAMPTZ     NOT NULL DEFAULT now(),
    exited_at       TIMESTAMPTZ,
    exit_reason     TEXT            CHECK (exit_reason IN ('normal', 'timeout', 'revoked', 'duress', 'crash')),
    signals_count   BIGINT          NOT NULL DEFAULT 0,
    CONSTRAINT eel_telemetry_sessions_pkey PRIMARY KEY (session_id)
);

CREATE INDEX eel_telemetry_sessions_device_entered
    ON eel_telemetry_sessions (device_id, entered_at DESC);

COMMENT ON TABLE eel_telemetry_sessions IS
    'ADR-0016: operator session windows on field devices; exit_reason constrained to 5 values';

-- ---------------------------------------------------------------------------
-- 3. eel_telemetry_signals (partitioned by occurred_at)
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_signals (
    signal_id           UUID            NOT NULL,
    device_id           UUID            NOT NULL REFERENCES eel_telemetry_devices (device_id),
    session_id          UUID            REFERENCES eel_telemetry_sessions (session_id),
    class               TEXT            NOT NULL
                            CHECK (class IN (
                                'location', 'time', 'activity', 'proximal_rfid', 'movement',
                                'login_logout', 'read', 'write', 'battery', 'lock_unlock',
                                'drop', 'network', 'attestation'
                            )),
    payload             JSONB           NOT NULL,
    occurred_at         TIMESTAMPTZ     NOT NULL,
    ingested_at         TIMESTAMPTZ     NOT NULL DEFAULT now(),
    monotonic_counter   BIGINT          NOT NULL,
    prev_hash           BYTEA,
    this_hash           BYTEA           NOT NULL,
    hmac_signature      BYTEA           NOT NULL,
    server_signature    BYTEA,
    trace_id            BYTEA,
    span_id             BYTEA,
    correlation_id      UUID,
    causation_id        UUID,
    CONSTRAINT eel_telemetry_signals_pkey PRIMARY KEY (signal_id, occurred_at)
) PARTITION BY RANGE (occurred_at);

-- Monthly partitions: 3 months ahead of migration date (2026-08 through 2026-10) plus default.
CREATE TABLE eel_telemetry_signals_2026_08
    PARTITION OF eel_telemetry_signals
    FOR VALUES FROM ('2026-08-01') TO ('2026-09-01');

CREATE TABLE eel_telemetry_signals_2026_09
    PARTITION OF eel_telemetry_signals
    FOR VALUES FROM ('2026-09-01') TO ('2026-10-01');

CREATE TABLE eel_telemetry_signals_2026_10
    PARTITION OF eel_telemetry_signals
    FOR VALUES FROM ('2026-10-01') TO ('2026-11-01');

CREATE TABLE eel_telemetry_signals_default
    PARTITION OF eel_telemetry_signals DEFAULT;

-- Indexes.
CREATE INDEX eel_telemetry_signals_device_occurred
    ON eel_telemetry_signals (device_id, occurred_at DESC);

CREATE INDEX eel_telemetry_signals_class_occurred
    ON eel_telemetry_signals (class, occurred_at DESC);

-- Unique per-device monotonic counter; the partition key must be included so
-- the uniqueness constraint is valid on the partitioned table.
CREATE UNIQUE INDEX eel_telemetry_signals_device_counter
    ON eel_telemetry_signals (device_id, monotonic_counter, occurred_at);

-- Partial index for high-priority signal classes.
CREATE INDEX eel_telemetry_signals_critical_class
    ON eel_telemetry_signals (class, occurred_at)
    WHERE class IN ('drop', 'lock_unlock', 'login_logout');

COMMENT ON TABLE eel_telemetry_signals IS
    'ADR-0016 data-class C1: append-only telemetry signal stream partitioned by month; '
    'each row carries a BLAKE3 prev_hash chain and HMAC-SHA256 device signature';

-- ---------------------------------------------------------------------------
-- 4. eel_telemetry_activity_frames
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_activity_frames (
    frame_id        UUID        NOT NULL DEFAULT gen_random_uuid(),
    device_id       UUID        NOT NULL REFERENCES eel_telemetry_devices (device_id),
    second_bucket   TIMESTAMPTZ NOT NULL,
    accel_x_avg     REAL        NOT NULL,
    accel_y_avg     REAL        NOT NULL,
    accel_z_avg     REAL        NOT NULL,
    accel_max_g     REAL        NOT NULL,
    gyro_x_avg      REAL        NOT NULL,
    gyro_y_avg      REAL        NOT NULL,
    gyro_z_avg      REAL        NOT NULL,
    motion_class    TEXT        NOT NULL
                        CHECK (motion_class IN ('stationary', 'walking', 'running', 'vehicle', 'unknown')),
    battery_pct     SMALLINT    NOT NULL,
    battery_temp_c  REAL        NOT NULL,
    network_class   TEXT,
    lat_coarse      REAL,
    lon_coarse      REAL,
    CONSTRAINT eel_telemetry_activity_frames_pkey PRIMARY KEY (frame_id)
);

CREATE INDEX eel_telemetry_activity_frames_device_bucket
    ON eel_telemetry_activity_frames (device_id, second_bucket DESC);

COMMENT ON TABLE eel_telemetry_activity_frames IS
    'ADR-0016: 1 Hz downsampled activity frames from 100 Hz raw movement signals';

-- ---------------------------------------------------------------------------
-- 5. eel_telemetry_geo_dwells
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_geo_dwells (
    dwell_id        UUID        NOT NULL DEFAULT gen_random_uuid(),
    device_id       UUID        NOT NULL REFERENCES eel_telemetry_devices (device_id),
    polygon_ref     TEXT        NOT NULL,
    entered_at      TIMESTAMPTZ NOT NULL,
    exited_at       TIMESTAMPTZ,
    dwell_seconds   INT,
    CONSTRAINT eel_telemetry_geo_dwells_pkey PRIMARY KEY (dwell_id)
);

CREATE INDEX eel_telemetry_geo_dwells_device_entered
    ON eel_telemetry_geo_dwells (device_id, entered_at DESC);

COMMENT ON TABLE eel_telemetry_geo_dwells IS
    'ADR-0016: nightly-computed geo-dwell periods from location signal stream';

-- ---------------------------------------------------------------------------
-- 6. eel_telemetry_alerts
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_alerts (
    alert_id            UUID        NOT NULL DEFAULT gen_random_uuid(),
    device_id           UUID        NOT NULL REFERENCES eel_telemetry_devices (device_id),
    class               TEXT        NOT NULL
                            CHECK (class IN (
                                'drop', 'geo_fence_breach', 'session_hijack',
                                'battery_low', 'rfid_absence', 'login_time_anomaly',
                                'read_heavy', 'coordinated_anomaly', 'chassis_tamper'
                            )),
    severity            TEXT        NOT NULL CHECK (severity IN ('info', 'warn', 'critical')),
    detected_at         TIMESTAMPTZ NOT NULL DEFAULT now(),
    triggering_signal   UUID,
    payload             JSONB       NOT NULL DEFAULT '{}',
    ecc_dispatched      BOOLEAN     NOT NULL DEFAULT FALSE,
    ecc_dispatched_at   TIMESTAMPTZ,
    resolved_at         TIMESTAMPTZ,
    resolution_note     TEXT,
    CONSTRAINT eel_telemetry_alerts_pkey PRIMARY KEY (alert_id)
);

CREATE INDEX eel_telemetry_alerts_unresolved
    ON eel_telemetry_alerts (device_id, detected_at DESC)
    WHERE resolved_at IS NULL;

COMMENT ON TABLE eel_telemetry_alerts IS
    'ADR-0016 data-class C1: append-only alert records; resolution recorded via resolved_at column';

-- ---------------------------------------------------------------------------
-- 7. eel_telemetry_queries (audit log of DSL queries)
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_queries (
    query_id                UUID        NOT NULL DEFAULT gen_random_uuid(),
    actor_svid              TEXT        NOT NULL,
    actor_role              TEXT,
    query_dsl               JSONB,
    query_text              TEXT,
    executed_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    elapsed_ms              INT,
    row_count               BIGINT,
    evidence_bundle_hash    BYTEA,
    evidence_bundle_ref     TEXT,
    correlation_id          UUID,
    CONSTRAINT eel_telemetry_queries_pkey PRIMARY KEY (query_id)
);

CREATE INDEX eel_telemetry_queries_actor_executed
    ON eel_telemetry_queries (actor_svid, executed_at DESC);

COMMENT ON TABLE eel_telemetry_queries IS
    'ADR-0016: immutable audit log of all DSL queries against the telemetry dataset';

-- ---------------------------------------------------------------------------
-- 8. eel_telemetry_snapshots (daily Merkle anchors)
-- ---------------------------------------------------------------------------

CREATE TABLE eel_telemetry_snapshots (
    snapshot_id         UUID        NOT NULL DEFAULT gen_random_uuid(),
    snapshot_date       DATE        NOT NULL,
    device_id           UUID        REFERENCES eel_telemetry_devices (device_id),
    device_merkle_root  BYTEA,
    snapshot_taken_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    tlog_leaf_index     BIGINT,
    tlog_root_hash      BYTEA,
    inclusion_proof     BYTEA,
    CONSTRAINT eel_telemetry_snapshots_pkey PRIMARY KEY (snapshot_id),
    CONSTRAINT eel_telemetry_snapshots_date_device_unique UNIQUE (snapshot_date, device_id)
);

COMMENT ON TABLE eel_telemetry_snapshots IS
    'ADR-0016: daily per-device Merkle roots anchored to P05 transparency log (section 6)';

-- ---------------------------------------------------------------------------
-- Append-only triggers for signals, alerts, snapshots
-- ---------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION eel_telemetry_append_only_guard()
    RETURNS trigger
    LANGUAGE plpgsql AS
$$
BEGIN
    RAISE EXCEPTION
        'table % is append-only; UPDATE and DELETE are not permitted (ADR-0016)',
        TG_TABLE_NAME;
END;
$$;

-- eel_telemetry_signals
CREATE TRIGGER eel_telemetry_signals_no_update
    BEFORE UPDATE ON eel_telemetry_signals
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();

CREATE TRIGGER eel_telemetry_signals_no_delete
    BEFORE DELETE ON eel_telemetry_signals
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();

-- eel_telemetry_alerts
CREATE TRIGGER eel_telemetry_alerts_no_update
    BEFORE UPDATE ON eel_telemetry_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();

CREATE TRIGGER eel_telemetry_alerts_no_delete
    BEFORE DELETE ON eel_telemetry_alerts
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();

-- eel_telemetry_snapshots
CREATE TRIGGER eel_telemetry_snapshots_no_update
    BEFORE UPDATE ON eel_telemetry_snapshots
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();

CREATE TRIGGER eel_telemetry_snapshots_no_delete
    BEFORE DELETE ON eel_telemetry_snapshots
    FOR EACH ROW EXECUTE FUNCTION eel_telemetry_append_only_guard();
