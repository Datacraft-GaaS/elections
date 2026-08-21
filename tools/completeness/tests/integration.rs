use eel_completeness_lib::{allowlist, scan};
use std::fs;
use tempfile::TempDir;

fn scratch() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn mk(dir: &TempDir, rel: &str, body: &str) {
    let p = dir.path().join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, body).unwrap();
}

fn kinds(dir: &TempDir) -> Vec<String> {
    let al = allowlist::load(&dir.path().join(".completeness-allowlist.yaml")).unwrap();
    let mut findings = scan::run(dir.path(), &al).unwrap();
    findings.sort_by(|a, b| a.kind.cmp(&b.kind));
    findings.iter().map(|f| f.kind.clone()).collect()
}

// ---- should trigger -------------------------------------------------------

#[test]
fn rust_todo_macro_triggers() {
    let d = scratch();
    mk(&d, "src/lib.rs", "fn stub() { todo!(\"not yet\") }\n");
    let ks = kinds(&d);
    assert!(ks.contains(&"todo_macro".to_owned()), "got: {ks:?}");
}

#[test]
fn rust_unimplemented_macro_triggers() {
    let d = scratch();
    mk(&d, "src/lib.rs", "fn stub() { unimplemented!() }\n");
    let ks = kinds(&d);
    assert!(
        ks.contains(&"unimplemented_macro".to_owned()),
        "got: {ks:?}"
    );
}

#[test]
fn rust_ignore_triggers() {
    let d = scratch();
    mk(&d, "src/lib.rs", "#[ignore]\n#[test]\nfn skipped() {}\n");
    let ks = kinds(&d);
    assert!(ks.contains(&"ignored_test".to_owned()), "got: {ks:?}");
}

#[test]
fn python_not_implemented_error_triggers() {
    let d = scratch();
    mk(
        &d,
        "svc/service.py",
        "def handle():\n    raise NotImplementedError\n",
    );
    let ks = kinds(&d);
    assert!(
        ks.contains(&"not_implemented_error".to_owned()),
        "got: {ks:?}"
    );
}

#[test]
fn python_pytest_skip_triggers() {
    let d = scratch();
    mk(
        &d,
        "tests/test_svc.py",
        "@pytest.mark.skip\ndef test_foo(): pass\n",
    );
    let ks = kinds(&d);
    assert!(ks.contains(&"pytest_skip".to_owned()), "got: {ks:?}");
}

#[test]
fn python_pass_body_triggers() {
    let d = scratch();
    mk(&d, "svc/x.py", "def foo():\n    pass\n");
    let ks = kinds(&d);
    assert!(ks.contains(&"pass_only_body".to_owned()), "got: {ks:?}");
}

#[test]
fn elixir_tag_skip_triggers() {
    let d = scratch();
    mk(
        &d,
        "lib/foo_test.exs",
        "@tag :skip\ntest \"skipped\" do\n  :ok\nend\n",
    );
    let ks = kinds(&d);
    assert!(ks.contains(&"exunit_tag_skip".to_owned()), "got: {ks:?}");
}

#[test]
fn js_skip_triggers() {
    let d = scratch();
    mk(&d, "src/foo.test.ts", "it.skip('not ready', () => {});\n");
    let ks = kinds(&d);
    assert!(ks.contains(&"js_skip".to_owned()), "got: {ks:?}");
}

#[test]
fn todo_comment_triggers_in_rust() {
    let d = scratch();
    mk(&d, "src/lib.rs", "// TODO: implement this\nfn foo() {}\n");
    let ks = kinds(&d);
    assert!(ks.contains(&"todo_comment".to_owned()), "got: {ks:?}");
}

#[test]
fn http_501_triggers() {
    let d = scratch();
    mk(
        &d,
        "src/handler.rs",
        "StatusCode::NOT_IMPLEMENTED // 501 stub\n",
    );
    let ks = kinds(&d);
    assert!(ks.contains(&"http_501".to_owned()), "got: {ks:?}");
}

// ---- should NOT trigger ---------------------------------------------------

#[test]
fn clean_rust_file_no_findings() {
    let d = scratch();
    mk(
        &d,
        "src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    let ks = kinds(&d);
    // no stub patterns
    assert!(
        !ks.contains(&"todo_macro".to_owned()) && !ks.contains(&"unimplemented_macro".to_owned()),
        "unexpected findings: {ks:?}"
    );
}

#[test]
fn clean_python_no_findings() {
    let d = scratch();
    mk(&d, "svc/handler.py", "def add(a, b):\n    return a + b\n");
    let ks = kinds(&d);
    assert!(
        !ks.contains(&"not_implemented_error".to_owned())
            && !ks.contains(&"pytest_skip".to_owned()),
        "unexpected findings: {ks:?}"
    );
}

#[test]
fn skip_dirs_excluded() {
    // A todo! inside target/ must not produce a finding.
    let d = scratch();
    mk(&d, "target/debug/build/foo.rs", "fn x() { todo!() }\n");
    let ks = kinds(&d);
    assert!(
        !ks.contains(&"todo_macro".to_owned()),
        "target/ should be excluded"
    );
}

// ---- allowlist suppression ------------------------------------------------

#[test]
fn allowlisted_finding_is_suppressed() {
    let d = scratch();
    mk(&d, "src/lib.rs", "fn stub() { todo!() }\n");

    // Finding ID format: file:line:kind
    // Line 1, so: src/lib.rs:1:todo_macro
    let al_content = "entries:\n  - id: \"src/lib.rs:1:todo_macro\"\n    reason: \"known stub\"\n";
    mk(&d, ".completeness-allowlist.yaml", al_content);

    let al = allowlist::load(&d.path().join(".completeness-allowlist.yaml")).unwrap();
    let findings = scan::run(d.path(), &al).unwrap();
    let todo_findings: Vec<_> = findings.iter().filter(|f| f.kind == "todo_macro").collect();
    assert!(!todo_findings.is_empty(), "should still appear in list");
    assert!(
        todo_findings.iter().all(|f| f.allowlisted),
        "should be marked allowlisted"
    );
}

#[test]
fn expired_allowlist_entry_not_active() {
    let d = scratch();
    mk(&d, "src/lib.rs", "fn stub() { todo!() }\n");
    let al_content =
        "entries:\n  - id: \"src/lib.rs:1:todo_macro\"\n    reason: \"old\"\n    expires: \"2020-01-01\"\n";
    mk(&d, ".completeness-allowlist.yaml", al_content);

    let al = allowlist::load(&d.path().join(".completeness-allowlist.yaml")).unwrap();
    let findings = scan::run(d.path(), &al).unwrap();
    let todo_findings: Vec<_> = findings.iter().filter(|f| f.kind == "todo_macro").collect();
    assert!(
        todo_findings.iter().all(|f| !f.allowlisted),
        "expired entry must not suppress finding"
    );
}
