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

-- Migration V3: kit_ota_manifests (WP-3.8)
CREATE TABLE kit_ota_manifests (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    kit_tier     TEXT NOT NULL CHECK (kit_tier IN ('bvr-kit', 'bvi-kit', 'legacy-android')),
    version      TEXT NOT NULL,
    download_url TEXT NOT NULL,
    manifest_sig TEXT NOT NULL,
    released_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    active       BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE UNIQUE INDEX kit_ota_manifests_tier_version
    ON kit_ota_manifests (kit_tier, version);

CREATE INDEX kit_ota_manifests_active
    ON kit_ota_manifests (kit_tier, released_at DESC)
    WHERE active = TRUE;

COMMENT ON TABLE kit_ota_manifests IS 'Signed OTA manifests for kit firmware updates (WP-3.8).';

-- Seed initial versions.
INSERT INTO kit_ota_manifests (kit_tier, version, download_url, manifest_sig)
VALUES
    ('bvr-kit',         '1.0.1', 'https://artifacts.eel.local/ota/bvr-kit/1.0.1.zip',         'stub-sig-bvr-1.0.1'),
    ('bvi-kit',         '1.0.1', 'https://artifacts.eel.local/ota/bvi-kit/1.0.1.zip',         'stub-sig-bvi-1.0.1'),
    ('legacy-android',  '1.0.0', 'https://artifacts.eel.local/ota/legacy-android/1.0.0.zip',  'stub-sig-leg-1.0.0');
