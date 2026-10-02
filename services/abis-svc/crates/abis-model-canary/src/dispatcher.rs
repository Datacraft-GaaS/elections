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

//! Request-time canary dispatcher: deterministic FNV-1a hash routing per subject.

use uuid::Uuid;

use crate::domain::CanaryRecord;

/// Returns true if this subject_id should be routed to the canary (to_model).
pub fn route_to_canary(subject_id: Uuid, canary: &CanaryRecord) -> bool {
    if canary.traffic_pct == 0 {
        return false;
    }
    if canary.traffic_pct >= 100 {
        return true;
    }
    let bytes = subject_id.as_bytes();
    let mut hash: u64 = 0xcbf29ce484222325;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    ((hash % 100) as u8) < canary.traffic_pct
}

/// Select the model ID to use for a given subject.
pub fn select_model(subject_id: Uuid, baseline_model: Uuid, canary: Option<&CanaryRecord>) -> Uuid {
    match canary {
        Some(c) if route_to_canary(subject_id, c) => c.to_model_id,
        _ => baseline_model,
    }
}
