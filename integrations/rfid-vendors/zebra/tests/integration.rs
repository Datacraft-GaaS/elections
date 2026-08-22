use eel_rfid_zebra::double::MockZebra;

#[test]
fn mock_register_ok() {
    let mock = MockZebra::new();
    let result = mock.register("zebra-001");
    assert_eq!(result["reader_id"], "zebra-001");
    assert_eq!(result["status"], "registered");
}

#[test]
fn mock_read_normal_ready() {
    let mock = MockZebra::new();
    let result = mock.read("epc-zebra-0001", false).unwrap();
    assert_eq!(result["tag_state"], "ready");
    assert_eq!(result["tamper_detected"], false);
}

#[test]
fn mock_read_tamper_override() {
    let mock = MockZebra::new();
    let result = mock.read("epc-zebra-0002", true).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_tamper_prob_always() {
    let mock = MockZebra::new().with_tamper_prob(1.0);
    let result = mock.read("epc-chaos-zebra", false).unwrap();
    assert_eq!(result["tamper_detected"], true);
}

#[test]
fn mock_challenge_returns_fields() {
    let mock = MockZebra::new();
    let result = mock.challenge("cafebabe-zebra").unwrap();
    assert!(result["nonce_hex"].as_str().unwrap().len() == 32);
    assert!(result["challenge_hex"].as_str().unwrap().len() == 64);
}
