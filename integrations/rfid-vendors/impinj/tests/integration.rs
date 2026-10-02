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

use eel_rfid_impinj::double::MockImpinj;

#[test]
fn mock_register_returns_reader_id() {
    let mock = MockImpinj::new();
    let result = mock.register("reader-001");
    assert_eq!(result["reader_id"], "reader-001");
    assert_eq!(result["status"], "registered");
}

#[test]
fn mock_read_normal_tag_state_ready() {
    let mock = MockImpinj::new();
    mock.register("reader-001");
    let result = mock.read("deadbeef00000001", false).unwrap();
    assert_eq!(result["accepted"], true);
    assert_eq!(result["tag_state"], "ready");
    assert_eq!(result["tamper_detected"], false);
}

#[test]
fn mock_read_tamper_indicator_true() {
    let mock = MockImpinj::new();
    let result = mock.read("deadbeef00000002", true).unwrap();
    assert_eq!(result["tag_state"], "tampered");
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_tamper_prob_always_injects_tamper() {
    let mock = MockImpinj::new().with_tamper_prob(1.0);
    let result = mock.read("epc-chaos-001", false).unwrap();
    assert_eq!(result["tamper_detected"], true, "tamper_prob=1.0 must always report tamper");
}

#[tokio::test]
async fn mock_challenge_returns_nonce_and_challenge() {
    let mock = MockImpinj::new();
    let result = mock.challenge("cafebabe00000001").unwrap();
    let nonce = result["nonce_hex"].as_str().unwrap();
    let challenge = result["challenge_hex"].as_str().unwrap();
    assert_eq!(nonce.len(), 32, "nonce_hex must be 16 bytes as hex");
    assert_eq!(challenge.len(), 64, "challenge_hex must be 32 bytes as hex");
}
