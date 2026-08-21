-- Grievance intake relay (M13 -> M23, WP-4.7)
-- USSD/SMS/WA session state; PII fields redacted before storage
CREATE TABLE IF NOT EXISTS grievance_intake (
    grievance_id     TEXT        NOT NULL PRIMARY KEY,  -- UUID7
    channel          TEXT        NOT NULL,               -- ussd|sms|whatsapp|flutter|pwa
    description_hash TEXT        NOT NULL,               -- SHA-256 of description; no raw PII
    relay_status     TEXT        NOT NULL DEFAULT 'pending',  -- pending|relayed|failed
    relay_ack_ref    TEXT,                               -- M23 acknowledgement reference
    submitted_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    relayed_at       TIMESTAMPTZ,
    duplicate_of     TEXT                                -- FK to grievance_id if duplicate
);
CREATE INDEX IF NOT EXISTS idx_gi_channel      ON grievance_intake (channel);
CREATE INDEX IF NOT EXISTS idx_gi_relay_status ON grievance_intake (relay_status);
