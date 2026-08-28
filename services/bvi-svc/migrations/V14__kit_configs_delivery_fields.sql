-- Device-facing delivery fields on kit_configs (getStationConfig).
-- Values are operational defaults applied at config-bundle creation time by
-- the enrolment pipeline; devices read them verbatim.
ALTER TABLE kit_configs ADD COLUMN IF NOT EXISTS match_threshold DOUBLE PRECISION NOT NULL DEFAULT 0.95;
ALTER TABLE kit_configs ADD COLUMN IF NOT EXISTS allowed_modalities JSONB NOT NULL DEFAULT '["fingerprint","iris"]';
ALTER TABLE kit_configs ADD COLUMN IF NOT EXISTS timeout_s INT NOT NULL DEFAULT 30;
ALTER TABLE kit_configs ADD COLUMN IF NOT EXISTS fallback_allowed BOOLEAN NOT NULL DEFAULT false;

ALTER TABLE kit_configs DROP CONSTRAINT IF EXISTS kit_configs_timeout_bounds_ck;
ALTER TABLE kit_configs ADD CONSTRAINT kit_configs_timeout_bounds_ck CHECK (timeout_s BETWEEN 5 AND 120);
ALTER TABLE kit_configs DROP CONSTRAINT IF EXISTS kit_configs_threshold_bounds_ck;
ALTER TABLE kit_configs ADD CONSTRAINT kit_configs_threshold_bounds_ck CHECK (match_threshold >= 0.5 AND match_threshold <= 1.0);
