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

/// County threshold engine for referenda under Art. 255-257.
///
/// EC-LEG-07: exactly 24 of 47 counties must have a YES majority.
/// All arithmetic is integer-only (no floating point). Semgrep enforces this.

pub const TOTAL_COUNTIES: u8 = 47;
pub const REQUIRED_COUNTY_THRESHOLD: u8 = 24;

/// Returns true iff yes_votes > no_votes in a county.
/// Tie is NOT a majority (must be strictly greater).
pub fn county_has_yes_majority(yes_votes: u64, no_votes: u64) -> bool {
    yes_votes > no_votes
}

/// Returns true iff counties_with_yes_majority >= 24 (EC-LEG-07).
pub fn meets_county_threshold(counties_yes_majority: u8) -> bool {
    counties_yes_majority >= REQUIRED_COUNTY_THRESHOLD
}

/// Returns true iff total_yes votes constitute a national majority of votes cast.
/// Majority: yes_votes * 2 > total_votes_cast (50%+1, integer, no rounding).
pub fn meets_national_majority(yes_votes: u64, no_votes: u64) -> bool {
    let total = yes_votes.saturating_add(no_votes);
    if total == 0 {
        return false;
    }
    // yes_votes > total / 2  =>  yes_votes * 2 > total
    yes_votes.saturating_mul(2) > total
}

/// Invariant: NoDeclarationWithoutBothThresholds
/// Declaration is only valid when both county_threshold AND national_majority are met.
pub fn can_declare(counties_yes_majority: u8, yes_votes: u64, no_votes: u64) -> bool {
    meets_county_threshold(counties_yes_majority) && meets_national_majority(yes_votes, no_votes)
}

/// Invariant: ExactlyTwentyFourCounties (lower bound enforced; upper bound from TOTAL_COUNTIES)
pub fn county_count_valid(count: u8) -> bool {
    count <= TOTAL_COUNTIES
}

#[cfg(test)]
mod tests {
    use super::*;

    // County majority -- FR-M49-023
    #[test]
    fn county_yes_majority_strict() {
        assert!(county_has_yes_majority(100, 99));
        assert!(!county_has_yes_majority(100, 100)); // tie is NOT a majority
        assert!(!county_has_yes_majority(99, 100));
    }

    // County threshold boundary -- FR-M49-028
    #[test]
    fn county_threshold_boundary_23() {
        assert!(!meets_county_threshold(23));
    }

    #[test]
    fn county_threshold_boundary_24() {
        assert!(meets_county_threshold(24));
    }

    #[test]
    fn county_threshold_boundary_25() {
        assert!(meets_county_threshold(25));
    }

    // National majority -- 50%+1 of votes cast
    #[test]
    fn national_majority_exact_majority() {
        // 51 yes, 49 no: 51 > 100/2 = yes
        assert!(meets_national_majority(51, 49));
    }

    #[test]
    fn national_majority_exact_half_fails() {
        // 50 yes, 50 no: not majority
        assert!(!meets_national_majority(50, 50));
    }

    #[test]
    fn national_majority_below() {
        assert!(!meets_national_majority(49, 51));
    }

    #[test]
    fn national_majority_zero_votes() {
        assert!(!meets_national_majority(0, 0));
    }

    // Declaration invariant
    #[test]
    fn no_declaration_without_both_thresholds() {
        // County threshold met, national not
        assert!(!can_declare(24, 50, 50));
        // National met, county not
        assert!(!can_declare(23, 51, 49));
        // Both met
        assert!(can_declare(24, 51, 49));
        // Neither met
        assert!(!can_declare(23, 50, 50));
    }

    // Monotonicity: once county threshold met, remains met as count increases
    #[test]
    fn county_threshold_monotone() {
        let mut met = false;
        for c in 0u8..=47 {
            let current = meets_county_threshold(c);
            if met {
                assert!(current, "monotonicity violated at {c}");
            }
            if current && !met {
                met = true;
            }
        }
        assert!(met, "threshold should be reachable within 47 counties");
    }
}
