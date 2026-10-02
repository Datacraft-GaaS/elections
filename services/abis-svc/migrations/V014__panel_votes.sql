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

-- Panel votes for Tier3 cases requiring M-of-N approval.
-- One row per reviewer per case; unique constraint prevents double-voting.

CREATE TABLE IF NOT EXISTS eel_panel_votes (
    id          UUID        PRIMARY KEY,
    case_id     UUID        NOT NULL REFERENCES eel_adjudication_cases_v2(id),
    reviewer_id UUID        NOT NULL,
    vote        TEXT        NOT NULL CHECK (vote IN ('Approve','Reject','Abstain')),
    rationale   TEXT,
    voted_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    hsm_signature BYTEA     NOT NULL,
    public_key_hex TEXT     NOT NULL
);

-- Each reviewer can only vote once per case.
CREATE UNIQUE INDEX IF NOT EXISTS uq_eel_panel_votes_reviewer_case
    ON eel_panel_votes (case_id, reviewer_id);

CREATE INDEX IF NOT EXISTS idx_eel_panel_votes_case
    ON eel_panel_votes (case_id, voted_at);
