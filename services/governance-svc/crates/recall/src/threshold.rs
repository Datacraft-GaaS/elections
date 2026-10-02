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

/// Art. 104(2) threshold: 30% of registered voters in the constituency.
/// All arithmetic is integer-only (no floating point). Semgrep enforces this.
///
/// Returns true iff verified_signatories >= ceil(registered_voters * 30 / 100).
/// Implemented as: verified_signatories * 100 >= registered_voters * 30
/// to avoid any division or floating-point.
pub fn meets_art_104_2_threshold(verified_signatories: u64, registered_voters: u64) -> bool {
    if registered_voters == 0 {
        return false;
    }
    // verified_signatories / registered_voters >= 30/100
    // => verified_signatories * 100 >= registered_voters * 30
    verified_signatories
        .checked_mul(100)
        .zip(registered_voters.checked_mul(30))
        .map(|(lhs, rhs)| lhs >= rhs)
        .unwrap_or(false)
}

/// Art. 104(3) success threshold for a recall election:
/// YES votes must be a majority of registered voters (not just votes cast).
/// i.e., yes_votes > registered_voters / 2  (integer majority: yes_votes * 2 > registered_voters)
pub fn meets_art_104_3_threshold(yes_votes: u64, registered_voters: u64) -> bool {
    if registered_voters == 0 {
        return false;
    }
    // yes_votes > registered_voters / 2
    // => yes_votes * 2 > registered_voters
    yes_votes
        .checked_mul(2)
        .map(|lhs| lhs > registered_voters)
        .unwrap_or(false)
}

/// Compute the threshold report numerator and denominator for display.
/// Returns (verified_signatories * 100, registered_voters * 30).
pub fn threshold_numerator_denominator(
    verified_signatories: u64,
    registered_voters: u64,
) -> (u64, u64) {
    (
        verified_signatories.saturating_mul(100),
        registered_voters.saturating_mul(30),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Art. 104(2) boundary tests -- AC-M48-01
    #[test]
    fn threshold_exact_30pct() {
        // 30 out of 100: exactly at threshold
        assert!(meets_art_104_2_threshold(30, 100));
    }

    #[test]
    fn threshold_below_30pct() {
        // 29 out of 100: below threshold (29.99...)
        assert!(!meets_art_104_2_threshold(29, 100));
    }

    #[test]
    fn threshold_above_30pct() {
        // 31 out of 100: above threshold
        assert!(meets_art_104_2_threshold(31, 100));
    }

    #[test]
    fn threshold_large_constituency() {
        // 90_000 out of 300_000: exactly 30%
        assert!(meets_art_104_2_threshold(90_000, 300_000));
        // 89_999 out of 300_000: just below
        assert!(!meets_art_104_2_threshold(89_999, 300_000));
    }

    #[test]
    fn threshold_zero_registered() {
        assert!(!meets_art_104_2_threshold(100, 0));
    }

    // Art. 104(3) recall election success threshold boundary tests -- AC-M48-15
    #[test]
    fn recall_election_majority_yes() {
        // 51 yes out of 100 registered: success
        assert!(meets_art_104_3_threshold(51, 100));
    }

    #[test]
    fn recall_election_exact_half_not_success() {
        // 50 yes out of 100 registered: NOT a majority (must be more than half)
        assert!(!meets_art_104_3_threshold(50, 100));
    }

    #[test]
    fn recall_election_below_majority() {
        // 49 yes out of 100 registered: fails
        assert!(!meets_art_104_3_threshold(49, 100));
    }

    #[test]
    fn no_fp_in_threshold() {
        // Confirm no float ops: this is a compile-time guarantee by the types,
        // but we do a spot check that the pure integer path is used.
        let (num, den) = threshold_numerator_denominator(30, 100);
        assert_eq!(num, 3000);
        assert_eq!(den, 3000);
        assert!(num >= den);
    }
}
