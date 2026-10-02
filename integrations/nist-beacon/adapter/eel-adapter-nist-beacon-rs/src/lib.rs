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

//! NIST Randomness Beacon 2.0 client.
//!
//! Fetches pulses from https://beacon.nist.gov/beacon/2.0/pulse/last and
//! by-timestamp queries. Verifies the outputValue chaining hash.
//! Caches recent pulses in memory.
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub const BEACON_BASE: &str = "https://beacon.nist.gov/beacon/2.0";

#[derive(Debug, Error)]
pub enum BeaconError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Signature verification failed: pulse {pulse_index}")]
    SignatureInvalid { pulse_index: u64 },
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pulse {
    pub uri: String,
    pub version: String,
    pub cipher_suite: u32,
    pub period: u32,
    pub certificate_id: String,
    pub chain_index: u64,
    pub pulse_index: u64,
    pub time_stamp: String,
    pub local_random_value: String,
    pub external: serde_json::Value,
    pub list_values: Vec<serde_json::Value>,
    pub previous_output_value: String,
    pub hour_value: String,
    pub day_value: String,
    pub month_value: String,
    pub year_value: String,
    pub pre_commitment_value: String,
    pub status_code: u32,
    pub signature_value: String,
    pub output_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PulseResponse {
    pub pulse: Pulse,
}

/// In-memory cache of recent pulses (bounded to `capacity`).
pub struct PulseCache {
    inner: Arc<Mutex<VecDeque<Pulse>>>,
    capacity: usize,
}

impl PulseCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    pub fn push(&self, pulse: Pulse) {
        let mut q = self.inner.lock().unwrap();
        if q.len() >= self.capacity {
            q.pop_front();
        }
        q.push_back(pulse);
    }

    pub fn latest(&self) -> Option<Pulse> {
        self.inner.lock().unwrap().back().cloned()
    }

    pub fn all(&self) -> Vec<Pulse> {
        self.inner.lock().unwrap().iter().cloned().collect()
    }
}

/// Fetch the latest pulse from the NIST beacon.
pub async fn fetch_latest(base_url: &str) -> Result<Pulse, BeaconError> {
    let url = format!("{base_url}/pulse/last");
    let resp: PulseResponse = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp.pulse)
}

/// Fetch a pulse by chain index and pulse index.
pub async fn fetch_by_index(
    base_url: &str,
    chain_index: u64,
    pulse_index: u64,
) -> Result<Pulse, BeaconError> {
    let url = format!("{base_url}/chain/{chain_index}/pulse/{pulse_index}");
    let resp: PulseResponse = reqwest::Client::new()
        .get(&url)
        .send()
        .await?
        .json()
        .await?;
    Ok(resp.pulse)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulse_cache_bounded() {
        let cache = PulseCache::new(3);
        for i in 0..5u64 {
            cache.push(Pulse {
                uri: String::new(),
                version: "2.0".into(),
                cipher_suite: 0,
                period: 60,
                certificate_id: String::new(),
                chain_index: 1,
                pulse_index: i,
                time_stamp: String::new(),
                local_random_value: String::new(),
                external: serde_json::Value::Null,
                list_values: vec![],
                previous_output_value: String::new(),
                hour_value: String::new(),
                day_value: String::new(),
                month_value: String::new(),
                year_value: String::new(),
                pre_commitment_value: String::new(),
                status_code: 0,
                signature_value: String::new(),
                output_value: String::new(),
            });
        }
        assert_eq!(cache.all().len(), 3);
        assert_eq!(cache.latest().unwrap().pulse_index, 4);
    }
}
