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

//! Batch ingest: up to 1000 signals per request with per-signal accept/reject.

use crate::error::TelemetryError;
use crate::ingest::{ingest_signal_pure, IngestOutcome, IngestRequest};
use crate::replay_defence::NonceStore;

/// Outcome for one signal in a batch.
#[derive(Debug)]
pub enum BatchItemResult {
    Accepted(IngestOutcome),
    Rejected { index: usize, reason: String },
}

/// Result of a batch ingest: partial success is allowed.
#[derive(Debug)]
pub struct BatchResult {
    pub accepted: usize,
    pub rejected: usize,
    pub items: Vec<BatchItemResult>,
}

/// Ingest up to 1000 signals. Signals beyond that limit are all rejected.
pub fn ingest_batch(
    signals: Vec<IngestRequest>,
    nonce_store: &mut dyn NonceStore,
    server_signing_key: &[u8],
) -> Result<BatchResult, TelemetryError> {
    let count = signals.len();
    if count > 1000 {
        return Err(TelemetryError::BatchTooLarge { count });
    }

    let mut items = Vec::with_capacity(count);
    let mut accepted = 0usize;
    let mut rejected = 0usize;

    for (index, req) in signals.into_iter().enumerate() {
        match ingest_signal_pure(&req, nonce_store, server_signing_key) {
            Ok(outcome) => {
                accepted += 1;
                items.push(BatchItemResult::Accepted(outcome));
            }
            Err(e) => {
                rejected += 1;
                items.push(BatchItemResult::Rejected {
                    index,
                    reason: e.to_string(),
                });
            }
        }
    }

    Ok(BatchResult {
        accepted,
        rejected,
        items,
    })
}
