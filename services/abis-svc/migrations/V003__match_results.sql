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

-- Match results: outcome, score, modality scores.
-- Data class C (no voter PII, just scores and refs) per ADR-0016.
CREATE TABLE IF NOT EXISTS eel_match_results (
    result_id        TEXT        PRIMARY KEY,
    request_id       TEXT        NOT NULL REFERENCES eel_match_requests(request_id),
    outcome          TEXT        NOT NULL CHECK (outcome IN ('match','no_match','adjudicate','insufficient_quality')),
    confidence_score REAL        NOT NULL,
    modality_scores  JSONB,
    attestation_hash TEXT,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_eel_match_results_request_id ON eel_match_results(request_id);
CREATE INDEX IF NOT EXISTS idx_eel_match_results_outcome    ON eel_match_results(outcome);
