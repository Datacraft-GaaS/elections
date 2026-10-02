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

//! Linear view-change pacemaker with exponential timeout backoff.
//!
//! The pacemaker triggers view changes when the current leader appears faulty
//! (no progress within the timeout window). Timeout doubles each successive
//! failure up to a configured maximum.
//!
//! Reference: Yin, Malkhi, Reiter, Gueta, Abraham.
//!   "HotStuff: BFT Consensus with Linearity and Responsiveness."
//!   PODC 2019. Section 6 (Pacemaker).

use std::time::Duration;
use tracing::{info, warn};

/// Exponential backoff pacemaker.
pub struct Pacemaker {
    /// Base timeout for view 0 and after successful commits.
    base_timeout: Duration,
    /// Maximum timeout (cap on exponential growth).
    max_timeout: Duration,
    /// Current timeout (grows on failure).
    current_timeout: Duration,
    /// Number of consecutive view-changes without a commit.
    consecutive_failures: u32,
    /// Current view number tracked by the pacemaker.
    pub current_view: u64,
}

impl Pacemaker {
    pub fn new(base_timeout: Duration, max_timeout: Duration) -> Self {
        Self {
            base_timeout,
            max_timeout,
            current_timeout: base_timeout,
            consecutive_failures: 0,
            current_view: 0,
        }
    }

    /// Current view timeout.
    pub fn timeout(&self) -> Duration {
        self.current_timeout
    }

    /// Called when a block is committed: reset backoff, advance view.
    pub fn on_commit(&mut self) {
        self.consecutive_failures = 0;
        self.current_timeout = self.base_timeout;
        self.current_view += 1;
        info!(
            view = self.current_view,
            timeout_ms = self.current_timeout.as_millis(),
            "pacemaker: commit, view advance"
        );
    }

    /// Called when a view-change is needed (leader timeout).
    ///
    /// Returns the new view number the replica should move to.
    pub fn on_timeout(&mut self) -> u64 {
        self.consecutive_failures += 1;
        // Double the timeout, capped at max_timeout.
        self.current_timeout = (self.current_timeout * 2).min(self.max_timeout);
        self.current_view += 1;
        warn!(
            view = self.current_view,
            failures = self.consecutive_failures,
            timeout_ms = self.current_timeout.as_millis(),
            "pacemaker: timeout, view-change"
        );
        self.current_view
    }

    /// Reset to initial state (e.g., after a node restart).
    pub fn reset(&mut self) {
        self.current_view = 0;
        self.consecutive_failures = 0;
        self.current_timeout = self.base_timeout;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pm() -> Pacemaker {
        Pacemaker::new(Duration::from_millis(500), Duration::from_millis(8000))
    }

    #[test]
    fn initial_timeout_is_base() {
        let p = pm();
        assert_eq!(p.timeout(), Duration::from_millis(500));
    }

    #[test]
    fn timeout_doubles_on_failure() {
        let mut p = pm();
        p.on_timeout();
        assert_eq!(p.timeout(), Duration::from_millis(1000));
        p.on_timeout();
        assert_eq!(p.timeout(), Duration::from_millis(2000));
    }

    #[test]
    fn timeout_capped_at_max() {
        let mut p = pm();
        for _ in 0..10 {
            p.on_timeout();
        }
        assert_eq!(p.timeout(), Duration::from_millis(8000));
    }

    #[test]
    fn commit_resets_timeout() {
        let mut p = pm();
        p.on_timeout();
        p.on_timeout();
        p.on_commit();
        assert_eq!(p.timeout(), Duration::from_millis(500));
    }

    #[test]
    fn view_increments_on_timeout() {
        let mut p = pm();
        let v = p.on_timeout();
        assert_eq!(v, 1);
        let v2 = p.on_timeout();
        assert_eq!(v2, 2);
    }

    #[test]
    fn view_increments_on_commit() {
        let mut p = pm();
        p.on_commit();
        assert_eq!(p.current_view, 1);
    }

    #[test]
    fn reset_returns_to_initial_state() {
        let mut p = pm();
        p.on_timeout();
        p.on_timeout();
        p.reset();
        assert_eq!(p.current_view, 0);
        assert_eq!(p.timeout(), Duration::from_millis(500));
    }
}
