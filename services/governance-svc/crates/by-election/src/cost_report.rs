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

use crate::models::CostReport;

/// Build a CostReport for a by-election from its line items.
/// Validates total_kes == sum of components on construction (FR-M50-038).
/// Returns Err if totals mismatch (422 in HTTP layer).
pub fn build_cost_report(
    by_election_id: String,
    kit_days_kes: u64,
    staff_days_kes: u64,
    transport_kes: u64,
    printing_kes: u64,
    venue_kes: u64,
    other_kes: u64,
) -> Result<CostReport, String> {
    let total_kes = kit_days_kes
        .saturating_add(staff_days_kes)
        .saturating_add(transport_kes)
        .saturating_add(printing_kes)
        .saturating_add(venue_kes)
        .saturating_add(other_kes);

    let report = CostReport {
        id: uuid::Uuid::now_v7().to_string(),
        by_election_id,
        kit_days_kes,
        staff_days_kes,
        transport_kes,
        printing_kes,
        venue_kes,
        other_kes,
        total_kes,
        submitted_at: Some(chrono::Utc::now().to_rfc3339()),
        submitted_to_cob: false,
    };

    report.validate_total()?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cost_report_valid() {
        let report = build_cost_report(
            "be-1".into(),
            100_000,
            200_000,
            50_000,
            30_000,
            20_000,
            10_000,
        )
        .expect("valid report");
        assert_eq!(report.total_kes, 410_000);
        assert!(report.validate_total().is_ok());
    }

    #[test]
    fn all_seven_line_items_present() {
        let report = build_cost_report("be-2".into(), 1, 2, 3, 4, 5, 6).expect("valid");
        assert_eq!(report.kit_days_kes, 1);
        assert_eq!(report.staff_days_kes, 2);
        assert_eq!(report.transport_kes, 3);
        assert_eq!(report.printing_kes, 4);
        assert_eq!(report.venue_kes, 5);
        assert_eq!(report.other_kes, 6);
        assert_eq!(report.total_kes, 21);
    }

    #[test]
    fn validate_total_detects_mismatch() {
        let mut report = build_cost_report("be-3".into(), 100, 200, 50, 30, 20, 10).expect("valid");
        // Tamper with total
        report.total_kes = 999;
        assert!(report.validate_total().is_err());
    }

    #[test]
    fn zero_cost_report_valid() {
        let report = build_cost_report("be-4".into(), 0, 0, 0, 0, 0, 0).expect("zero is valid");
        assert_eq!(report.total_kes, 0);
    }
}
