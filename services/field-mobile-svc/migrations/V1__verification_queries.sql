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

-- Verification queries audit log (M13, WP-4.7)
-- Tier C: channel, query_type, result_status logged; no PII; token_sha256 = hashed only
CREATE TABLE IF NOT EXISTS verification_queries (
    query_id           TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    channel            TEXT        NOT NULL,               -- flutter|pwa|ussd|whatsapp|api
    actor_session_hash TEXT        NOT NULL,               -- hashed, no PII
    query_type         TEXT        NOT NULL,               -- token|results|proof|civic
    result_status      TEXT        NOT NULL,               -- verified|unverifiable|not_found|error
    token_sha256       TEXT,                               -- hashed ballot token only
    ip_prefix          TEXT,                               -- /24 for IPv4, /48 for IPv6 (EC-SEC-09)
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_vq_created_at ON verification_queries (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_vq_channel    ON verification_queries (channel);
