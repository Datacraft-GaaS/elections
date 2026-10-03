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

//! Peer gossip: poll configured witness peers' heads and feed every
//! observation through the same split-view detector as local cosigns.
//!
//! `WITNESS_PEERS` is a comma-separated list of peer base URLs exposing
//! `GET /api/v1/transparency/witnesses/head`.

use std::sync::Arc;

use tracing::{info, warn};

/// Extract `(tree_size, root_hash)` from a peer head response.
///
/// Returns `None` for malformed payloads (missing fields, wrong types) so
/// callers can skip bad peers without panicking. Pure function — testable
/// offline against arbitrary JSON.
pub fn parse_peer_head(v: &serde_json::Value) -> Option<(u64, String)> {
    let tree_size = v["tree_size"].as_u64()?;
    let root = v["root_hash"].as_str().map(str::to_string)?;
    Some((tree_size, root))
}

/// One gossip round over all peers. Returns the number of observations fed
/// to the detector.
pub async fn poll_peers_once(
    peers: &[String],
    detector: &Arc<std::sync::Mutex<super::split_view::SplitViewDetector>>,
    client: &reqwest::Client,
    self_name: &str,
) -> usize {
    let mut observed = 0usize;
    for peer in peers {
        let url = format!(
            "{}/api/v1/transparency/witnesses/head",
            peer.trim_end_matches('/')
        );
        match client.get(&url).send().await {
            Ok(resp) => match resp.json::<serde_json::Value>().await {
                Ok(v) => match parse_peer_head(&v) {
                    Some((ts, root)) => {
                        let mut d = match detector.lock() {
                            Ok(d) => d,
                            Err(_) => return observed,
                        };
                        let split = d.observe_root(&format!("peer:{peer}"), ts, &root);
                        if split {
                            // Findings are retained on the detector and served
                            // via GET .../split-views; the error log above is
                            // emitted by observe_root itself.
                            warn!(peer = %peer, tree_size = ts, "gossip observed conflicting head");
                        }
                        observed += 1;
                    }
                    None => warn!(peer = %peer, "peer head missing/malformed fields; skipped"),
                },
                Err(e) => warn!(peer = %peer, error = %e, "peer head decode failed"),
            },
            Err(e) => warn!(peer = %peer, error = %e, "peer unreachable"),
        }
    }
    info!(self = %self_name, observations = observed, "gossip round complete");
    observed
}

/// Spawn the periodic gossip loop.
pub fn spawn_gossip(
    peers: Vec<String>,
    detector: Arc<std::sync::Mutex<crate::split_view::SplitViewDetector>>,
    interval_secs: u64,
) {
    if peers.is_empty() {
        info!("WITNESS_PEERS empty; gossip disabled");
        return;
    }
    tokio::spawn(async move {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .expect("http client");
        let mut ticker =
            tokio::time::interval(std::time::Duration::from_secs(interval_secs.max(1)));
        loop {
            ticker.tick().await;
            poll_peers_once(&peers, &detector, &client, "tlog-witness").await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_head() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"tree_size": 12, "root_hash": "abc123"}"#).unwrap();
        assert_eq!(parse_peer_head(&v), Some((12u64, "abc123".to_string())));
    }

    #[test]
    fn missing_fields_are_skipped() {
        assert!(parse_peer_head(&serde_json::json!({})).is_none());
        assert!(parse_peer_head(&serde_json::json!({"tree_size": 1})).is_none());
        assert!(parse_peer_head(&serde_json::json!({"root_hash": "abc"})).is_none());
    }

    #[test]
    fn wrong_types_are_skipped_without_panic() {
        assert!(
            parse_peer_head(&serde_json::json!({"tree_size": "big", "root_hash": "x"})).is_none()
        );
        assert!(parse_peer_head(&serde_json::json!({"tree_size": -1, "root_hash": "x"})).is_none());
        assert!(
            parse_peer_head(&serde_json::json!({"tree_size": 1.5, "root_hash": "x"})).is_none()
        );
        assert!(parse_peer_head(&serde_json::json!({"tree_size": 1, "root_hash": 42})).is_none());
        assert!(
            parse_peer_head(&serde_json::json!({"tree_size": null, "root_hash": null})).is_none()
        );
        // Entirely wrong JSON shape (array / scalar) must not panic.
        assert!(parse_peer_head(&serde_json::json!([1, 2, 3])).is_none());
        assert!(parse_peer_head(&serde_json::json!("hello")).is_none());
    }

    #[tokio::test]
    async fn poll_round_with_unreachable_malformed_peers_does_not_panic() {
        let detector = Arc::new(std::sync::Mutex::new(
            crate::split_view::SplitViewDetector::new(),
        ));
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()
            .unwrap();
        let peers = vec![
            "http://127.0.0.1:1".to_string(), // connection refused fast
        ];
        let observed = poll_peers_once(&peers, &detector, &client, "test").await;
        assert_eq!(observed, 0);
    }
}
