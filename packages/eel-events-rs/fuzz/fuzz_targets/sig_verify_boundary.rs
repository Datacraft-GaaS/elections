#![no_main]
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

use arbitrary::Arbitrary;
use eel_events_rs::test_utils::EphemeralKeyProvider;
use eel_events_rs::{consumer::verify_signature, envelope::DataClass, Envelope};
use libfuzzer_sys::fuzz_target;

#[derive(Arbitrary, Debug)]
struct FuzzInput {
    sig_prefix: Vec<u8>,
    sig_body: Vec<u8>,
}

fuzz_target!(|input: FuzzInput| {
    let provider = EphemeralKeyProvider::new();
    // Build a minimal valid envelope then inject a fuzzed signature string.
    let mut env = Envelope {
        specversion: "1.0".into(),
        id: "01950f8c-0000-7000-0000-000000000001".into(),
        event_type: "eel.test.fuzz.v1".into(),
        source: "spiffe://eel.local/fuzzer".into(),
        subject: None,
        time: chrono::Utc::now(),
        datacontenttype: "application/json".into(),
        dataschema: None,
        data: serde_json::Value::Null,
        eel_stream_id: "01950f8c-0000-7000-0000-000000000001".into(),
        eel_seq: 0,
        eel_hlc: "0.0.0".into(),
        eel_prev_hash: None,
        eel_correlation_id: None,
        eel_causation_id: None,
        eel_producer_key_id: None,
        eel_signature: Some(format!(
            "{}:{}",
            String::from_utf8_lossy(&input.sig_prefix),
            String::from_utf8_lossy(&input.sig_body)
        )),
        eel_election_id: None,
        eel_data_class: DataClass::E,
    };
    let _ = verify_signature(&env, &provider);
});
