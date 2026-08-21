CREATE TABLE eel_queue_snapshots (
    id                       UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    station_code             TEXT        NOT NULL,
    kit_throughput_per_hour  INT         NOT NULL DEFAULT 0,
    clerk_report_count       INT         NOT NULL DEFAULT 0,
    cv_headcount             INT         NOT NULL DEFAULT 0,
    qr_scan_count            INT         NOT NULL DEFAULT 0,
    estimated_wait_minutes   INT         NOT NULL DEFAULT 0,
    disenfranchisement_flag  BOOLEAN     NOT NULL DEFAULT FALSE,
    s14a_recommended         BOOLEAN     NOT NULL DEFAULT FALSE,
    fused_at                 TIMESTAMPTZ NOT NULL DEFAULT now(),
    signature                TEXT,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_eel_queue_snapshots_station_code ON eel_queue_snapshots(station_code);
CREATE INDEX idx_eel_queue_snapshots_fused_at     ON eel_queue_snapshots(fused_at DESC);
