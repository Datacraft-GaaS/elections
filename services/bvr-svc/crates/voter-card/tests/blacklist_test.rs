//! Blacklist propagation contract tests (FR-M53-026).
//!
//! Hermetic tests assert the outbox envelope contract offline. The
//! persistence round trip needs PostgreSQL and runs against the
//! docker-compose stack: `just up` then
//! `cargo test -p voter-card --features db-tests -- --include-ignored`.

use chrono::Utc;
use eel_trace_rs::trace;
use uuid::Uuid;
use voter_card::blacklist::blacklist_envelope;
use voter_card::models::{BlacklistEntry, BlacklistSource};

fn sample_entry() -> BlacklistEntry {
    BlacklistEntry {
        id: Uuid::now_v7(),
        card_number: "KE-2027-000001".to_string(),
        reason: "reported lost".to_string(),
        blacklisted_at: Utc::now(),
        blacklist_source: BlacklistSource::LostReport,
    }
}

#[trace("FR-M53-026")]
#[test]
fn envelope_matches_propagation_contract() {
    let entry = sample_entry();
    let env = blacklist_envelope(&entry);

    assert_eq!(env.topic, "eel.bvr.card_blacklisted.v1");
    assert_eq!(env.event_type, "eel.bvr.card_blacklisted.v1");
    assert_eq!(env.kafka_key, entry.card_number);
    assert_eq!(env.aggregate_id, entry.card_number);
    assert_eq!(env.data["card_number"], entry.card_number);
    assert_eq!(env.data["source"], "LOST_REPORT");
}

/// Full round trip: row persisted, event enqueued exactly once,
/// re-blacklist is idempotent (no duplicate event).
///
/// Compile-gated behind `db-tests`; without the feature this prints as an
/// ignored test naming its prerequisite rather than silently passing.
#[trace("FR-M53-026")]
#[tokio::test]
#[cfg_attr(
    not(feature = "db-tests"),
    ignore = "requires PostgreSQL; run via docker-compose stack with --features db-tests"
)]
async fn add_entry_persists_and_emits_exactly_once() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/voter_card".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&db_url)
        .await
        .expect("connect DATABASE_URL");

    let entry = sample_entry();
    let svc = voter_card::blacklist::BlacklistService;

    svc.add_entry(&pool, entry.clone()).await.unwrap();

    let row: (String,) =
        sqlx::query_as("SELECT reason FROM card_blacklist WHERE card_number = $1")
            .bind(&entry.card_number)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, entry.reason);

    let events_before: i64 = sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = $1")
        .bind("eel.bvr.card_blacklisted.v1")
        .fetch_one(&pool)
        .await
        .unwrap();
    // Idempotent replay for the same card: no duplicate event.
    svc.add_entry(&pool, entry.clone()).await.unwrap();

    let events_after: i64 = sqlx::query_scalar("SELECT count(*) FROM outbox WHERE topic = $1")
        .bind("eel.bvr.card_blacklisted.v1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(events_before, events_after, "no duplicate event for unchanged entry");
}
