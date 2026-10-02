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

-- portal-svc migration V1: cache_control_hints (Refinery managed)
-- Stores per-snapshot cache expiry hints used by portal-api and edge caches.
-- Petition-window rules: T+0..T+7 aggressive, T+8..T+30 relaxed, T+31+ near-static.
-- No PII in this table.

CREATE TABLE IF NOT EXISTS cache_control_hints (
    snapshot_id         UUID        NOT NULL,
    merkle_root         TEXT        NOT NULL,
    published_at        TIMESTAMPTZ NOT NULL,
    max_age_seconds     INTEGER     NOT NULL DEFAULT 30,
    s_maxage_seconds    INTEGER     NOT NULL DEFAULT 30,
    stale_while_revalidate INTEGER  NOT NULL DEFAULT 10,
    petition_day        INTEGER,          -- days since national declaration (NULL if pre-declaration)
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id)
);

CREATE INDEX IF NOT EXISTS idx_cch_published_at ON cache_control_hints (published_at DESC);

COMMENT ON TABLE cache_control_hints IS
  'Per-snapshot cache expiry hints for the edge and portal-api (WP-3.9, FR-M11-024).';
COMMENT ON COLUMN cache_control_hints.petition_day IS
  'Null before declaration; 0 on declaration day; drives TTL rules per ADR-0010.';
