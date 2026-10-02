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

//! SLA sweep scheduler: flag near-breach and breached requests.

use crate::domain::{RectificationRequest, SlaStatus};
use tracing::warn;

/// Classify all open requests by SLA status. Returns (near_breach, breached) counts.
pub fn sweep_sla(requests: &[RectificationRequest]) -> (usize, usize) {
    let mut near = 0usize;
    let mut breached = 0usize;
    for r in requests {
        match r.sla_status() {
            SlaStatus::NearBreach => {
                near += 1;
                warn!(req_id = %r.id, "SLA near breach");
            }
            SlaStatus::Breached => {
                breached += 1;
                warn!(req_id = %r.id, "SLA BREACHED");
            }
            SlaStatus::OnTrack => {}
        }
    }
    (near, breached)
}
