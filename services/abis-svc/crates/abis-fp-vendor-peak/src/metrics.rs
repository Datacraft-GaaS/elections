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

//! Prometheus counters for peak-mode consensus.
//!
//! Counters:
//!   peak_consensus_agreements_total   - 3-way (or 2-of-2) consensus accepted.
//!   peak_consensus_disagreements_total - consensus rejected.
//!   peak_vendor_degraded_total{vendor} - per-vendor degradation events.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::RwLock;

static AGREEMENTS: AtomicU64 = AtomicU64::new(0);
static DISAGREEMENTS: AtomicU64 = AtomicU64::new(0);

// Simple per-vendor counter map (avoids pulling in the full prometheus crate).
static VENDOR_DEGRADED: std::sync::LazyLock<RwLock<HashMap<String, AtomicU64>>> =
    std::sync::LazyLock::new(|| RwLock::new(HashMap::new()));

pub fn inc_consensus_agreements() {
    AGREEMENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_consensus_disagreements() {
    DISAGREEMENTS.fetch_add(1, Ordering::Relaxed);
}

pub fn inc_vendor_degraded(vendor: &str) {
    // Try read-lock path first (fast).
    if let Ok(map) = VENDOR_DEGRADED.read() {
        if let Some(counter) = map.get(vendor) {
            counter.fetch_add(1, Ordering::Relaxed);
            return;
        }
    }
    // Insert on first sight.
    if let Ok(mut map) = VENDOR_DEGRADED.write() {
        map.entry(vendor.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .fetch_add(1, Ordering::Relaxed);
    }
}

/// Snapshot for health endpoints.
#[derive(Debug, Clone)]
pub struct MetricsSnapshot {
    pub agreements: u64,
    pub disagreements: u64,
    pub vendor_degraded: HashMap<String, u64>,
}

pub fn snapshot() -> MetricsSnapshot {
    let vendor_degraded = VENDOR_DEGRADED
        .read()
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.load(Ordering::Relaxed)))
                .collect()
        })
        .unwrap_or_default();
    MetricsSnapshot {
        agreements: AGREEMENTS.load(Ordering::Relaxed),
        disagreements: DISAGREEMENTS.load(Ordering::Relaxed),
        vendor_degraded,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_increment() {
        let before = AGREEMENTS.load(Ordering::Relaxed);
        inc_consensus_agreements();
        assert_eq!(AGREEMENTS.load(Ordering::Relaxed), before + 1);
    }

    #[test]
    fn vendor_degraded_counter_accumulates() {
        inc_vendor_degraded("test_vendor_x");
        inc_vendor_degraded("test_vendor_x");
        let snap = snapshot();
        assert!(*snap.vendor_degraded.get("test_vendor_x").unwrap_or(&0) >= 2);
    }
}
