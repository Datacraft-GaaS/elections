-- M37: Voter Education CMS
-- migration: 0001_training_cms

CREATE TABLE training_cms_content_items (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title               TEXT NOT NULL,
    language            TEXT NOT NULL,                      -- BCP-47 language tag, e.g. sw, en, om
    format              TEXT NOT NULL,                      -- video, audio, text, infographic
    topic_category      TEXT NOT NULL,
    target_audience     TEXT NOT NULL,
    election_cycle_tag  TEXT NOT NULL,
    status              TEXT NOT NULL DEFAULT 'draft',      -- draft, in_review, approved, published, expired, archived
    expiry_date         TIMESTAMPTZ,
    created_by          UUID NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cms_items_status           ON training_cms_content_items (status);
CREATE INDEX idx_cms_items_language         ON training_cms_content_items (language);
CREATE INDEX idx_cms_items_election_cycle   ON training_cms_content_items (election_cycle_tag);

CREATE TABLE training_cms_content_versions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id     UUID NOT NULL REFERENCES training_cms_content_items (id),
    version_num SMALLINT NOT NULL,
    body        TEXT,
    asset_url   TEXT,
    created_by  UUID NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (item_id, version_num)
);

CREATE INDEX idx_cms_versions_item_id ON training_cms_content_versions (item_id);

CREATE TABLE training_cms_fact_check_reviews (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id     UUID NOT NULL REFERENCES training_cms_content_items (id),
    reviewer_id UUID NOT NULL,
    decision    TEXT NOT NULL,                              -- approved, rejected, requires_changes
    annotations JSONB NOT NULL DEFAULT '{}',
    reviewed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cms_fact_check_item_id ON training_cms_fact_check_reviews (item_id);

CREATE TABLE training_cms_distribution_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    item_id     UUID NOT NULL REFERENCES training_cms_content_items (id),
    channel     TEXT NOT NULL,                              -- M13_feed, sms, whatsapp, county_portal
    county_code TEXT,
    distributed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_cms_distribution_item_id    ON training_cms_distribution_events (item_id);
CREATE INDEX idx_cms_distribution_channel    ON training_cms_distribution_events (channel);
