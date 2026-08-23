//! Blacklist propagation for lost/stolen voter ID cards.
//!
//! Target: <=30 min propagation to all BVI edge nodes.
//! Topic: eel.bvr.card_blacklisted.v1
//!
//! `add_entry` performs one transactional write: the blacklist row and the
//! outbox envelope are committed together, so Kafka propagation via Debezium
//! is atomic with the business write (see `eel-outbox-rs`).

use chrono::Utc;
use eel_outbox_rs::{Outbox, OutboxEnvelope};
use serde_json::json;
use sqlx::PgPool;

use crate::models::BlacklistEntry;

pub struct BlacklistService;

/// SPIFFE identity of this service used as the event source.
const SOURCE_URI: &str = "spiffe://eel.local/service/bvr-backend";

impl BlacklistService {
    /// Persist the blacklist entry and enqueue the propagation event.
    ///
    /// Idempotent on `card_number`: re-blacklisting an already blacklisted
    /// card refreshes the reason/timestamp but emits no duplicate event,
    /// so edge nodes converge without double-processing.
    pub async fn add_entry(&self, pool: &PgPool, entry: BlacklistEntry) -> anyhow::Result<()> {
        let mut tx = pool.begin().await?;

        // Idempotency gate: an identical existing entry is a no-op so edge
        // nodes never receive duplicate events for the same card state.
        let existing: Option<(String,)> =
            sqlx::query_as("SELECT reason FROM card_blacklist WHERE card_number = $1")
                .bind(&entry.card_number)
                .fetch_optional(&mut *tx)
                .await?;
        if existing.as_ref().is_some_and(|(reason,)| reason == &entry.reason) {
            tx.commit().await?;
            tracing::debug!(card_number = %entry.card_number, "blacklist entry unchanged");
            return Ok(());
        }

        sqlx::query(
            "INSERT INTO card_blacklist (id, card_number, reason, blacklisted_at, source)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (card_number)
             DO UPDATE SET reason = EXCLUDED.reason,
                           blacklisted_at = EXCLUDED.blacklisted_at,
                           source = EXCLUDED.source",
        )
        .bind(entry.id)
        .bind(&entry.card_number)
        .bind(&entry.reason)
        .bind(entry.blacklisted_at)
        .bind(blacklist_source_str(&entry.blacklist_source))
        .execute(&mut *tx)
        .await?;

        let outbox = Outbox::new(pool.clone());
        let envelope = blacklist_envelope(&entry);
        outbox.emit(&mut tx, &envelope).await?;

        tx.commit().await?;

        tracing::info!(
            card_number = %entry.card_number,
            topic = %envelope.topic,
            "blacklisted card"
        );
        Ok(())
    }
}

/// Map the enum to its wire representation (matches the SCREAMING_SNAKE_CASE
/// serde renaming used by `BlacklistSource`).
fn blacklist_source_str(source: &crate::models::BlacklistSource) -> &'static str {
    use crate::models::BlacklistSource::*;
    match source {
        LostReport => "LOST_REPORT",
        StolenReport => "STOLEN_REPORT",
        Deactivation => "DEACTIVATION",
        SystemFlag => "SYSTEM_FLAG",
    }
}

/// Build the CloudEvents-style outbox envelope for a blacklist entry.
///
/// Public because the topic/payload contract is asserted by integration
/// tests and consumed by downstream BVI edge projections.
pub fn blacklist_envelope(entry: &BlacklistEntry) -> OutboxEnvelope {
    let now = Utc::now();
    OutboxEnvelope {
        id: entry.id.to_string(),
        event_type: "eel.bvr.card_blacklisted.v1".into(),
        source: SOURCE_URI.into(),
        subject: Some(format!("card/{}", entry.card_number)),
        time: now.to_rfc3339(),
        eel_stream_id: entry.id.to_string(),
        eel_seq: 1,
        eel_hlc: now.timestamp_millis(),
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: None,
        eel_election_id: None,
        eel_data_class: Some("B".into()),
        data: json!({
            "card_number": entry.card_number,
            "reason": entry.reason,
            "blacklisted_at": entry.blacklisted_at.to_rfc3339(),
            "source": blacklist_source_str(&entry.blacklist_source),
        }),
        topic: "eel.bvr.card_blacklisted.v1".into(),
        kafka_key: entry.card_number.clone(),
        aggregate_type: "card".into(),
        aggregate_id: entry.card_number.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BlacklistSource;

    fn sample_entry() -> BlacklistEntry {
        BlacklistEntry {
            id: uuid::Uuid::now_v7(),
            card_number: "VC-2027-000123".into(),
            reason: "reported stolen at station 47-0001".into(),
            blacklisted_at: Utc::now(),
            blacklist_source: BlacklistSource::StolenReport,
        }
    }

    #[test]
    fn envelope_carries_contract_topic_and_payload() {
        let entry = sample_entry();
        let env = blacklist_envelope(&entry);

        assert_eq!(env.topic, "eel.bvr.card_blacklisted.v1");
        assert_eq!(env.event_type, "eel.bvr.card_blacklisted.v1");
        assert_eq!(env.kafka_key, entry.card_number);
        assert_eq!(env.aggregate_type, "card");
        assert_eq!(env.aggregate_id, entry.card_number);
        assert_eq!(env.subject.as_deref(), Some("card/VC-2027-000123"));
        assert_eq!(env.eel_data_class.as_deref(), Some("B"));
        assert_eq!(env.data["card_number"], entry.card_number);
        assert_eq!(env.data["source"], "STOLEN_REPORT");
    }

    #[test]
    fn envelope_id_is_stable_per_entry() {
        let entry = sample_entry();
        let env = blacklist_envelope(&entry);
        assert_eq!(env.id, entry.id.to_string());
        assert_eq!(env.eel_stream_id, entry.id.to_string());
    }

    #[test]
    fn source_strings_match_serde_renaming() {
        use crate::models::BlacklistSource::*;
        assert_eq!(blacklist_source_str(&LostReport), "LOST_REPORT");
        assert_eq!(blacklist_source_str(&StolenReport), "STOLEN_REPORT");
        assert_eq!(blacklist_source_str(&Deactivation), "DEACTIVATION");
        assert_eq!(blacklist_source_str(&SystemFlag), "SYSTEM_FLAG");
    }
}
