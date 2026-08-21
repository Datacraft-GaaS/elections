//! Blacklist propagation for lost/stolen voter ID cards.
//!
//! Target: <=30 min propagation to all BVI edge nodes.
//! Topic: eel.bvr.card_blacklisted.v1

use crate::models::BlacklistEntry;

pub struct BlacklistService;

impl BlacklistService {
    pub fn new() -> Self {
        BlacklistService
    }

    pub async fn add_entry(&self, entry: BlacklistEntry) -> anyhow::Result<()> {
        // TODO: write to DB + publish to eel.bvr.card_blacklisted.v1
        // Use Kafka producer with acks=all to guarantee propagation.
        tracing::info!(card_number = %entry.card_number, "blacklisting card");
        Ok(())
    }
}

impl Default for BlacklistService {
    fn default() -> Self {
        Self::new()
    }
}
