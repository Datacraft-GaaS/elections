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

use abis_standards_interpol::{
    entity::{link_crime_to_person, CrimeEntity, PersonEntity},
    envelope::{InterpolEnvelope, MessagePurpose, NcbCode, ResponseWindow},
    InterpolError,
};

fn ken_ncb() -> NcbCode {
    NcbCode::new("KEN").unwrap()
}

fn int_ncb() -> NcbCode {
    NcbCode::new("INT").unwrap()
}

#[test]
fn envelope_build_and_round_trip() {
    let env = InterpolEnvelope::build(
        ken_ncb(),
        int_ncb(),
        MessagePurpose::CriminalSearch,
        ResponseWindow::Standard,
    )
    .unwrap();
    let serialized = env.serialize();
    let parsed = InterpolEnvelope::parse(&serialized).unwrap();
    assert_eq!(parsed.ncb_sender.as_str(), "KEN");
    assert_eq!(parsed.ncb_receiver.as_str(), "INT");
    assert_eq!(parsed.purpose, MessagePurpose::CriminalSearch);
}

#[test]
fn ncb_code_rejects_too_long() {
    let result = NcbCode::new("KENY");
    assert!(matches!(result, Err(InterpolError::InvalidNcbCode(_))));
}

#[test]
fn ncb_code_rejects_numeric() {
    let result = NcbCode::new("12");
    assert!(matches!(result, Err(InterpolError::InvalidNcbCode(_))));
}

#[test]
fn add_record_sets_idc() {
    let mut env = InterpolEnvelope::build(
        ken_ncb(),
        int_ncb(),
        MessagePurpose::PersonSearch,
        ResponseWindow::Immediate,
    )
    .unwrap();
    env.add_record(4, 7, vec![0xAA, 0xBB]);
    assert_eq!(env.records[0].idc, 7);
    assert_eq!(env.records[0].record_type, 4);
}

#[test]
fn person_entity_creation_and_linkage() {
    let crime = CrimeEntity {
        reference_number: "CR-2024-001".into(),
        crime_type: "Fraud".into(),
        date_occurred: "2024-01-15".into(),
        ncb_origin: ken_ncb(),
    };
    let person = PersonEntity {
        family_name: "DOE".into(),
        given_name: "JOHN".into(),
        birth_date: "19800101".into(),
        nationality: "KEN".into(),
        ncb_origin: ken_ncb(),
    };
    let linked = link_crime_to_person(&crime, &person);
    assert_eq!(linked.crime.reference_number, "CR-2024-001");
    assert_eq!(linked.person.family_name, "DOE");
}

#[test]
fn message_purpose_preserved_through_round_trip() {
    for purpose in [
        MessagePurpose::CriminalSearch,
        MessagePurpose::PersonSearch,
        MessagePurpose::LostDocumentSearch,
        MessagePurpose::StolenVehicleSearch,
    ] {
        let env = InterpolEnvelope::build(
            ken_ncb(),
            int_ncb(),
            purpose.clone(),
            ResponseWindow::Standard,
        )
        .unwrap();
        let parsed = InterpolEnvelope::parse(&env.serialize()).unwrap();
        assert_eq!(parsed.purpose, purpose);
    }
}

#[test]
fn response_window_variants() {
    assert_eq!(ResponseWindow::Immediate.hours(), 0);
    assert_eq!(ResponseWindow::Standard.hours(), 24);
    assert_eq!(ResponseWindow::Relaxed.hours(), 72);
}

#[test]
fn response_window_in_serialized_output() {
    let env = InterpolEnvelope::build(
        ken_ncb(),
        int_ncb(),
        MessagePurpose::CriminalSearch,
        ResponseWindow::Relaxed,
    )
    .unwrap();
    let bytes = env.serialize();
    let parsed = InterpolEnvelope::parse(&bytes).unwrap();
    assert_eq!(parsed.response_window, ResponseWindow::Relaxed);
}

#[test]
fn envelope_with_three_records_parses_all() {
    let mut env = InterpolEnvelope::build(
        ken_ncb(),
        int_ncb(),
        MessagePurpose::StolenVehicleSearch,
        ResponseWindow::Standard,
    )
    .unwrap();
    env.add_record(1, 1, vec![0x01]);
    env.add_record(2, 2, vec![0x02]);
    env.add_record(4, 3, vec![0x03]);
    let parsed = InterpolEnvelope::parse(&env.serialize()).unwrap();
    assert_eq!(parsed.records.len(), 3);
    assert_eq!(parsed.records[0].idc, 1);
    assert_eq!(parsed.records[1].idc, 2);
    assert_eq!(parsed.records[2].idc, 3);
}
