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
