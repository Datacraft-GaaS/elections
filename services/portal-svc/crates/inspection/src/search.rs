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

use std::{
    collections::HashMap,
    hint,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::time::sleep;

/// Maximum lookups per subject per day.
pub const DAILY_CAP: usize = 5;

/// Fixed response window in milliseconds (anti-enumeration).
pub const FIXED_WINDOW_MS: u64 = 200;

/// Per-day lookup counter keyed by (subject_id, date).
#[derive(Clone, Default)]
pub struct LookupCap {
    counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl LookupCap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn key(subject_id: &str) -> String {
        let today = chrono::Utc::now().format("%Y-%m-%d");
        format!("{}:{}", subject_id, today)
    }

    /// Returns true if the subject is under cap; increments counter.
    pub fn check_and_increment(&self, subject_id: &str) -> bool {
        let key = Self::key(subject_id);
        let mut counts = self.counts.lock().expect("lock poisoned");
        let entry = counts.entry(key).or_insert(0);
        if *entry >= DAILY_CAP {
            return false;
        }
        *entry += 1;
        true
    }

    pub fn count(&self, subject_id: &str) -> usize {
        let key = Self::key(subject_id);
        let counts = self.counts.lock().expect("lock poisoned");
        *counts.get(&key).unwrap_or(&0)
    }
}

/// Result shape returned from a lookup — fixed structure regardless of found/not-found
/// to prevent enumeration via response-shape differences.
#[derive(Debug, serde::Serialize)]
pub struct LookupResult {
    pub found: bool,
    pub station_id: Option<String>,
    /// Opaque token for client to display; never raw voter data.
    pub display_ref: Option<String>,
}

/// Constant-time search: always runs for at least FIXED_WINDOW_MS total wall time.
///
/// The inner work is `black_box`-ed to prevent the compiler from optimising it away.
/// Any remaining time is padded with `sleep`.
pub async fn constant_time_lookup(
    voter_id_hash: &str,
    cap: &LookupCap,
    subject_id: &str,
) -> Result<LookupResult, &'static str> {
    let start = Instant::now();

    if !cap.check_and_increment(subject_id) {
        // Still pad to fixed window before returning.
        pad_to_fixed_window(start).await;
        return Err("daily lookup cap exceeded");
    }

    // Simulate actual DB lookup — prevent compiler elision.
    let found = hint::black_box(!voter_id_hash.is_empty() && voter_id_hash.len() == 64);

    let result = LookupResult {
        found,
        station_id: if found { Some("ST-001".into()) } else { None },
        display_ref: if found {
            Some(format!("REF-{}", &voter_id_hash[..8]))
        } else {
            None
        },
    };

    pad_to_fixed_window(start).await;
    Ok(result)
}

async fn pad_to_fixed_window(start: Instant) {
    let target = Duration::from_millis(FIXED_WINDOW_MS);
    let elapsed = start.elapsed();
    if elapsed < target {
        sleep(target - elapsed).await;
    }
}
