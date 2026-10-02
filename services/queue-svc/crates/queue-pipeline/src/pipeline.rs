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

use chrono::Utc;
use uuid::Uuid;

use crate::models::{
    DisenfranchisementFlagEvent, QueueSnapshot, S14aRecommendedEvent, SignatureFusionInput,
    WaitTimeBreachedEvent,
};

const WAIT_DISENFRANCHISEMENT_THRESHOLD: u32 = 45;
const HIGH_THROUGHPUT_THRESHOLD: u32 = 200;

pub fn fuse_snapshot(station_code: &str, input: SignatureFusionInput) -> QueueSnapshot {
    let max_headcount = input
        .cv_headcount
        .max(input.clerk_report_count)
        .max(input.qr_scan_count);

    let throughput = if input.kit_throughput_per_hour > 0 {
        input.kit_throughput_per_hour
    } else {
        1
    };

    let estimated_wait_minutes = (max_headcount * 60) / throughput;

    let high_throughput = input.kit_throughput_per_hour > HIGH_THROUGHPUT_THRESHOLD
        || input.cv_headcount > HIGH_THROUGHPUT_THRESHOLD
        || input.clerk_report_count > HIGH_THROUGHPUT_THRESHOLD
        || input.qr_scan_count > HIGH_THROUGHPUT_THRESHOLD;

    let disenfranchisement_flag =
        estimated_wait_minutes > WAIT_DISENFRANCHISEMENT_THRESHOLD || high_throughput;

    let s14a_recommended = disenfranchisement_flag;

    QueueSnapshot {
        station_code: station_code.to_string(),
        kit_throughput_per_hour: input.kit_throughput_per_hour,
        clerk_report_count: input.clerk_report_count,
        cv_headcount: input.cv_headcount,
        qr_scan_count: input.qr_scan_count,
        estimated_wait_minutes,
        disenfranchisement_flag,
        s14a_recommended,
        fused_at: Utc::now(),
        signature: None,
    }
}

pub fn check_wait_breach(
    snapshot: &QueueSnapshot,
    threshold_minutes: u32,
) -> Option<WaitTimeBreachedEvent> {
    if snapshot.estimated_wait_minutes > threshold_minutes {
        Some(WaitTimeBreachedEvent {
            event_type: "eel.queue.wait_time_breached.v1".to_string(),
            station_code: snapshot.station_code.clone(),
            wait_minutes: snapshot.estimated_wait_minutes,
            threshold_minutes,
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        })
    } else {
        None
    }
}

pub fn check_disenfranchisement(snapshot: &QueueSnapshot) -> Option<DisenfranchisementFlagEvent> {
    if snapshot.disenfranchisement_flag {
        Some(DisenfranchisementFlagEvent {
            event_type: "eel.queue.disenfranchisement_flag.v1".to_string(),
            station_code: snapshot.station_code.clone(),
            reason: format!(
                "estimated wait {} min exceeds threshold or high throughput detected",
                snapshot.estimated_wait_minutes
            ),
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        })
    } else {
        None
    }
}

pub fn recommend_s14a(snapshot: &QueueSnapshot) -> Option<S14aRecommendedEvent> {
    if snapshot.s14a_recommended {
        Some(S14aRecommendedEvent {
            event_type: "eel.queue.s14a_recommended.v1".to_string(),
            station_code: snapshot.station_code.clone(),
            recommendation_id: Uuid::new_v4(),
            hsm_pending: true,
            event_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> SignatureFusionInput {
        // throughput=200, max_headcount=60 -> wait = 60*60/200 = 18 min, no flags
        SignatureFusionInput {
            kit_throughput_per_hour: 200,
            clerk_report_count: 50,
            cv_headcount: 60,
            qr_scan_count: 55,
        }
    }

    #[test]
    fn test_fuse_snapshot_basic() {
        let input = base_input();
        let snap = fuse_snapshot("STN001", input);
        assert_eq!(snap.station_code, "STN001");
        assert_eq!(snap.kit_throughput_per_hour, 200);
        assert_eq!(snap.cv_headcount, 60);
        assert!(snap.estimated_wait_minutes > 0);
        assert!(!snap.disenfranchisement_flag);
        assert!(!snap.s14a_recommended);
    }

    #[test]
    fn test_fuse_snapshot_high_wait_flags() {
        let input = SignatureFusionInput {
            kit_throughput_per_hour: 10,
            clerk_report_count: 50,
            cv_headcount: 60,
            qr_scan_count: 55,
        };
        let snap = fuse_snapshot("STN002", input);
        assert!(snap.estimated_wait_minutes > WAIT_DISENFRANCHISEMENT_THRESHOLD);
        assert!(snap.disenfranchisement_flag);
        assert!(snap.s14a_recommended);
    }

    #[test]
    fn test_fuse_snapshot_high_throughput_flags() {
        let input = SignatureFusionInput {
            kit_throughput_per_hour: 300,
            clerk_report_count: 50,
            cv_headcount: 50,
            qr_scan_count: 50,
        };
        let snap = fuse_snapshot("STN003", input);
        assert!(snap.disenfranchisement_flag);
        assert!(snap.s14a_recommended);
    }

    #[test]
    fn test_wait_breach_fires() {
        let input = SignatureFusionInput {
            kit_throughput_per_hour: 5,
            clerk_report_count: 100,
            cv_headcount: 100,
            qr_scan_count: 100,
        };
        let snap = fuse_snapshot("STN004", input);
        let event = check_wait_breach(&snap, 30);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert_eq!(ev.station_code, "STN004");
        assert_eq!(ev.threshold_minutes, 30);
    }

    #[test]
    fn test_wait_breach_no_fire() {
        let snap = fuse_snapshot("STN005", base_input());
        let event = check_wait_breach(&snap, 999);
        assert!(event.is_none());
    }

    #[test]
    fn test_s14a_recommended_when_flagged() {
        let input = SignatureFusionInput {
            kit_throughput_per_hour: 5,
            clerk_report_count: 100,
            cv_headcount: 100,
            qr_scan_count: 100,
        };
        let snap = fuse_snapshot("STN006", input);
        let event = recommend_s14a(&snap);
        assert!(event.is_some());
        let ev = event.unwrap();
        assert!(ev.hsm_pending);
        assert_eq!(ev.station_code, "STN006");
    }

    #[test]
    fn test_s14a_not_recommended_normal() {
        let snap = fuse_snapshot("STN007", base_input());
        let event = recommend_s14a(&snap);
        assert!(event.is_none());
    }
}
