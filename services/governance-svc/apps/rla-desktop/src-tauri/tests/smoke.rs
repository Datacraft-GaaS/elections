#[test]
fn error_serialises() {
    use rla_desktop_lib::error::RlaError;
    let e = RlaError::NotFound("x".to_string());
    let s = format!("{}", e);
    assert!(s.contains("x"));
}
