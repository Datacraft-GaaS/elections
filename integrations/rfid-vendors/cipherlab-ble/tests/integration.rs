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

use eel_rfid_cipherlab_ble::double::MockCipherLabBle;

#[test]
fn mock_register_ok() {
    let mock = MockCipherLabBle::new();
    let result = mock.register("cl-ble-handheld-001");
    assert_eq!(result["reader_id"], "cl-ble-handheld-001");
    assert_eq!(result["status"], "registered");
}

#[test]
fn mock_read_normal_ready() {
    let mock = MockCipherLabBle::new();
    let result = mock.read("epc-cl-ble-0001", false).unwrap();
    assert_eq!(result["tag_state"], "ready");
    assert_eq!(result["tamper_detected"], false);
}

#[test]
fn mock_read_tamper_override() {
    let mock = MockCipherLabBle::new();
    let result = mock.read("epc-cl-ble-0002", true).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_tamper_prob_always() {
    let mock = MockCipherLabBle::new().with_tamper_prob(1.0);
    let result = mock.read("epc-chaos-cl-ble", false).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_challenge_returns_fields() {
    let mock = MockCipherLabBle::new();
    let result = mock.challenge("cafebabe-cl-ble").unwrap();
    assert_eq!(result["nonce_hex"].as_str().unwrap().len(), 32);
    assert_eq!(result["challenge_hex"].as_str().unwrap().len(), 64);
}

#[test]
fn mock_battery_low_reported_in_status() {
    let mock = MockCipherLabBle::new().with_battery_low();
    let status = mock.status("cl-ble-001");
    assert_eq!(status["battery_ok"], false);
    assert_eq!(status["healthy"], false);
}
