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

-- Participation token lookup log (M13, WP-4.7)
CREATE TABLE IF NOT EXISTS participation_token_lookups (
    lookup_id           TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    token_sha256        TEXT        NOT NULL,
    inclusion_proof_ref TEXT,                              -- reference to tlog entry
    verified_at         TIMESTAMPTZ,
    status              TEXT        NOT NULL DEFAULT 'pending',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_ptl_token_sha256 ON participation_token_lookups (token_sha256);
