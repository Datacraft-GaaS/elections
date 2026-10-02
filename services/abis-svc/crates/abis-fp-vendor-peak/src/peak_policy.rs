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

//! PeakPolicy: controls when 2-of-3 vendor consensus is active.
//!
//! During the election freeze window (T-1..T+7) the policy is
//! EnabledDuringFreeze. Outside the window the system falls back to the
//! standard single-vendor path unless AlwaysEnabled is set.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Controls activation of the peak cross-vendor consensus mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PeakPolicy {
    /// Active only within [from, to].
    EnabledDuringFreeze {
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    },
    /// Always active regardless of time.
    AlwaysEnabled,
    /// Always inactive; standard single-vendor path.
    Disabled,
}

impl PeakPolicy {
    /// Returns true when the policy is currently active at `now`.
    pub fn is_active(&self, now: DateTime<Utc>) -> bool {
        match self {
            PeakPolicy::EnabledDuringFreeze { from, to } => now >= *from && now <= *to,
            PeakPolicy::AlwaysEnabled => true,
            PeakPolicy::Disabled => false,
        }
    }

    /// Returns the freeze window if the policy carries one.
    pub fn freeze_window(&self) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
        match self {
            PeakPolicy::EnabledDuringFreeze { from, to } => Some((*from, *to)),
            _ => None,
        }
    }
}

impl Default for PeakPolicy {
    fn default() -> Self {
        PeakPolicy::Disabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ts(y: i32, m: u32, d: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, 0, 0, 0).unwrap()
    }

    #[test]
    fn freeze_active_inside_window() {
        let p = PeakPolicy::EnabledDuringFreeze {
            from: ts(2027, 8, 8),
            to: ts(2027, 8, 16),
        };
        assert!(p.is_active(ts(2027, 8, 10)));
    }

    #[test]
    fn freeze_inactive_outside_window() {
        let p = PeakPolicy::EnabledDuringFreeze {
            from: ts(2027, 8, 8),
            to: ts(2027, 8, 16),
        };
        assert!(!p.is_active(ts(2027, 8, 7)));
        assert!(!p.is_active(ts(2027, 8, 17)));
    }

    #[test]
    fn always_enabled_is_always_active() {
        let p = PeakPolicy::AlwaysEnabled;
        assert!(p.is_active(ts(2020, 1, 1)));
        assert!(p.is_active(ts(2030, 12, 31)));
    }

    #[test]
    fn disabled_is_never_active() {
        let p = PeakPolicy::Disabled;
        assert!(!p.is_active(ts(2027, 8, 10)));
    }

    #[test]
    fn boundary_inclusive_from() {
        let p = PeakPolicy::EnabledDuringFreeze {
            from: ts(2027, 8, 8),
            to: ts(2027, 8, 16),
        };
        assert!(p.is_active(ts(2027, 8, 8)));
    }

    #[test]
    fn boundary_inclusive_to() {
        let p = PeakPolicy::EnabledDuringFreeze {
            from: ts(2027, 8, 8),
            to: ts(2027, 8, 16),
        };
        assert!(p.is_active(ts(2027, 8, 16)));
    }
}
