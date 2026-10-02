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

-- Declaration event-source tables (WP-3.3, M51).
-- Append-only; no DELETE or UPDATE paths in application code.

CREATE TABLE IF NOT EXISTS declarations (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    contest_id       TEXT        NOT NULL,
    election_id      TEXT        NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'pending'
                                 CHECK (status IN (
                                     'pending','declared','runoff_required',
                                     'tie_pending','amended','annulled')),
    winner_id        TEXT,
    declared_at      TIMESTAMPTZ,
    declared_by      TEXT,
    ceremony_id      TEXT,
    hsm_signature    TEXT,
    tally_snapshot_id TEXT,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS certificates_of_election (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    declaration_id   UUID        NOT NULL REFERENCES declarations(id),
    winner_id        TEXT        NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'draft'
                                 CHECK (status IN ('draft','issued','superseded','revoked')),
    issued_at        TIMESTAMPTZ,
    hsm_signature    TEXT,
    pdf_minio_ref    TEXT,
    tlog_leaf_index  BIGINT,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS petition_windows (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    declaration_id   UUID        NOT NULL REFERENCES declarations(id),
    status           TEXT        NOT NULL DEFAULT 'unopened'
                                 CHECK (status IN ('unopened','open','closed','petitioned')),
    -- T+28d 23:59:59 EAT inclusive (Art. 87 / s.75); stored as UTC (EC-TIM-04)
    window_open_at   TIMESTAMPTZ,
    window_close_at  TIMESTAMPTZ,
    petition_court_case TEXT,
    stream_version   BIGINT      NOT NULL DEFAULT 0,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS judiciary_acks (
    id               UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    declaration_id   UUID        NOT NULL REFERENCES declarations(id),
    registrar_id     TEXT        NOT NULL,
    acknowledged_at  TIMESTAMPTZ NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Outbox shared with the nested workspace (transactional exactly-once delivery).
CREATE TABLE IF NOT EXISTS declaration_outbox (
    id               BIGSERIAL   PRIMARY KEY,
    aggregate_type   TEXT        NOT NULL,
    aggregate_id     UUID        NOT NULL,
    event_type       TEXT        NOT NULL,
    payload          JSONB       NOT NULL,
    topic            TEXT        NOT NULL,
    published        BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS ix_declaration_outbox_unpublished
    ON declaration_outbox (published, created_at)
    WHERE published = FALSE;
