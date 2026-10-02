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
