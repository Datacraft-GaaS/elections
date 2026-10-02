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

use eel_crypto_rs::ed25519::{generate_keypair, Ed25519SigningKey};
use eel_events_rs::{
    consumer::verify_signature, envelope::DataClass, EventBuilder, SpiffeKeyProvider,
};

// Minimal in-test key provider.
struct TestProvider {
    key: Ed25519SigningKey,
}

impl TestProvider {
    fn new() -> Self {
        Self {
            key: generate_keypair(),
        }
    }
}

impl SpiffeKeyProvider for TestProvider {
    fn signing_key(&self) -> &Ed25519SigningKey {
        &self.key
    }

    fn key_id(&self) -> &str {
        "test-key-1"
    }
}

#[test]
fn builder_produces_valid_envelope() {
    let provider = TestProvider::new();
    let env = EventBuilder::new()
        .event_type("eel.rts.form34a_captured.v1")
        .source("spiffe://eel.local/service/rts-backend")
        .stream_id("01950f8c-0000-7000-0000-000000000001")
        .seq(0)
        .hlc("1723731000123.0.1")
        .data(serde_json::json!({"station_id": "47-001"}))
        .data_class(DataClass::B)
        .build(&provider)
        .unwrap();

    assert_eq!(env.specversion, "1.0");
    assert_eq!(env.event_type, "eel.rts.form34a_captured.v1");
    assert!(env.eel_signature.is_some());
    assert_eq!(env.eel_data_class, DataClass::B);
}

#[test]
fn envelope_json_roundtrip() {
    let provider = TestProvider::new();
    let env = EventBuilder::new()
        .event_type("eel.custody.kit_dispatched.v1")
        .source("spiffe://eel.local/service/custody")
        .stream_id("01950f8c-0000-7000-0000-000000000002")
        .seq(1)
        .hlc("1723731000200.0.1")
        .data_class(DataClass::C)
        .build(&provider)
        .unwrap();

    let json = serde_json::to_string(&env).unwrap();
    let decoded: eel_events_rs::Envelope = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded.id, env.id);
    assert_eq!(decoded.event_type, env.event_type);
    assert_eq!(decoded.eel_stream_id, env.eel_stream_id);
}

#[test]
fn signature_verifies_correctly() {
    let provider = TestProvider::new();
    let env = EventBuilder::new()
        .event_type("eel.rts.tally_published.v1")
        .source("spiffe://eel.local/service/rts")
        .stream_id("01950f8c-0000-7000-0000-000000000003")
        .seq(0)
        .hlc("1723731000300.0.1")
        .build(&provider)
        .unwrap();

    assert!(verify_signature(&env, &provider).is_ok());
}

#[test]
fn tampered_envelope_fails_verification() {
    let provider = TestProvider::new();
    let mut env = EventBuilder::new()
        .event_type("eel.rts.tally_published.v1")
        .source("spiffe://eel.local/service/rts")
        .stream_id("01950f8c-0000-7000-0000-000000000004")
        .seq(0)
        .hlc("1723731000400.0.1")
        .build(&provider)
        .unwrap();

    // Tamper with a field after signing.
    env.eel_seq = 999;

    assert!(verify_signature(&env, &provider).is_err());
}

#[test]
fn gap_buffer_orders_events() {
    use eel_events_rs::{Envelope, GapBuffer};

    let make_env = |seq: u64| Envelope {
        specversion: "1.0".into(),
        id: format!("id-{}", seq),
        event_type: "eel.test.v1".into(),
        source: "src".into(),
        subject: None,
        time: chrono::Utc::now(),
        datacontenttype: "application/json".into(),
        dataschema: None,
        data: serde_json::Value::Null,
        eel_stream_id: "stream-1".into(),
        eel_seq: seq,
        eel_hlc: format!("{}.0.1", seq),
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: None,
        eel_election_id: None,
        eel_data_class: DataClass::E,
    };

    let mut buf = GapBuffer::new();

    let r0 = buf.insert(make_env(0));
    let r2 = buf.insert(make_env(2));
    let r1 = buf.insert(make_env(1));

    assert_eq!(r0.len(), 1);
    assert_eq!(r0[0].eel_seq, 0);
    assert_eq!(r2.len(), 0); // Buffered.
    assert_eq!(r1.len(), 2); // Fills gap: delivers 1 then 2.
    assert_eq!(r1[0].eel_seq, 1);
    assert_eq!(r1[1].eel_seq, 2);
}

#[test]
fn dlq_and_tamper_topic_names() {
    use eel_events_rs::{dlq_topic, tamper_topic};
    assert_eq!(dlq_topic("eel.rts.form34a.v1"), "eel.rts.form34a.v1.dlq");
    assert_eq!(
        tamper_topic("eel.rts.form34a.v1"),
        "eel.rts.form34a.v1.tamper"
    );
}
