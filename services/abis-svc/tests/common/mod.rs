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

//! Shared test rig for abis-svc integration tests.
//!
//! Tests in this directory are feature gated behind `cross-crate-integration`
//! and target a deployed abis-api backed by PostgreSQL and Kafka (the
//! docker-compose stack, or testcontainers started by the operator). There is
//! no silent skip: without the feature the tests are ignored; with the feature
//! and without the stack they fail loudly on connection errors.
//!
//! Tests requiring real Docker containers (testcontainers PG) are gated by
//! `cross-crate-integration`.

#![allow(dead_code)]


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
