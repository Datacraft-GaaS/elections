-- Kit certificate registry.
CREATE TABLE IF NOT EXISTS kit_certs (
  id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  kit_serial      TEXT        NOT NULL UNIQUE,
  cert_pem        TEXT        NOT NULL,
  issued_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
  revoked         BOOLEAN     NOT NULL DEFAULT false,
  revoked_at      TIMESTAMPTZ,
  revoked_reason  TEXT,
  revoked_by      TEXT
);

CREATE INDEX ON kit_certs (kit_serial);
CREATE INDEX ON kit_certs (revoked) WHERE revoked = true;
