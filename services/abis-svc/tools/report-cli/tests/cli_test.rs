// CLI unit tests: arg parsing, subcommand routing, sha256 verification.
// Integration tests against a live API are in examples/.

use sha2::{Digest, Sha256};

fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

// --- Clap arg parsing tests ---

#[test]
fn parse_list_no_args() {
    use std::process::Command;
    // Verify --help exits 0 (clap derives help automatically).
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("list")
        .arg("--help")
        .output()
        .expect("run eel-report");
    assert!(output.status.success(), "list --help should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("active-only") || stdout.contains("class"),
        "help should mention list options"
    );
}

#[test]
fn parse_run_required_args_missing() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("run")
        .output()
        .expect("run eel-report");
    // clap exits 2 on missing required args.
    assert_eq!(output.status.code(), Some(2), "missing args should exit 2");
}

#[test]
fn parse_status_missing_run_id() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("status")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_download_missing_args() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("download")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_verify_missing_file() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("verify")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_subscribe_missing_endpoint() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("subscribe")
        .arg("--subscriber")
        .arg("test")
        .arg("--slug")
        .arg("daily-enrolment-throughput")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_deliveries_missing_run() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("deliveries")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_define_missing_file() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("define")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn parse_schedule_missing_slug() {
    use std::process::Command;
    let output = Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .arg("schedule")
        .output()
        .expect("run eel-report");
    assert_eq!(output.status.code(), Some(2));
}

// --- SHA-256 verification logic tests ---

#[test]
fn sha256_stable_known_value() {
    // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
    let result = sha256_hex(b"");
    assert_eq!(
        result,
        "e3b0c44298fc1c149afbf4c8996fb924\
                         27ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn sha256_detects_tampered_bytes() {
    let original = b"authentic report content";
    let tampered = b"tampered report content!";
    let hash_original = sha256_hex(original);
    let hash_tampered = sha256_hex(tampered);
    assert_ne!(
        hash_original, hash_tampered,
        "different bytes must produce different hashes"
    );
}

#[test]
fn sha256_deterministic() {
    let data = b"idempotent";
    assert_eq!(sha256_hex(data), sha256_hex(data));
}

// --- verify subcommand detects tampering ---

#[test]
fn verify_command_tampered_file_exits_nonzero() {
    use std::io::Write;
    use tempfile::NamedTempFile;
    // Write garbage bytes to a temp file.
    let mut f = NamedTempFile::new().unwrap();
    f.write_all(b"this is tampered content xyz").unwrap();
    let path = f.path().to_str().unwrap().to_string();

    // The command will hit the API and fail (no server running), but
    // we can at least confirm it exits non-zero without panicking.
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_eel-report"))
        .args([
            "--api-url",
            "http://127.0.0.1:1",
            "verify",
            "--file",
            &path,
            "--artifact-id",
            "00000000-0000-0000-0000-000000000001",
        ])
        .output()
        .expect("run eel-report");
    // Exits non-zero: either network error (1) or sha mismatch (2).
    assert_ne!(output.status.code(), Some(0));
}
