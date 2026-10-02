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

//! Identity persistence tracker across a video frame sequence.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Per-frame tracking state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackState {
    /// Identity label for the tracked subject.
    pub identity_id: String,
    /// Number of frames the current identity has been tracked.
    pub frame_count: u32,
    /// Iris centre position (cx, cy) in image coordinates.
    pub last_position: (f32, f32),
}

/// Errors from tracking operations.
#[derive(Debug, Error)]
pub enum TrackerError {
    #[error("no frames have been processed yet")]
    NoFrames,
}

/// Tracks iris identity consistency across a video sequence.
///
/// A frame is consistent when the identity label matches the previous frame
/// and the iris centre has not drifted beyond `max_drift_px` pixels.
#[derive(Debug)]
pub struct IrisTracker {
    state: Option<TrackState>,
    max_drift_px: f32,
}

impl IrisTracker {
    pub fn new(max_drift_px: f32) -> Self {
        Self {
            state: None,
            max_drift_px,
        }
    }

    /// Update tracker with the iris centre and identity for the current frame.
    ///
    /// Returns `true` when identity is consistent with the previous frame
    /// (same `identity_id` AND drift within `max_drift_px`).
    /// On the very first call there is no prior state, so `true` is returned.
    pub fn update(&mut self, cx: f32, cy: f32, identity_id: &str) -> Result<bool, TrackerError> {
        let consistent = match &self.state {
            None => true,
            Some(prev) => {
                let dx = cx - prev.last_position.0;
                let dy = cy - prev.last_position.1;
                let drift = (dx * dx + dy * dy).sqrt();
                prev.identity_id == identity_id && drift <= self.max_drift_px
            }
        };

        let frame_count = if consistent {
            self.state.as_ref().map_or(1, |s| {
                if s.identity_id == identity_id {
                    s.frame_count + 1
                } else {
                    1
                }
            })
        } else {
            1
        };

        self.state = Some(TrackState {
            identity_id: identity_id.to_owned(),
            frame_count,
            last_position: (cx, cy),
        });

        Ok(consistent)
    }

    /// Returns true when at least one frame has been processed.
    pub fn is_tracking(&self) -> bool {
        self.state.is_some()
    }

    /// Number of consecutive frames the current identity has been tracked.
    pub fn frame_count(&self) -> u32 {
        self.state.as_ref().map_or(0, |s| s.frame_count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_update_returns_true() {
        let mut tracker = IrisTracker::new(20.0);
        assert!(tracker.update(100.0, 100.0, "subject-1").expect("update"));
    }

    #[test]
    fn is_tracking_false_before_any_update() {
        let tracker = IrisTracker::new(20.0);
        assert!(!tracker.is_tracking());
    }

    #[test]
    fn is_tracking_true_after_update() {
        let mut tracker = IrisTracker::new(20.0);
        tracker.update(50.0, 50.0, "subject-1").expect("update");
        assert!(tracker.is_tracking());
    }

    #[test]
    fn frame_count_zero_before_update() {
        let tracker = IrisTracker::new(20.0);
        assert_eq!(tracker.frame_count(), 0);
    }

    #[test]
    fn frame_count_increments_on_consistent_updates() {
        let mut tracker = IrisTracker::new(20.0);
        tracker.update(100.0, 100.0, "subject-1").expect("update");
        tracker.update(101.0, 100.0, "subject-1").expect("update");
        tracker.update(102.0, 100.0, "subject-1").expect("update");
        assert_eq!(tracker.frame_count(), 3);
    }

    #[test]
    fn large_drift_returns_false() {
        let mut tracker = IrisTracker::new(10.0);
        tracker.update(100.0, 100.0, "subject-1").expect("update");
        let consistent = tracker.update(200.0, 200.0, "subject-1").expect("update");
        assert!(!consistent);
    }

    #[test]
    fn small_drift_within_threshold_returns_true() {
        let mut tracker = IrisTracker::new(20.0);
        tracker.update(100.0, 100.0, "subject-1").expect("update");
        let consistent = tracker.update(105.0, 105.0, "subject-1").expect("update");
        assert!(consistent);
    }

    #[test]
    fn identity_change_returns_false() {
        let mut tracker = IrisTracker::new(20.0);
        tracker.update(100.0, 100.0, "subject-1").expect("update");
        let consistent = tracker.update(100.0, 100.0, "subject-2").expect("update");
        assert!(!consistent);
    }

    #[test]
    fn identity_change_resets_consistency_and_frame_count() {
        let mut tracker = IrisTracker::new(20.0);
        tracker.update(100.0, 100.0, "subject-1").expect("update");
        tracker.update(101.0, 100.0, "subject-1").expect("update");
        tracker.update(100.0, 100.0, "subject-2").expect("update");
        // After identity change, frame count resets for new identity.
        assert_eq!(tracker.frame_count(), 1);
    }

    #[test]
    fn drift_exactly_at_threshold_is_consistent() {
        let mut tracker = IrisTracker::new(10.0);
        tracker.update(0.0, 0.0, "subject-1").expect("update");
        // Drift of exactly 10.0 should be consistent (<=).
        let consistent = tracker.update(10.0, 0.0, "subject-1").expect("update");
        assert!(consistent);
    }
}
