// Direct path: integrations/rfid-vendors/cipherlab/double/mock_cipherlab.rs
// In-memory double for CipherLab RS36 handheld PDA + tamper injection.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

/// In-memory CipherLab RS36 double for handheld PDA scenario.
///
/// Simulates BLE + serial pairing model. Tamper injection knob for chaos scenarios.
///
/// Knobs:
/// - `tamper_prob`: fraction [0.0, 1.0] of reads that inject a tamper event.
/// - `error_rate`: fraction [0.0, 1.0] of requests returning a simulated error.
/// - `battery_low`: when true, status reports battery_ok=false.
pub struct MockCipherLab {
    pub tamper_prob: f64,
    pub error_rate: f64,
    pub battery_low: bool,
    read_count: Arc<AtomicU64>,
    registered_readers: Arc<Mutex<Vec<String>>>,
}

impl MockCipherLab {
    pub fn new() -> Self {
        Self {
            tamper_prob: 0.0,
            error_rate: 0.0,
            battery_low: false,
            read_count: Arc::new(AtomicU64::new(0)),
            registered_readers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_tamper_prob(mut self, prob: f64) -> Self {
        assert!((0.0..=1.0).contains(&prob));
        self.tamper_prob = prob;
        self
    }

    pub fn with_error_rate(mut self, rate: f64) -> Self {
        assert!((0.0..=1.0).contains(&rate));
        self.error_rate = rate;
        self
    }

    pub fn with_battery_low(mut self) -> Self {
        self.battery_low = true;
        self
    }

    pub fn register(&self, reader_id: &str) -> Value {
        self.registered_readers
            .lock()
            .unwrap()
            .push(reader_id.to_string());
        json!({ "reader_id": reader_id, "status": "registered" })
    }

    pub fn read(&self, tag_epc: &str, tamper_override: bool) -> Option<Value> {
        if self.should_error() {
            return None;
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

    pub fn challenge(&self, tag_epc: &str) -> Option<Value> {
        if self.should_error() {
            return None;
        }
        Some(json!({
            "tag_epc": tag_epc,
            "nonce_hex": "cc".repeat(16),
            "challenge_hex": "dd".repeat(32),
        }))
    }

    pub fn status(&self, reader_id: &str) -> Value {
        json!({
            "reader_id": reader_id,
            "healthy": !self.battery_low,
            "battery_ok": !self.battery_low,
            "tag_count": self.read_count.load(Ordering::Relaxed),
            "adapter_type": "cipherlab",
        })
    }

    fn should_tamper(&self) -> bool {
        if self.tamper_prob >= 1.0 {
            return true;
        }
        if self.tamper_prob <= 0.0 {
            return false;
        }
        let count = self.read_count.load(Ordering::Relaxed);
        (count % 100) < (self.tamper_prob * 100.0) as u64
    }

    fn should_error(&self) -> bool {
        if self.error_rate >= 1.0 {
            return true;
        }
        if self.error_rate <= 0.0 {
            return false;
        }
        let count = self.read_count.load(Ordering::Relaxed);
        (count % 100) < (self.error_rate * 100.0) as u64
    }
}

impl Default for MockCipherLab {
    fn default() -> Self {
        Self::new()
    }
}
