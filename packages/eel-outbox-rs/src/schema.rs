// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Debezium outbox table DDL.

pub const CREATE_OUTBOX_TABLE: &str = r#"
CREATE TABLE IF NOT EXISTS outbox (
    id              UUID        PRIMARY KEY,
    aggregate_id    TEXT        NOT NULL,
    aggregate_type  TEXT        NOT NULL,
    topic           TEXT        NOT NULL,
    key             TEXT        NOT NULL,
    payload         JSONB       NOT NULL,
    headers         JSONB       NOT NULL DEFAULT '{}',
    sig             BYTEA,
    hlc             BIGINT      NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    published_at    TIMESTAMPTZ
);
"#;

pub const CREATE_OUTBOX_INDEX: &str = r#"
CREATE INDEX IF NOT EXISTS outbox_published_at_idx
    ON outbox (published_at)
    WHERE published_at IS NULL;
"#;
