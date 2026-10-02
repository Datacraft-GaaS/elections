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

-- portal-svc migration V2: snapshot_index (Refinery managed)
-- Quick-lookup table for /api/v1/portal/snapshots/latest (ADR-0011 Tier 1).
-- Populated by the Kafka consumer projecting eel.rts.snapshot_published.v1.
-- No PII.

CREATE TABLE IF NOT EXISTS snapshot_index (
    snapshot_id         UUID        NOT NULL,
    scope               TEXT        NOT NULL,   -- national|county|constituency|ward|...
    scope_code          TEXT,                   -- the Commission scope code; NULL for national
    position_code       TEXT        NOT NULL,   -- presidential|governor|senator|...
    merkle_root         TEXT        NOT NULL,
    cosign_sig          TEXT        NOT NULL,   -- base64url-encoded cosign bundle
    tarball_url         TEXT        NOT NULL,   -- MinIO object path for bulk.tar.gz
    published_at        TIMESTAMPTZ NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (snapshot_id)
);

CREATE INDEX IF NOT EXISTS idx_si_scope_published ON snapshot_index (scope, scope_code, published_at DESC);
CREATE INDEX IF NOT EXISTS idx_si_latest ON snapshot_index (published_at DESC);

COMMENT ON TABLE snapshot_index IS
  'Append-only snapshot index for fast /snapshots/latest lookups (WP-3.9, M11).';
