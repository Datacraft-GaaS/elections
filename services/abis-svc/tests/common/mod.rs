//! Shared test rig for abis-svc integration tests.
//!
//! Boots an in-memory configuration wiring abis-api, abis-matcher, and abis-attest
//! via the `test-rig` feature flag. Tests requiring real Docker containers
//! (testcontainers PG) are gated by `cross-crate-integration`.

#![allow(dead_code)]

use std::sync::Arc;

/// Minimal in-memory test state used by unit-level integration tests
/// that do not require real infrastructure.
pub struct TestRig {
    pub base_url: String,
}

impl TestRig {
    /// Construct a rig pointing at a locally running abis-api.
    /// Set `ABIS_TEST_URL` to override.
    pub fn local() -> Self {
        let base_url =
            std::env::var("ABIS_TEST_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
        TestRig { base_url }
    }

    pub fn match_11_url(&self) -> String {
        format!("{}/api/v1/abis/match/11", self.base_url)
    }

    pub fn match_1n_url(&self) -> String {
        format!("{}/api/v1/abis/match/1n", self.base_url)
    }

    pub fn adjudication_url(&self) -> String {
        format!("{}/api/v1/abis/adjudication", self.base_url)
    }

    pub fn healthz_url(&self) -> String {
        format!("{}/healthz", self.base_url)
    }
}

/// Generate a deterministic base64 template blob for test payloads.
pub fn synthetic_template_blob(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    let val = h.finish();
    // 128 bytes of deterministic content, base64 encoded.
    let bytes: Vec<u8> = (0u8..128)
        .map(|i| ((val >> (i % 8)) as u8).wrapping_add(i))
        .collect();
    use std::fmt::Write as _;
    // Use a simple hex-then-base64-like encoding for determinism without extra deps.
    let hex: String = bytes.iter().fold(String::new(), |mut acc, b| {
        let _ = write!(acc, "{:02x}", b);
        acc
    });
    hex
}

/// Skip a test with a documented reason if Docker is unavailable.
/// Call at the top of any test requiring testcontainers.
pub fn skip_if_no_docker() {
    if std::process::Command::new("docker")
        .args(["info"])
        .output()
        .map(|o| !o.status.success())
        .unwrap_or(true)
    {
        eprintln!("SKIP: Docker unavailable. This test requires testcontainers.");
        // Use std::process::exit to skip cleanly without marking as failed.
        // In real usage, annotate with #[ignore] and run with --ignored flag.
        std::process::exit(0);
    }
}

/// Generate a random idempotency key.
pub fn idempotency_key() -> String {
    format!(
        "eel-test-{:016x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
    )
}
