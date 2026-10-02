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

use chrono::Duration;

use crate::domain::{CalendarEvent, CalendarEventType, GazetteNotice, NoticeType};
use uuid::Uuid;

/// Maps a CalendarEvent to derived GazetteNotices.
///
/// For PollingDay, derives a CampaignStart notice 60 days before.
/// Additional rule derivations can be added here.
pub fn derive_notices(event: &CalendarEvent) -> Vec<GazetteNotice> {
    match event.event_type {
        CalendarEventType::PollingDay => {
            let campaign_start_at = event.scheduled_at - Duration::days(60);
            vec![GazetteNotice {
                id: Uuid::now_v7(),
                notice_type: NoticeType::CampaignStart,
                body: format!(
                    "Pursuant to statutory requirements, the campaign period commences on {}. \
                     This is derived from the Polling Day scheduled on {}.",
                    campaign_start_at.format("%Y-%m-%d"),
                    event.scheduled_at.format("%Y-%m-%d")
                ),
                statutory_rule_ids: vec![],
                legal_approver_id: None,
                hsm_signature: None,
                published_at: None,
                gov_printer_ref: None,
            }]
        }
        CalendarEventType::NominationOpens => {
            vec![GazetteNotice {
                id: Uuid::now_v7(),
                notice_type: NoticeType::NominationNotice,
                body: format!(
                    "Nominations open on {}.",
                    event.scheduled_at.format("%Y-%m-%d")
                ),
                statutory_rule_ids: vec![],
                legal_approver_id: None,
                hsm_signature: None,
                published_at: None,
                gov_printer_ref: None,
            }]
        }
        CalendarEventType::ResultsPublication => {
            vec![GazetteNotice {
                id: Uuid::now_v7(),
                notice_type: NoticeType::ResultsNotice,
                body: format!(
                    "Results will be published on {}.",
                    event.scheduled_at.format("%Y-%m-%d")
                ),
                statutory_rule_ids: vec![],
                legal_approver_id: None,
                hsm_signature: None,
                published_at: None,
                gov_printer_ref: None,
            }]
        }
        _ => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn polling_day_event() -> CalendarEvent {
        CalendarEvent {
            id: Uuid::now_v7(),
            election_id: Uuid::now_v7(),
            event_type: CalendarEventType::PollingDay,
            scheduled_at: chrono::Utc.with_ymd_and_hms(2027, 8, 9, 6, 0, 0).unwrap(),
            gazette_ref: None,
            statutory_citation: None,
        }
    }

    #[test]
    fn polling_day_derives_campaign_start_60_days_before() {
        let event = polling_day_event();
        let notices = derive_notices(&event);
        assert_eq!(notices.len(), 1);
        let notice = &notices[0];
        assert_eq!(notice.notice_type, NoticeType::CampaignStart);
        // PollingDay 2027-08-09 => campaign start 2027-06-10
        assert!(notice.body.contains("2027-06-10"));
    }

    #[test]
    fn campaign_start_boundary_exact_60_days() {
        // boundary: exactly 60 days before
        let polling_day = chrono::Utc.with_ymd_and_hms(2027, 8, 9, 0, 0, 0).unwrap();
        let expected_campaign_start = chrono::Utc.with_ymd_and_hms(2027, 6, 10, 0, 0, 0).unwrap();
        let derived = polling_day - Duration::days(60);
        assert_eq!(derived, expected_campaign_start);
    }

    #[test]
    fn unknown_event_type_derives_no_notices() {
        let event = CalendarEvent {
            id: Uuid::now_v7(),
            election_id: Uuid::now_v7(),
            event_type: CalendarEventType::CampaignSilence,
            scheduled_at: chrono::Utc::now(),
            gazette_ref: None,
            statutory_citation: None,
        };
        let notices = derive_notices(&event);
        assert!(notices.is_empty());
    }

    #[test]
    fn nomination_opens_derives_nomination_notice() {
        let event = CalendarEvent {
            id: Uuid::now_v7(),
            election_id: Uuid::now_v7(),
            event_type: CalendarEventType::NominationOpens,
            scheduled_at: chrono::Utc::now(),
            gazette_ref: None,
            statutory_citation: None,
        };
        let notices = derive_notices(&event);
        assert_eq!(notices.len(), 1);
        assert_eq!(notices[0].notice_type, NoticeType::NominationNotice);
    }
}
