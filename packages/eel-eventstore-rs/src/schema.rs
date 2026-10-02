// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// DDL for the event store tables. Run once per database / schema.
pub const CREATE_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS eel_events (
    stream_id       UUID        NOT NULL,
    stream_version  BIGINT      NOT NULL,
    id              UUID        NOT NULL,
    event_type      TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    prev_hash       TEXT,
    hlc             TEXT        NOT NULL,
    recorded_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT eel_events_pkey PRIMARY KEY (stream_id, stream_version)
);

CREATE TABLE IF NOT EXISTS eel_snapshots (
    stream_id       UUID        NOT NULL,
    stream_version  BIGINT      NOT NULL,
    state           JSONB       NOT NULL,
    taken_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT eel_snapshots_pkey PRIMARY KEY (stream_id)
);

CREATE INDEX IF NOT EXISTS eel_events_stream_version_idx
    ON eel_events (stream_id, stream_version);

CREATE INDEX IF NOT EXISTS eel_events_hlc_idx
    ON eel_events (stream_id, hlc);
"#;
