-- BVV objection workflow (M07).
CREATE TABLE IF NOT EXISTS objections (
  id                    UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  obj_id                UUID        NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  objector_session_hash TEXT        NOT NULL,
  target_voter_id       TEXT        NOT NULL,
  objection_type        TEXT        NOT NULL CHECK (objection_type IN ('rectification','transfer_request','removal_request')),
  justification         TEXT        NOT NULL,
  evidence_ref          TEXT,
  status                TEXT        NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','under_review','approved','rejected')),
  resolution            TEXT,
  filed_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
  resolved_at           TIMESTAMPTZ,
  resolved_by           TEXT
);

CREATE INDEX ON objections (status);
CREATE INDEX ON objections (target_voter_id);
CREATE INDEX ON objections (filed_at DESC);
