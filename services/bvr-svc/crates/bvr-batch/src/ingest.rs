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

use crate::error::BatchError;
use bvr_domain::voter::{BiometricTemplateRef, Voter};

use serde::{Deserialize, Serialize};

/// One row in an ingest batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRow {
    pub idempotency_key: String,
    pub national_id: String,
    pub full_name: String,
    pub dob: String,
    pub county_code: String,
    pub polling_station_id: String,
    pub election_scope: String,
    pub biometric_template_refs: Vec<BiometricTemplateRef>,
}

/// Per-row outcome for partial-batch failure report (EC-DAT-07).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowOutcome {
    pub row_index: usize,
    pub idempotency_key: String,
    pub success: bool,
    pub voter_id: Option<String>,
    pub error: Option<String>,
}

/// Result of a batch ingest call.
#[derive(Debug)]
pub struct BatchResult {
    pub total: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub outcomes: Vec<RowOutcome>,
}

/// Process a batch of ingest rows. Each row is validated independently;
/// partial-batch failures do not roll back accepted rows.
///
/// Target throughput: 1 000 rows/sec (ADR-0018).
pub async fn process_batch(rows: Vec<IngestRow>) -> BatchResult {
    let total = rows.len();
    let mut outcomes = Vec::with_capacity(total);
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    for (idx, row) in rows.into_iter().enumerate() {
        match validate_row(&row) {
            Ok(voter) => {
                outcomes.push(RowOutcome {
                    row_index: idx,
                    idempotency_key: row.idempotency_key,
                    success: true,
                    voter_id: Some(voter.voter_id.clone()),
                    error: None,
                });
                accepted += 1;
            }
            Err(e) => {
                outcomes.push(RowOutcome {
                    row_index: idx,
                    idempotency_key: row.idempotency_key,
                    success: false,
                    voter_id: None,
                    error: Some(e.to_string()),
                });
                rejected += 1;
            }
        }
    }

    BatchResult {
        total,
        accepted,
        rejected,
        outcomes,
    }
}

fn validate_row(row: &IngestRow) -> Result<Voter, BatchError> {
    if row.national_id.trim().is_empty() {
        return Err(BatchError::RowValidation("national_id required".into()));
    }
    if row.full_name.trim().is_empty() {
        return Err(BatchError::RowValidation("full_name required".into()));
    }
    if row.county_code.trim().is_empty() {
        return Err(BatchError::RowValidation("county_code required".into()));
    }
    let voter = Voter::new(
        row.national_id.clone(),
        row.full_name.clone(),
        row.dob.clone(),
        row.county_code.clone(),
        row.polling_station_id.clone(),
        row.election_scope.clone(),
    );
    Ok(voter)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_row(idx: usize, bad_id: bool) -> IngestRow {
        IngestRow {
            idempotency_key: format!("key-{idx}"),
            national_id: if bad_id {
                "".into()
            } else {
                format!("KE{idx:08}")
            },
            full_name: "Test Voter".into(),
            dob: "1990-01-01".into(),
            county_code: "047".into(),
            polling_station_id: "047-001-001".into(),
            election_scope: "2027-general".into(),
            biometric_template_refs: vec![],
        }
    }

    #[tokio::test]
    async fn partial_batch_reports_per_row() {
        let rows = vec![make_row(0, false), make_row(1, true), make_row(2, false)];
        let result = process_batch(rows).await;
        assert_eq!(result.total, 3);
        assert_eq!(result.accepted, 2);
        assert_eq!(result.rejected, 1);
        assert!(!result.outcomes[1].success);
        assert!(result.outcomes[1].error.is_some());
    }
}
