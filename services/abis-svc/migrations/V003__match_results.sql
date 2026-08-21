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
