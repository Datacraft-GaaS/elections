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

-- M52: election calendar + gazette initial schema

CREATE TABLE eel_calendar_events (
    id                  UUID PRIMARY KEY,
    election_id         UUID NOT NULL,
    event_type          TEXT NOT NULL,
    scheduled_at        TIMESTAMPTZ NOT NULL,
    gazette_ref         TEXT,
    statutory_citation  TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_eel_calendar_events_election ON eel_calendar_events (election_id);
CREATE INDEX idx_eel_calendar_events_scheduled ON eel_calendar_events (scheduled_at);

CREATE TABLE eel_statutory_rules (
    id          UUID PRIMARY KEY,
    citation    TEXT NOT NULL UNIQUE,
    rule_type   TEXT NOT NULL,
    params      JSONB NOT NULL DEFAULT '{}',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE eel_gazette_notices (
    id                  UUID PRIMARY KEY,
    notice_type         TEXT NOT NULL,
    body                TEXT NOT NULL,
    statutory_rule_ids  UUID[] NOT NULL DEFAULT '{}',
    legal_approver_id   UUID,
    hsm_signature       TEXT,
    published_at        TIMESTAMPTZ,
    gov_printer_ref     TEXT UNIQUE,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE eel_subscriptions (
    id              UUID PRIMARY KEY,
    subscriber_id   UUID NOT NULL,
    channel         TEXT NOT NULL,
    event_types     TEXT[] NOT NULL DEFAULT '{}',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (subscriber_id, channel)
);
