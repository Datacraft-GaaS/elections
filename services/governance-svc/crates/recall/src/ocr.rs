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

use crate::models::{RowError, SignatoryImportResult, SignatoryImportRow};

/// OCR confidence gate: rows below this score are flagged for manual re-entry.
/// Configurable in production; default from FR-M48-005.
pub const DEFAULT_OCR_CONFIDENCE_GATE: f64 = 0.80;

/// Process a batch of OCR-imported signatory rows.
///
/// Rules applied per FR-M48-005 to FR-M48-008:
/// - Rows with ocr_confidence < gate are flagged for manual re-entry (batch continues).
/// - voter_id must be non-empty (format validation).
/// - Duplicates within the batch are rejected; per-row result returned.
/// - Valid rows are returned in the accepted list for CVR cross-reference.
///
/// Image deletion (FR-M48-019) is the caller's responsibility: delete the source
/// image from storage within 1 s of calling this function.
pub fn process_batch(
    batch_id: String,
    rows: Vec<SignatoryImportRow>,
    gate: f64,
    existing_voter_ids: &std::collections::HashSet<String>,
) -> SignatoryImportResult {
    let total_rows = rows.len();
    let mut accepted = 0usize;
    let mut flagged_for_manual = 0usize;
    let mut duplicates = 0usize;
    let not_on_roll: usize = 0;
    let mut errors = Vec::new();
    let mut seen_in_batch = std::collections::HashSet::new();

    for (i, row) in rows.into_iter().enumerate() {
        // Format validation per FR-M48-017
        if row.voter_id.is_empty() {
            errors.push(RowError {
                row_index: i,
                voter_id: row.voter_id,
                reason: "voter_id is empty".to_string(),
            });
            continue;
        }

        // OCR confidence gate per FR-M48-005
        if row.ocr_confidence.map(|c| c < gate).unwrap_or(false) {
            flagged_for_manual += 1;
            errors.push(RowError {
                row_index: i,
                voter_id: row.voter_id,
                reason: format!(
                    "ocr_confidence {:.3} below gate {:.3}; manual re-entry required",
                    row.ocr_confidence.unwrap_or(0.0),
                    gate
                ),
            });
            continue;
        }

        // Duplicate in current batch per FR-M48-007
        if !seen_in_batch.insert(row.voter_id.clone()) {
            duplicates += 1;
            errors.push(RowError {
                row_index: i,
                voter_id: row.voter_id,
                reason: "duplicate voter_id within this batch".to_string(),
            });
            continue;
        }

        // Duplicate in existing petition signatories per FR-M48-007
        if existing_voter_ids.contains(&row.voter_id) {
            duplicates += 1;
            errors.push(RowError {
                row_index: i,
                voter_id: row.voter_id,
                reason: "voter_id already recorded in petition".to_string(),
            });
            continue;
        }

        // NOT_ON_ROLL check would be performed via M02 CVR cross-reference in production.
        // Stub: assume on roll if voter_id passes format check above.
        let _ = not_on_roll; // suppresses unused warning; real cross-ref increments this

        accepted += 1;
    }

    SignatoryImportResult {
        batch_id,
        total_rows,
        accepted,
        flagged_for_manual,
        duplicates,
        not_on_roll,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn row(voter_id: &str, confidence: Option<f64>) -> SignatoryImportRow {
        SignatoryImportRow {
            voter_id: voter_id.to_string(),
            name: "Test Voter".to_string(),
            ward: "Ward 1".to_string(),
            signature_date: "2025-01-01".to_string(),
            ocr_confidence: confidence,
        }
    }

    #[test]
    fn accepts_high_confidence_rows() {
        let rows = vec![row("V001", Some(0.95)), row("V002", Some(0.85))];
        let result = process_batch(
            "B1".into(),
            rows,
            DEFAULT_OCR_CONFIDENCE_GATE,
            &HashSet::new(),
        );
        assert_eq!(result.accepted, 2);
        assert_eq!(result.flagged_for_manual, 0);
        assert_eq!(result.duplicates, 0);
    }

    #[test]
    fn flags_low_confidence_rows() {
        let rows = vec![row("V001", Some(0.70)), row("V002", Some(0.95))];
        let result = process_batch(
            "B2".into(),
            rows,
            DEFAULT_OCR_CONFIDENCE_GATE,
            &HashSet::new(),
        );
        assert_eq!(result.accepted, 1);
        assert_eq!(result.flagged_for_manual, 1);
    }

    #[test]
    fn rejects_duplicate_within_batch() {
        let rows = vec![row("V001", Some(0.90)), row("V001", Some(0.92))];
        let result = process_batch(
            "B3".into(),
            rows,
            DEFAULT_OCR_CONFIDENCE_GATE,
            &HashSet::new(),
        );
        assert_eq!(result.accepted, 1);
        assert_eq!(result.duplicates, 1);
    }

    #[test]
    fn rejects_existing_petition_signatory() {
        let mut existing = HashSet::new();
        existing.insert("V001".to_string());
        let rows = vec![row("V001", Some(0.95))];
        let result = process_batch("B4".into(), rows, DEFAULT_OCR_CONFIDENCE_GATE, &existing);
        assert_eq!(result.accepted, 0);
        assert_eq!(result.duplicates, 1);
    }

    #[test]
    fn rejects_empty_voter_id() {
        let rows = vec![row("", Some(0.95))];
        let result = process_batch(
            "B5".into(),
            rows,
            DEFAULT_OCR_CONFIDENCE_GATE,
            &HashSet::new(),
        );
        assert_eq!(result.accepted, 0);
        assert_eq!(result.errors.len(), 1);
    }

    #[test]
    fn batch_continues_after_flagged_row() {
        // Partial batch failure: flagged row doesn't block valid rows -- EC-DAT-07
        let rows = vec![
            row("V001", Some(0.50)), // flagged
            row("V002", Some(0.95)), // accepted
            row("V003", Some(0.91)), // accepted
        ];
        let result = process_batch(
            "B6".into(),
            rows,
            DEFAULT_OCR_CONFIDENCE_GATE,
            &HashSet::new(),
        );
        assert_eq!(result.accepted, 2);
        assert_eq!(result.flagged_for_manual, 1);
    }
}
