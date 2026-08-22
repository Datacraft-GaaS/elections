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
