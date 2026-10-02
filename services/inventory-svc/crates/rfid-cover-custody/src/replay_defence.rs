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

//! Replay-nonce deduplication within a 5-minute window per tag.
//!
//! Production plugs to Valkey (SET NX EX 300); this module provides:
//! - `NonceStore` trait for both implementations to satisfy
//! - `InMemoryNonceStore` for tests and single-instance deployments

use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;

use crate::error::RfidError;

/// Window within which a repeated nonce for the same tag is rejected.
pub const REPLAY_WINDOW: Duration = Duration::seconds(300); // 5 minutes

/// A (tag_id, nonce) pair that was seen at `seen_at`.
#[derive(Debug, Clone)]
struct Seen {
    seen_at: DateTime<Utc>,
}

/// Nonce deduplication interface.
pub trait NonceStore: Send + Sync {
    /// Record a nonce for `tag_id` as of `now`.
    ///
    /// Returns `Ok(())` if the nonce is fresh. Returns `Err(ReplayDetected)` if
    /// the same (tag_id, nonce) was already seen within the replay window.
    fn check_and_record(
        &mut self,
        tag_id: &str,
        nonce: &[u8],
        now: DateTime<Utc>,
    ) -> Result<(), RfidError>;
}

/// In-memory nonce store using a HashMap. Prunes expired entries on each call.
///
/// Not suitable for multi-instance deployments; use the Valkey adapter in production.
#[derive(Default)]
pub struct InMemoryNonceStore {
    seen: HashMap<(String, Vec<u8>), Seen>,
}

impl InMemoryNonceStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove entries older than `REPLAY_WINDOW` relative to `now`.
    fn prune(&mut self, now: DateTime<Utc>) {
        self.seen.retain(|_, v| now - v.seen_at < REPLAY_WINDOW);
    }
}

impl NonceStore for InMemoryNonceStore {
    fn check_and_record(
        &mut self,
        tag_id: &str,
        nonce: &[u8],
        now: DateTime<Utc>,
    ) -> Result<(), RfidError> {
        self.prune(now);

        let key = (tag_id.to_owned(), nonce.to_vec());
        if self.seen.contains_key(&key) {
            return Err(RfidError::ReplayDetected {
                tag_id: tag_id.to_owned(),
            });
        }

        self.seen.insert(key, Seen { seen_at: now });
        Ok(())
    }
}
