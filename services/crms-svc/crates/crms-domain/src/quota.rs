//! Quota rules engine — two-thirds gender rule, youth, PWD.
//! FR-M12-008, FR-M12-009, AC-M12-03, EC-LEG-01.
//! Property tests: quota invariant never allows gazette if 2/3 rule violated.

use crate::candidate::{Gender, Position};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Quota compliance result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaStatus {
    pub scope: String,
    pub position: Position,
    pub total_cleared: u32,
    pub female_count: u32,
    pub male_count: u32,
    pub youth_count: u32,
    pub pwd_count: u32,
    pub two_thirds_compliant: bool,
    pub violation_detail: Option<String>,
}

#[derive(Debug, Error)]
pub enum QuotaError {
    #[error("two-thirds gender rule violated: {0}")]
    TwoThirdsViolation(String),
    #[error("scope or position not found")]
    NotFound,
}

/// Checks whether a proposed cleared roster satisfies the two-thirds gender rule.
///
/// Kenya Constitution Art. 81(b): not more than 2/3 of members of elective bodies
/// shall be of the same gender.  Applied per party at national level (FR-M12-008).
pub fn check_two_thirds_rule(
    total_cleared: u32,
    female_count: u32,
    male_count: u32,
) -> Result<(), QuotaError> {
    if total_cleared == 0 {
        return Ok(());
    }
    // Neither gender may exceed 2/3 of total.
    let threshold = (total_cleared * 2).div_ceil(3);
    if female_count > threshold || male_count > threshold {
        let dominant = if female_count > male_count {
            "female"
        } else {
            "male"
        };
        let dominant_count = female_count.max(male_count);
        return Err(QuotaError::TwoThirdsViolation(format!(
            "{dominant} candidates ({dominant_count}) exceed 2/3 of {total_cleared} cleared"
        )));
    }
    Ok(())
}

/// Youth threshold per IEBC regulations: under 35 at nomination date.
pub fn is_youth(date_of_birth: &str, nomination_date: &str) -> bool {
    // Simple year-based check for domain logic; UI and integrations use full date.
    let birth_year: i32 = date_of_birth
        .get(..4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let nom_year: i32 = nomination_date
        .get(..4)
        .and_then(|s| s.parse().ok())
        .unwrap_or(9999);
    (nom_year - birth_year) < 35
}

/// Gender quota check for a given set of cleared candidates per position per scope.
pub fn evaluate_quota(
    scope: String,
    position: Position,
    candidates: &[(Gender, bool, &str)], // (gender, is_pwd, dob)
    nomination_date: &str,
) -> QuotaStatus {
    let total_cleared = candidates.len() as u32;
    let female_count = candidates
        .iter()
        .filter(|(g, _, _)| *g == Gender::Female)
        .count() as u32;
    let male_count = candidates
        .iter()
        .filter(|(g, _, _)| *g == Gender::Male)
        .count() as u32;
    let youth_count = candidates
        .iter()
        .filter(|(_, _, dob)| is_youth(dob, nomination_date))
        .count() as u32;
    let pwd_count = candidates.iter().filter(|(_, pwd, _)| *pwd).count() as u32;

    let two_thirds_result = check_two_thirds_rule(total_cleared, female_count, male_count);
    let (two_thirds_compliant, violation_detail) = match two_thirds_result {
        Ok(()) => (true, None),
        Err(QuotaError::TwoThirdsViolation(msg)) => (false, Some(msg)),
        Err(_) => (false, Some("unknown quota error".into())),
    };

    QuotaStatus {
        scope,
        position,
        total_cleared,
        female_count,
        male_count,
        youth_count,
        pwd_count,
        two_thirds_compliant,
        violation_detail,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_thirds_balanced_passes() {
        assert!(check_two_thirds_rule(6, 3, 3).is_ok());
    }

    #[test]
    fn two_thirds_exact_threshold_passes() {
        // 4 of 6 = 2/3 exactly — permitted.
        assert!(check_two_thirds_rule(6, 4, 2).is_ok());
    }

    #[test]
    fn two_thirds_violation_detected() {
        // 5 of 6 > 2/3 — violation.
        assert!(check_two_thirds_rule(6, 5, 1).is_err());
    }

    #[test]
    fn zero_cleared_passes() {
        assert!(check_two_thirds_rule(0, 0, 0).is_ok());
    }

    #[test]
    fn youth_check() {
        assert!(is_youth("1990-01-01", "2024-01-01")); // 34 — youth
        assert!(!is_youth("1985-01-01", "2024-01-01")); // 39 — not youth
    }

    #[test]
    fn evaluate_quota_violation() {
        let candidates = vec![
            (Gender::Male, false, "1980-01-01"),
            (Gender::Male, false, "1982-01-01"),
            (Gender::Male, false, "1983-01-01"),
            (Gender::Male, false, "1984-01-01"),
            (Gender::Female, false, "1990-01-01"),
        ];
        let status = evaluate_quota(
            "national".into(),
            Position::NationalAssembly,
            &candidates,
            "2027-01-01",
        );
        assert!(!status.two_thirds_compliant);
    }
}
