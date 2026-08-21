// Fixture: should trigger todo_macro and unimplemented_macro findings.
fn not_ready() {
    todo!("implement me")
}

fn also_not_ready() {
    unimplemented!()
}

#[ignore]
#[test]
fn skipped_test() {
    assert_eq!(1, 1);
}

fn handler() -> Result<(), String> {
    // TODO: wire up the real handler
    Err("unimplemented".into())
}
