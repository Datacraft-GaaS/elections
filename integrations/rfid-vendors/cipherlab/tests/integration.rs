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

use eel_rfid_cipherlab::double::MockCipherLab;

#[test]
fn mock_register_ok() {
    let mock = MockCipherLab::new();
    let result = mock.register("cl-handheld-001");
    assert_eq!(result["reader_id"], "cl-handheld-001");
    assert_eq!(result["status"], "registered");
}

#[test]
fn mock_read_normal_ready() {
    let mock = MockCipherLab::new();
    let result = mock.read("epc-cl-0001", false).unwrap();
    assert_eq!(result["tag_state"], "ready");
    assert_eq!(result["tamper_detected"], false);
}

#[test]
fn mock_read_tamper_override() {
    let mock = MockCipherLab::new();
    let result = mock.read("epc-cl-0002", true).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_tamper_prob_always() {
    let mock = MockCipherLab::new().with_tamper_prob(1.0);
    let result = mock.read("epc-chaos-cl", false).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_challenge_returns_fields() {
    let mock = MockCipherLab::new();
    let result = mock.challenge("cafebabe-cl").unwrap();
    assert_eq!(result["nonce_hex"].as_str().unwrap().len(), 32);
    assert_eq!(result["challenge_hex"].as_str().unwrap().len(), 64);
}
