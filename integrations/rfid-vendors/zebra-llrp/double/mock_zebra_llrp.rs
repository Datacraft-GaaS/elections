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

// Direct path: integrations/rfid-vendors/zebra-llrp/double/mock_zebra_llrp.rs
// Included via #[path] from src/double.rs.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

/// In-memory Zebra FX9600 LLRP double for dev and chaos testing.
///
/// The reader is accessed via LLRP (open standard); no commercial SDK.
///
/// Knobs:
/// - `tamper_prob`: fraction [0.0, 1.0] of reads that simulate cover-lift tamper.
/// - `error_rate`: fraction [0.0, 1.0] of requests returning a simulated error.
pub struct MockZebraLlrp {
    pub tamper_prob: f64,
    pub error_rate: f64,
    read_count: Arc<AtomicU64>,
    registered_readers: Arc<Mutex<Vec<String>>>,
}

impl MockZebraLlrp {
    pub fn new() -> Self {
        Self {
            tamper_prob: 0.0,
            error_rate: 0.0,
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
            "nonce_hex": "00".repeat(16),
            "challenge_hex": "bb".repeat(32),
        }))
    }

    pub fn status(&self, reader_id: &str) -> Value {
        json!({
            "reader_id": reader_id,
            "healthy": true,
            "tag_count": self.read_count.load(Ordering::Relaxed),
            "adapter_type": "zebra",
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

impl Default for MockZebraLlrp {
    fn default() -> Self {
        Self::new()
    }
}
