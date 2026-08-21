-- Transactional outbox (ADR-0015). Debezium reads this table via logical replication.
CREATE TABLE IF NOT EXISTS outbox (
  id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
  event_id      UUID        NOT NULL UNIQUE DEFAULT gen_random_uuid(),
  topic         TEXT        NOT NULL,
  partition_key TEXT,
  payload       JSONB       NOT NULL,
  headers       JSONB       NOT NULL DEFAULT '{}',
  signature     TEXT,
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  published_at  TIMESTAMPTZ
);

CREATE INDEX ON outbox (created_at) WHERE published_at IS NULL;
