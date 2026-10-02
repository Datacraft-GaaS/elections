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

//! NIST beacon contract tests against the double.
//!
//! Set NIST_DOUBLE_URL env var to point at the double (default: http://localhost:8090).

use eel_adapter_nist_beacon_rs::{fetch_latest, PulseCache};

fn double_url() -> String {
    std::env::var("NIST_DOUBLE_URL").unwrap_or_else(|_| "http://localhost:8090".into())
}

#[tokio::test]
async fn fetch_latest_from_double() {
    let base = format!("{}/beacon/2.0", double_url());
    // Skip if double not running (CI without docker).
    if reqwest::Client::new()
        .get(format!("{}/health", double_url()))
        .send()
        .await
        .is_err()
    {
        return;
    }
    let pulse = fetch_latest(&base).await.expect("fetch_latest must succeed");
    assert_eq!(pulse.chain_index, 1);
    assert!(!pulse.output_value.is_empty());
}

#[test]
fn cache_holds_latest() {
    let cache = PulseCache::new(10);
    assert!(cache.latest().is_none());
}
