-- M17 Speech Integrity: media items, analysis findings, adjudications.
-- Shared idempotency ledger for ADR-0011 mutating operations (24h replay window).
-- All content-bearing columns are class D operational metadata.

CREATE TABLE IF NOT EXISTS ecc_request_fingerprints (
    fingerprint_id  UUID PRIMARY KEY,
    operation       TEXT        NOT NULL,
    idempotency_key TEXT        NOT NULL,
    request_sha256  CHAR(64)    NOT NULL,
    response_status INTEGER     NOT NULL,
    response_body   JSONB       NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (operation, idempotency_key)
);

CREATE INDEX IF NOT EXISTS idx_ecc_request_fingerprints_created
    ON ecc_request_fingerprints (created_at);

CREATE TABLE IF NOT EXISTS speech_media_items (
    item_id        UUID PRIMARY KEY,
    url            TEXT        NOT NULL CHECK (char_length(url) <= 2048),
    url_sha256     CHAR(64)    NOT NULL UNIQUE,
    media_type     TEXT        NOT NULL CHECK (media_type IN ('audio', 'video', 'text')),
    language_code  TEXT        NOT NULL,
    source_platform TEXT       NOT NULL CHECK (char_length(source_platform) <= 64),
    captured_at    TIMESTAMPTZ NOT NULL,
    status         TEXT        NOT NULL DEFAULT 'queued'
                               CHECK (status IN ('queued', 'processing', 'flagged', 'cleared')),
    submitted_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_speech_media_items_status ON speech_media_items (status);
CREATE INDEX IF NOT EXISTS idx_speech_media_items_submitted ON speech_media_items (submitted_at);

CREATE TABLE IF NOT EXISTS speech_analysis_findings (
    finding_id UUID PRIMARY KEY,
    item_id    UUID             NOT NULL REFERENCES speech_media_items (item_id),
    category   TEXT             NOT NULL CHECK (category IN ('hate_speech', 'incitement', 'misinformation')),
    confidence DOUBLE PRECISION NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    excerpt    TEXT             NULL CHECK (excerpt IS NULL OR char_length(excerpt) <= 1000),
    created_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_speech_findings_item ON speech_analysis_findings (item_id);
CREATE INDEX IF NOT EXISTS idx_speech_findings_category ON speech_analysis_findings (category);

CREATE TABLE IF NOT EXISTS speech_adjudications (
    adjudication_id UUID PRIMARY KEY,
    item_id         UUID        NOT NULL UNIQUE REFERENCES speech_media_items (item_id),
    verdict         TEXT        NOT NULL CHECK (verdict IN ('harmful', 'borderline', 'cleared')),
    grounds         TEXT        NOT NULL CHECK (char_length(grounds) BETWEEN 10 AND 4000),
    referral_agency TEXT        NOT NULL CHECK (referral_agency IN ('NPS', 'DPP', 'KNCHR', 'none')),
    analyst_notes   TEXT        NULL CHECK (analyst_notes IS NULL OR char_length(analyst_notes) <= 4000),
    decided_by      TEXT        NULL,
    decided_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_speech_adjudications_verdict ON speech_adjudications (verdict);
CREATE INDEX IF NOT EXISTS idx_speech_adjudications_agency ON speech_adjudications (referral_agency);
