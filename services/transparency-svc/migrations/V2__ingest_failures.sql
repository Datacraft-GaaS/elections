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

-- Dead-letter table: events that failed to append to the transparency log.
-- The Kafka offset is NOT advanced past these rows until replayed; a non-empty
-- open set means the public log is missing results and must page an operator.

CREATE TABLE IF NOT EXISTS tlog_ingest_failures (
    id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    topic           TEXT        NOT NULL,
    partition       INT         NOT NULL,
    kafka_offset    BIGINT      NOT NULL,
    event_id        TEXT,
    error_kind      TEXT        NOT NULL,
    error_detail    TEXT        NOT NULL DEFAULT '',
    failed_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    replayed_at     TIMESTAMPTZ,
    PRIMARY KEY (id),
    UNIQUE (topic, partition, kafka_offset)
);

CREATE INDEX IF NOT EXISTS tlog_ingest_failures_open_idx
    ON tlog_ingest_failures (failed_at) WHERE replayed_at IS NULL;

COMMENT ON TABLE tlog_ingest_failures IS
    'ADR-0017 integrity: failed appends park here and block offset advance; an open row means the public log is missing an event';
