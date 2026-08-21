//! Batch ingest logic for CRS/NRB/KRA records.
//!
//! Write-behind pattern: accepted records are published to Kafka topic
//! `eel.bvr.voter_register_event.v1` before DB commit.

use crate::models::{BatchIngestionReport, BatchIngestRequest};
use chrono::Utc;
use uuid::Uuid;

pub struct IngestService;

impl IngestService {
    pub fn new() -> Self {
        IngestService
    }

    pub async fn ingest(&self, req: BatchIngestRequest) -> anyhow::Result<BatchIngestionReport> {
        // TODO: validate records against source schema, match voter IDs,
        // publish voter_register_event.v1 to Kafka, persist to DB.
        let count = req.records.len() as u64;
        Ok(BatchIngestionReport {
            id: Uuid::now_v7(),
            source: req.source,
            received_at: Utc::now(),
            record_count: count,
            matched: count,
            unmatched: 0,
            flagged: 0,
            actioned: 0,
            errors: 0,
            report_url: None,
        })
    }
}

impl Default for IngestService {
    fn default() -> Self {
        Self::new()
    }
}
