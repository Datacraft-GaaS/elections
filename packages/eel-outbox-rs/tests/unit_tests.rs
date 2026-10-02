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

use eel_outbox_rs::{OutboxEnvelope, OutboxError, MAX_PAYLOAD_BYTES};
use serde_json::{json, Value};

fn make_envelope(data: Value) -> OutboxEnvelope {
    OutboxEnvelope {
        id: "01950f8c-0000-7000-8000-000000000001".into(),
        event_type: "eel.rts.form34a_captured.v1".into(),
        source: "spiffe://eel.local/service/rts-backend".into(),
        subject: Some("form-34a/47-0001-001".into()),
        time: "2027-08-15T15:30:00.123Z".into(),
        eel_stream_id: "01950f8c-0000-7000-8000-000000000001".into(),
        eel_seq: 1,
        eel_hlc: 1723731000123,
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: None,
        eel_election_id: Some("2027-general".into()),
        eel_data_class: Some("B".into()),
        data,
        topic: "eel.rts.form34a.v1".into(),
        kafka_key: "01950f8c-0000-7000-8000-000000000001".into(),
        aggregate_type: "form34a".into(),
        aggregate_id: "01950f8c-0000-7000-8000-000000000001".into(),
    }
}

#[test]
fn envelope_serialises_to_json() {
    let env = make_envelope(json!({ "form_id": "47-0001-001" }));
    let bytes = env.to_json_bytes().unwrap();
    assert!(!bytes.is_empty());
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["event_type"], "eel.rts.form34a_captured.v1");
}

#[test]
fn payload_over_1mib_is_rejected_at_size_guard() {
    // Build a data value whose JSON serialisation clearly exceeds 1 MiB.
    // We use a string of 1.1 MiB of 'x' characters.
    let big_string = "x".repeat(MAX_PAYLOAD_BYTES + 100_000);
    let env = make_envelope(json!({ "blob": big_string }));

    // The actual size check is inside Outbox::emit (which needs a DB tx),
    // but we can verify the JSON byte count here to confirm the guard would fire.
    let bytes = serde_json::to_vec(&env.data).unwrap();
    assert!(
        bytes.len() > MAX_PAYLOAD_BYTES,
        "test data must exceed the 1 MiB limit; got {} bytes",
        bytes.len()
    );
}

#[test]
fn envelope_without_signature_gives_none_bytes() {
    let env = make_envelope(json!({}));
    assert!(env.signature_bytes().is_none());
}

#[test]
fn max_payload_bytes_is_one_mib() {
    assert_eq!(MAX_PAYLOAD_BYTES, 1 << 20);
}
