// This file is included as a module from src/double.rs.
// Direct path: integrations/rfid-vendors/impinj/double/mock_impinj.rs

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::{json, Value};
use tokio::time::sleep;

/// In-memory Impinj double for dev and chaos testing.
///
/// Knobs:
/// - `tamper_prob`: fraction [0.0, 1.0] of reads that simulate cover-lift tamper.
/// - `read_interval_ms`: synthetic read interval (used by `emit_reads`).
/// - `error_rate`: fraction [0.0, 1.0] of requests returning a simulated error.
pub struct MockImpinj {
    pub tamper_prob: f64,
    pub read_interval_ms: u64,
    pub error_rate: f64,
    read_count: Arc<AtomicU64>,
    registered_readers: Arc<Mutex<Vec<String>>>,
}

impl MockImpinj {
    pub fn new() -> Self {
        Self {
            tamper_prob: 0.0,
            read_interval_ms: 100,
            error_rate: 0.0,
            read_count: Arc::new(AtomicU64::new(0)),
            registered_readers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Inject tamper with the given probability.
    pub fn with_tamper_prob(mut self, prob: f64) -> Self {
        assert!((0.0..=1.0).contains(&prob), "tamper_prob must be in [0, 1]");
        self.tamper_prob = prob;
        self
    }

    /// Set synthetic error rate for chaos testing.
    pub fn with_error_rate(mut self, rate: f64) -> Self {
        assert!((0.0..=1.0).contains(&rate), "error_rate must be in [0, 1]");
        self.error_rate = rate;
        self
    }

    /// Simulate reader registration.
    pub fn register(&self, reader_id: &str) -> Value {
        let mut readers = self.registered_readers.lock().unwrap();
        readers.push(reader_id.to_string());
        json!({ "reader_id": reader_id, "status": "registered" })
    }

    /// Simulate a single tag read; respects tamper_prob and error_rate.
    pub fn read(&self, tag_epc: &str, tamper_override: bool) -> Option<Value> {
        if self.should_error() {
            return None; // simulate 503
        }
        let tamper = tamper_override || self.should_tamper();
        self.read_count.fetch_add(1, Ordering::Relaxed);
        let tag_state = if tamper { "tampered" } else { "ready" };
        Some(json!({
            "tag_epc": tag_epc,
            "accepted": true,
            "tag_state": tag_state,
            "tamper_detected": tamper,
        }))
    }

    /// Simulate issuing a Gen2v2 challenge; returns a synthetic nonce + challenge pair.
    pub fn challenge(&self, tag_epc: &str) -> Option<Value> {
        if self.should_error() {
            return None;
        }
        let nonce_hex = "00".repeat(16);
        let challenge_hex = "aa".repeat(32);
        Some(json!({
            "tag_epc": tag_epc,
            "nonce_hex": nonce_hex,
            "challenge_hex": challenge_hex,
        }))
    }

    /// Return simulated reader status.
    pub fn status(&self, reader_id: &str) -> Value {
        let count = self.read_count.load(Ordering::Relaxed);
        json!({
            "reader_id": reader_id,
            "healthy": true,
            "tag_count": count,
            "adapter_type": "impinj",
        })
    }

    /// Emit `n` synthetic reads at the configured interval.
    pub async fn emit_reads(&self, tag_epcs: &[&str], n: u64) -> Vec<Value> {
        let mut results = Vec::new();
        for i in 0..n {
            let epc = tag_epcs[i as usize % tag_epcs.len()];
            if let Some(r) = self.read(epc, false) {
                results.push(r);
            }
            if self.read_interval_ms > 0 {
                sleep(Duration::from_millis(self.read_interval_ms)).await;
            }
        }
        results
    }

    fn should_tamper(&self) -> bool {
        if self.tamper_prob <= 0.0 {
            return false;
        }
        if self.tamper_prob >= 1.0 {
            return true;
        }
        // Deterministic for the first read: use read_count parity to avoid rand dep.
        let count = self.read_count.load(Ordering::Relaxed);
        (count % 100) < (self.tamper_prob * 100.0) as u64
    }

    fn should_error(&self) -> bool {
        if self.error_rate <= 0.0 {
            return false;
        }
        if self.error_rate >= 1.0 {
            return true;
        }
        let count = self.read_count.load(Ordering::Relaxed);
        (count % 100) < (self.error_rate * 100.0) as u64
    }
}

impl Default for MockImpinj {
    fn default() -> Self {
        Self::new()
    }
}
