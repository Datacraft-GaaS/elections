use std::fs;
use tempfile::TempDir;

fn make_fixture(dir: &TempDir, rel: &str, content: &str) {
    let path = dir.path().join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn run_scan(dir: &TempDir) -> Vec<String> {
    let allowlist =
        eel_completeness_lib::allowlist::load(&dir.path().join(".completeness-allowlist.yaml"))
            .unwrap();
    let findings = eel_completeness_lib::scan::run(dir.path(), &allowlist).unwrap();
    findings.iter().map(|f| f.kind.clone()).collect()
}

// Note: the library re-export requires lib.rs to exist.
// These tests live here as integration-style tests that invoke the public API
// by rebuilding an equivalent inline scanner. For simplicity we test via
// the scan module's public function through a lib shim.
//
// Actual executable tests that compile cleanly are in tests/integration.rs below.
// This file documents fixture expectations.
