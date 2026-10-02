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

//! Integration test: outbox round-trip with real Postgres.
//!
//! Skipped unless the `integration-tests` feature is enabled.
//! Run with:
//!   cargo nextest run --package eel-outbox-rs --features integration-tests

#[cfg(feature = "integration-tests")]
mod integration {
    use eel_outbox_rs::{Outbox, OutboxEnvelope};
    use serde_json::json;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    async fn make_pool(url: &str) -> sqlx::PgPool {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(url)
            .await
            .expect("connect to test postgres")
    }

    fn sample_envelope() -> OutboxEnvelope {
        OutboxEnvelope {
            id: "01950f8c-0000-7000-8000-000000000099".into(),
            event_type: "eel.rts.form34a_captured.v1".into(),
            source: "spiffe://eel.local/service/rts-backend".into(),
            subject: Some("form-34a/47-0001-001".into()),
            time: "2027-08-15T15:30:00.123Z".into(),
            eel_stream_id: "01950f8c-0000-7000-8000-000000000099".into(),
            eel_seq: 1,
            eel_hlc: 1723731000123,
            eel_prev_hash: None,
            eel_correlation_id: None,
            eel_causation_id: None,
            eel_producer_key_id: None,
            eel_signature: None,
            eel_election_id: Some("2027-general".into()),
            eel_data_class: Some("B".into()),
            data: json!({ "form_id": "47-0001-001", "station": "47-0001" }),
            topic: "eel.rts.form34a.v1".into(),
            kafka_key: "01950f8c-0000-7000-8000-000000000099".into(),
            aggregate_type: "form34a".into(),
            aggregate_id: "01950f8c-0000-7000-8000-000000000099".into(),
        }
    }

    #[tokio::test]
    async fn outbox_emit_inserts_row_and_can_be_read_back() {
        let container = Postgres::default().start().await.expect("start postgres");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

        let pool = make_pool(&url).await;
        let outbox = Outbox::new(pool.clone());
        outbox.migrate().await.expect("migrate");

        let envelope = sample_envelope();
        let mut tx = pool.begin().await.expect("begin");
        outbox.emit(&mut tx, &envelope).await.expect("emit");
        tx.commit().await.expect("commit");

        // Verify the row is present.
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM outbox")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(row.0, 1, "expected one outbox row after emit");

        // published_at must be NULL (Debezium marks it after publishing).
        let published: (Option<chrono::DateTime<chrono::Utc>>,) =
            sqlx::query_as("SELECT published_at FROM outbox LIMIT 1")
                .fetch_one(&pool)
                .await
                .expect("select published_at");
        assert!(published.0.is_none(), "published_at should be NULL before Debezium picks it up");
    }

    #[tokio::test]
    async fn rolled_back_tx_leaves_no_outbox_row() {
        let container = Postgres::default().start().await.expect("start postgres");
        let port = container.get_host_port_ipv4(5432).await.expect("get port");
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");

        let pool = make_pool(&url).await;
        let outbox = Outbox::new(pool.clone());
        outbox.migrate().await.expect("migrate");

        let envelope = sample_envelope();
        {
            let mut tx = pool.begin().await.expect("begin");
            outbox.emit(&mut tx, &envelope).await.expect("emit");
            // drop tx without commit => rollback
        }

        let row: (i64,) = sqlx::query_as("SELECT COUNT(*)::bigint FROM outbox")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(row.0, 0, "rollback must leave no outbox row");
    }
}
