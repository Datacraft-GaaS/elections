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

//! Batch ingest logic for CRS/NRB/KRA records.
//!
//! Write-behind pattern: accepted records are published to Kafka topic
//! `eel.bvr.voter_register_event.v1` before DB commit. The validation,
//! matching, and confidence-scoring core is pure and unit-tested here; the
//! Kafka publish and DB persist are wired at the service boundary.

use crate::models::{BatchIngestRequest, BatchIngestionReport, EventSource};
use chrono::Utc;
use uuid::Uuid;

/// A record that passed schema validation and is ready for matching.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedRecord {
    pub national_id: String,
    pub biometric_ref: Option<String>,
    pub name: String,
}

/// Outcome of matching a validated record against the voter register.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchOutcome {
    pub matched: bool,
    /// Deterministic rule-based confidence in [0.0, 1.0] (FR-M35-009).
    pub confidence: f32,
    /// True when confidence is below the officer-review threshold (FR-M35-010).
    pub flagged: bool,
}

/// Validate a raw record against the source schema (FR-M35-001/002/003).
///
/// Returns `Err(reason)` when the record is malformed or missing a required
/// field. A Kenyan national ID is 7-8 digits; CRS death records additionally
/// require a biometric reference for confident matching (FR-M35-008).
pub fn validate_record(
    record: &serde_json::Value,
    source: &EventSource,
) -> Result<ValidatedRecord, String> {
    let obj = record
        .as_object()
        .ok_or_else(|| "record is not a JSON object".to_string())?;

    let national_id = obj
        .get("national_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "missing national_id".to_string())?;
    if !(7..=8).contains(&national_id.len()) || !national_id.chars().all(|c| c.is_ascii_digit()) {
        return Err("national_id must be 7-8 digits".to_string());
    }

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "missing name".to_string())?
        .to_string();

    let biometric_ref = match source {
        EventSource::Crs => {
            let b = obj
                .get("biometric_ref")
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| "CRS death record requires biometric_ref".to_string())?;
            Some(b.to_string())
        }
        _ => obj
            .get("biometric_ref")
            .and_then(|v| v.as_str())
            .map(String::from),
    };

    Ok(ValidatedRecord {
        national_id: national_id.to_string(),
        biometric_ref,
        name,
    })
}

/// Match a validated record against the set of known national IDs and, for
/// CRS, the biometric reference (FR-M35-008).
///
/// Confidence is deterministic:
/// - national ID + biometric reference both match: 1.0
/// - national ID matches, biometric reference absent: 0.7
/// - national ID matches, biometric reference mismatched: 0.5
/// - no national ID match: 0.0
pub fn match_record(
    record: &ValidatedRecord,
    known_ids: &std::collections::HashSet<&str>,
    known_biometrics: &std::collections::HashSet<&str>,
) -> MatchOutcome {
    let id_matched = known_ids.contains(record.national_id.as_str());
    if !id_matched {
        return MatchOutcome {
            matched: false,
            confidence: 0.0,
            flagged: false,
        };
    }
    let (confidence, matched) = match &record.biometric_ref {
        Some(b) if known_biometrics.contains(b.as_str()) => (1.0, true),
        Some(_) => (0.5, true),
        None => (0.7, true),
    };
    MatchOutcome {
        matched,
        confidence,
        flagged: confidence < 0.8,
    }
}

pub struct IngestService;

impl IngestService {
    pub fn new() -> Self {
        IngestService
    }

    /// Process a batch. `known_ids` and `known_biometrics` are the voter
    /// register reference sets; when no register is wired they are empty and
    /// every record is reported unmatched (never falsely matched).
    pub fn process(
        &self,
        req: &BatchIngestRequest,
        known_ids: &std::collections::HashSet<String>,
        known_biometrics: &std::collections::HashSet<String>,
    ) -> BatchIngestionReport {
        let id_set: std::collections::HashSet<&str> =
            known_ids.iter().map(String::as_str).collect();
        let bio_set: std::collections::HashSet<&str> =
            known_biometrics.iter().map(String::as_str).collect();

        let mut matched = 0u64;
        let mut unmatched = 0u64;
        let mut flagged = 0u64;
        let mut errors = 0u64;

        for record in &req.records {
            match validate_record(record, &req.source) {
                Err(_) => errors += 1,
                Ok(valid) => {
                    let outcome = match_record(&valid, &id_set, &bio_set);
                    if outcome.matched {
                        matched += 1;
                    } else {
                        unmatched += 1;
                    }
                    if outcome.flagged {
                        flagged += 1;
                    }
                }
            }
        }

        BatchIngestionReport {
            id: Uuid::now_v7(),
            source: req.source.clone(),
            received_at: Utc::now(),
            record_count: req.records.len() as u64,
            matched,
            unmatched,
            flagged,
            actioned: matched,
            errors,
            report_url: None,
        }
    }

    pub async fn ingest(&self, req: BatchIngestRequest) -> anyhow::Result<BatchIngestionReport> {
        // No voter register is wired in this profile; an empty reference set
        // means no record is ever falsely matched. The Kafka publish and DB
        // persist happen at the service boundary once a register is present.
        Ok(self.process(&req, &Default::default(), &Default::default()))
    }
}

impl Default for IngestService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;
    use serde_json::json;

    fn crs_record(national_id: &str, bio: Option<&str>) -> serde_json::Value {
        let mut m = serde_json::Map::new();
        m.insert("national_id".into(), json!(national_id));
        m.insert("name".into(), json!("JANE DOE"));
        if let Some(b) = bio {
            m.insert("biometric_ref".into(), json!(b));
        }
        serde_json::Value::Object(m)
    }

    #[trace("FR-M35-001")]
    #[test]
    fn crs_record_requires_biometric_ref() {
        let rec = crs_record("12345678", None);
        assert!(validate_record(&rec, &EventSource::Crs).is_err());
        let rec2 = crs_record("12345678", Some("BIO-1"));
        assert!(validate_record(&rec2, &EventSource::Crs).is_ok());
    }

    #[trace("FR-M35-001")]
    #[test]
    fn national_id_must_be_7_to_8_digits() {
        for bad in ["123", "123456789", "12A45678"] {
            let rec = crs_record(bad, Some("BIO-1"));
            assert!(
                validate_record(&rec, &EventSource::Crs).is_err(),
                "accepted {bad}"
            );
        }
    }

    #[trace("FR-M35-008")]
    #[test]
    fn confident_match_requires_id_and_biometric() {
        let rec = ValidatedRecord {
            national_id: "12345678".into(),
            biometric_ref: Some("BIO-1".into()),
            name: "JANE DOE".into(),
        };
        let ids = std::collections::HashSet::from(["12345678"]);
        let bios = std::collections::HashSet::from(["BIO-1"]);
        let o = match_record(&rec, &ids, &bios);
        assert!(o.matched);
        assert_eq!(o.confidence, 1.0);
        assert!(!o.flagged);
    }

    #[trace("FR-M35-009")]
    #[test]
    fn id_only_match_is_lower_confidence_and_flagged() {
        let rec = ValidatedRecord {
            national_id: "12345678".into(),
            biometric_ref: None,
            name: "JANE DOE".into(),
        };
        let ids = std::collections::HashSet::from(["12345678"]);
        let o = match_record(&rec, &ids, &Default::default());
        assert!(o.matched);
        assert_eq!(o.confidence, 0.7);
        assert!(o.flagged);
    }

    #[trace("FR-M35-009")]
    #[test]
    fn no_id_match_is_unmatched_zero_confidence() {
        let rec = ValidatedRecord {
            national_id: "99999999".into(),
            biometric_ref: Some("BIO-1".into()),
            name: "JANE DOE".into(),
        };
        let o = match_record(&rec, &Default::default(), &Default::default());
        assert!(!o.matched);
        assert_eq!(o.confidence, 0.0);
    }

    #[trace("FR-M35-034")]
    #[test]
    fn process_counts_matched_unmatched_flagged_errors() {
        let req = BatchIngestRequest {
            source: EventSource::Crs,
            records: vec![
                crs_record("12345678", Some("BIO-1")), // matched, confident
                crs_record("87654321", Some("BIO-2")), // unmatched
                crs_record("12345678", None),          // error: missing biometric
            ],
        };
        let ids = std::collections::HashSet::from(["12345678".to_string()]);
        let bios = std::collections::HashSet::from(["BIO-1".to_string()]);
        let report = IngestService::new().process(&req, &ids, &bios);
        assert_eq!(report.record_count, 3);
        assert_eq!(report.matched, 1);
        assert_eq!(report.unmatched, 1);
        assert_eq!(report.errors, 1);
        assert_eq!(report.actioned, 1);
    }
}
