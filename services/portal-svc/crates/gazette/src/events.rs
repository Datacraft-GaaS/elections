use serde_json::{json, Value};
use uuid::Uuid;

const SOURCE: &str = "spiffe://eel.local/service/portal-gazette";

pub fn calendar_event_scheduled_v1(election_id: &Uuid, calendar_event_id: &Uuid) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.calendar_event_scheduled.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("calendar-event/{}", calendar_event_id),
        "datacontenttype": "application/json",
        "data": {
            "election_id": election_id.to_string(),
            "calendar_event_id": calendar_event_id.to_string(),
        }
    })
}

pub fn notice_signed_v1(notice_id: &Uuid, hsm_key_ref: &str) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.notice_signed.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("gazette-notice/{}", notice_id),
        "datacontenttype": "application/json",
        "data": {
            "notice_id": notice_id.to_string(),
            "hsm_key_ref": hsm_key_ref,
        }
    })
}

pub fn gazette_submitted_v1(notice_id: &Uuid, gov_printer_ref: &str) -> Value {
    json!({
        "specversion": "1.0",
        "type": "eel.portal.gazette_submitted.v1",
        "source": SOURCE,
        "id": Uuid::now_v7().to_string(),
        "subject": format!("gazette-notice/{}", notice_id),
        "datacontenttype": "application/json",
        "data": {
            "notice_id": notice_id.to_string(),
            "gov_printer_ref": gov_printer_ref,
        }
    })
}
