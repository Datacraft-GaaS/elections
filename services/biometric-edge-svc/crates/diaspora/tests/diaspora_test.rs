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

//! Integration tests for the eel-diaspora crate (WP-5.8, M24).

use chrono::{NaiveDate, Utc};
use eel_diaspora::{
    domain::{CustodyEvent, EmbassyStation, Form34ADiaspora, PouchCustody},
    import::{import_roll_csv, ImportProfile},
    roll::{roll_from_voter_ids, snapshot_hash},
    transmission::{attempt_delivery, batch_transmit, BatchStatus},
    tz::local_to_utc,
};
use proptest::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// ---- TZ round-trip across all 5 profiles -----------------------------------

proptest! {
    #[test]
    fn prop_tz_roundtrip_all_profiles(hour in 1u32..22, minute in 0u32..59) {
        let profiles = [
            ("Africa/Nairobi",   "2027-08-10"),
            ("Europe/London",    "2027-08-10"),
            ("America/New_York", "2027-08-10"),
            ("Asia/Dubai",       "2027-08-10"),
            ("Australia/Sydney", "2027-08-10"),
        ];
        for (iana, date_str) in profiles {
            let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
            let local = date.and_hms_opt(hour, minute, 0).unwrap();
            let utc = local_to_utc(iana, local).unwrap();
            let back = eel_diaspora::tz::utc_to_local(iana, utc).unwrap().naive_local();
            prop_assert_eq!(back, local, "round-trip failed for {}", iana);
        }
    }
}

// ---- Roll freeze idempotency -----------------------------------------------

#[test]
fn roll_freeze_is_idempotent() {
    let station = Uuid::now_v7();
    let roll = roll_from_voter_ids(station, vec!["V1".into(), "V2".into(), "V3".into()]);
    assert!(!roll.frozen);
    let frozen = roll.clone().freeze(Utc::now());
    assert!(frozen.frozen);
    let frozen2 = frozen.clone().freeze(Utc::now());
    assert_eq!(frozen.id, frozen2.id);
    assert_eq!(frozen.hash, frozen2.hash);
    assert_eq!(frozen.voter_count, frozen2.voter_count);
    assert!(frozen2.frozen);
}

#[test]
fn roll_hash_is_deterministic_and_order_independent() {
    let h1 = snapshot_hash(&mut ["C".into(), "A".into(), "B".into()]);
    let h2 = snapshot_hash(&mut ["A".into(), "B".into(), "C".into()]);
    assert_eq!(h1, h2);
    assert_eq!(h1.len(), 64); // hex blake3 output
}

// ---- Batch transmission deduplication --------------------------------------

#[test]
fn batch_transmission_dedupes_identical_forms() {
    let station = Uuid::now_v7();
    let form = Form34ADiaspora::new(
        station,
        "FORM-001",
        HashMap::from([("A".to_owned(), 10u64)]),
        0,
        0,
    );
    let (batch, deduped) = batch_transmit(vec![form.clone(), form.clone()], Utc::now());
    assert_eq!(deduped.len(), 1);
    assert_eq!(batch.form_ids.len(), 1);
}

#[test]
fn batch_keeps_distinct_forms() {
    let station = Uuid::now_v7();
    let f1 = Form34ADiaspora::new(station, "F1", HashMap::new(), 0, 0);
    let f2 = Form34ADiaspora::new(station, "F2", HashMap::new(), 0, 0);
    let (batch, deduped) = batch_transmit(vec![f1, f2], Utc::now());
    assert_eq!(deduped.len(), 2);
    assert_eq!(batch.form_ids.len(), 2);
}

// ---- Custody transfer chain ordering ---------------------------------------

#[test]
fn custody_chain_preserves_insertion_order() {
    let mut pouch = PouchCustody::new(vec![]);
    let now = Utc::now();

    for i in 0..5u64 {
        let event = CustodyEvent {
            transferred_at: now,
            from_person: format!("officer-{i}"),
            to_person: format!("officer-{}", i + 1),
            location: "Airport".to_owned(),
            gps_lat: -1.28,
            gps_lon: 36.82,
            signature: "sig".to_owned(),
        };
        pouch.transfer(event);
    }

    assert_eq!(pouch.custody_transfers.len(), 5);
    for (i, event) in pouch.custody_transfers.iter().enumerate() {
        assert_eq!(event.from_person, format!("officer-{i}"));
    }
}

// ---- CSV import: valid rows and rejection report ---------------------------

const VALID_CSV: &str = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,1234,Jane Doe,1980,US-DC-001
V002,5678,John Smith,1975,UK-LON-001
V003,9012,Ali Hassan,1990,AE-DXB-001";

#[test]
fn import_parses_valid_rows() {
    let report = import_roll_csv(ImportProfile::Washington, VALID_CSV);
    assert_eq!(report.accepted, 3);
    assert_eq!(report.rejected, 0);
    assert!(report.rows.iter().all(|r| r.ok));
}

#[test]
fn import_rejects_invalid_national_id() {
    let csv = "id,national_id_last4,name_nfc,dob_yyyy,station_code
BAD,AB12,Jane Doe,1980,US-DC-001";
    let report = import_roll_csv(ImportProfile::Washington, csv);
    assert_eq!(report.rejected, 1);
    assert!(!report.rows[0].ok);
    assert!(report.rows[0].error.is_some());
}

#[test]
fn import_rejects_out_of_range_dob() {
    let csv = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,1234,Jane Doe,1850,US-DC-001";
    let report = import_roll_csv(ImportProfile::Washington, csv);
    assert_eq!(report.rejected, 1);
}

#[test]
fn import_mixed_valid_invalid_rows() {
    let csv = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,1234,Jane Doe,1980,US-DC-001
BAD,,Jane,abc,
V003,9012,Ali Hassan,1990,AE-DXB-001";
    let report = import_roll_csv(ImportProfile::Dubai, csv);
    assert_eq!(report.accepted, 2);
    assert_eq!(report.rejected, 1);
}

// ---- Form 34A signature emit -----------------------------------------------

#[test]
fn form34a_stores_hsm_signature() {
    let mut form = Form34ADiaspora::new(
        Uuid::now_v7(),
        "FORM-SIG-001",
        HashMap::from([("candidate-A".to_owned(), 120u64)]),
        5,
        2,
    );
    form.hsm_signature = Some("base64-stub==".to_owned());
    assert_eq!(form.hsm_signature.as_deref(), Some("base64-stub=="));
}

#[test]
fn form34a_new_has_no_transmission() {
    let form = Form34ADiaspora::new(Uuid::now_v7(), "FORM-001", HashMap::new(), 0, 0);
    assert!(form.transmitted_at.is_none());
    assert!(form.transmission_batch_id.is_none());
    assert!(form.hsm_signature.is_none());
}

// ---- Station open/close lifecycle ------------------------------------------

#[test]
fn station_starts_closed() {
    let s = EmbassyStation::new(
        "US",
        "Washington",
        "America/New_York",
        "123 K St",
        "US-DC-001",
        2,
    );
    assert!(!s.is_open());
}

#[test]
fn station_is_open_after_opened_at_set() {
    let mut s = EmbassyStation::new(
        "GB",
        "London",
        "Europe/London",
        "1 Kings Rd",
        "UK-LON-001",
        1,
    );
    s.opened_at = Some(Utc::now());
    assert!(s.is_open());
}

// ---- Async batch delivery tests --------------------------------------------

#[tokio::test]
async fn successful_delivery_marks_batch_sent() {
    let station = Uuid::now_v7();
    let form = Form34ADiaspora::new(
        station,
        "FORM-ASYNC-001",
        HashMap::from([("X".to_owned(), 5u64)]),
        0,
        0,
    );
    let (batch, forms) = batch_transmit(vec![form], Utc::now());
    let (updated, envelopes, result) =
        attempt_delivery(batch, &forms, |_| async { Ok(()) }, Utc::now()).await;
    assert!(result.is_ok());
    assert_eq!(updated.status, BatchStatus::Sent);
    assert_eq!(envelopes.len(), 1);
}
