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

use chrono::{Duration, TimeZone, Utc};
use eel_portal_gazette::{
    derive_notices,
    domain::{CalendarEvent, CalendarEventType, NoticeType},
    events::{gazette_submitted_v1, notice_signed_v1},
};
use uuid::Uuid;

fn polling_day(year: i32, month: u32, day: u32) -> CalendarEvent {
    CalendarEvent {
        id: Uuid::now_v7(),
        election_id: Uuid::now_v7(),
        event_type: CalendarEventType::PollingDay,
        scheduled_at: Utc.with_ymd_and_hms(year, month, day, 6, 0, 0).unwrap(),
        gazette_ref: None,
        statutory_citation: None,
    }
}

#[test]
fn polling_day_derives_campaign_start_exactly_60_days_before() {
    let event = polling_day(2027, 8, 9);
    let notices = derive_notices(&event);
    assert_eq!(notices.len(), 1, "should derive exactly one notice");
    assert_eq!(notices[0].notice_type, NoticeType::CampaignStart);
    let expected = event.scheduled_at - Duration::days(60);
    assert!(
        notices[0]
            .body
            .contains(&expected.format("%Y-%m-%d").to_string()),
        "notice body should reference campaign start date"
    );
}

#[test]
fn boundary_60_days_before_polling_day() {
    // 2028-02-29 is a leap year; 60 days before = 2027-12-31 (non-leap boundary)
    let event = polling_day(2028, 2, 29);
    let notices = derive_notices(&event);
    assert_eq!(notices.len(), 1);
    let campaign_start = event.scheduled_at - Duration::days(60);
    let expected_date = "2027-12-31";
    assert!(
        notices[0].body.contains(expected_date),
        "expected campaign start date {} in body",
        expected_date
    );
    assert_eq!(campaign_start.format("%Y-%m-%d").to_string(), expected_date);
}

#[test]
fn notice_sign_event_has_correct_type() {
    let notice_id = Uuid::now_v7();
    let ev = notice_signed_v1(&notice_id, "hsm://eel.local/key");
    assert_eq!(ev["type"], "eel.portal.notice_signed.v1");
    assert_eq!(ev["data"]["notice_id"], notice_id.to_string());
    assert_eq!(ev["data"]["hsm_key_ref"], "hsm://eel.local/key");
}

#[test]
fn gazette_submitted_event_is_idempotent_by_ref() {
    // Idempotency: same notice_id produces same gov_printer_ref
    let notice_id = Uuid::now_v7();
    let ref1 = format!("GPR-{}", notice_id);
    let ev1 = gazette_submitted_v1(&notice_id, &ref1);
    let ev2 = gazette_submitted_v1(&notice_id, &ref1);
    // Both events carry the same notice_id and gov_printer_ref
    assert_eq!(ev1["data"]["notice_id"], ev2["data"]["notice_id"]);
    assert_eq!(
        ev1["data"]["gov_printer_ref"],
        ev2["data"]["gov_printer_ref"]
    );
    // Each event gets a distinct id (not idempotent at envelope level, only at data level)
    assert_ne!(ev1["id"], ev2["id"]);
}

#[test]
fn calendar_event_scheduled_event_has_correct_type() {
    use eel_portal_gazette::events::calendar_event_scheduled_v1;
    let election_id = Uuid::now_v7();
    let calendar_event_id = Uuid::now_v7();
    let ev = calendar_event_scheduled_v1(&election_id, &calendar_event_id);
    assert_eq!(ev["type"], "eel.portal.calendar_event_scheduled.v1");
    assert_eq!(ev["data"]["election_id"], election_id.to_string());
}

#[test]
fn non_polling_day_events_derive_no_campaign_start() {
    let silence_event = CalendarEvent {
        id: Uuid::now_v7(),
        election_id: Uuid::now_v7(),
        event_type: CalendarEventType::CampaignSilence,
        scheduled_at: Utc::now(),
        gazette_ref: None,
        statutory_citation: None,
    };
    let notices = derive_notices(&silence_event);
    assert!(
        notices
            .iter()
            .all(|n| n.notice_type != NoticeType::CampaignStart),
        "CampaignSilence should not derive CampaignStart"
    );
}

#[test]
fn notice_signed_event_subject_contains_notice_id() {
    let notice_id = Uuid::now_v7();
    let ev = notice_signed_v1(&notice_id, "key-ref");
    let subject = ev["subject"].as_str().unwrap();
    assert!(subject.contains(&notice_id.to_string()));
}
