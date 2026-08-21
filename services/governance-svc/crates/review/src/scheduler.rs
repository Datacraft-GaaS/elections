//! Long-drip scheduler: checks HIGH-severity findings for 30-day silence and emits alerts.
use crate::engine::ReviewEngine;
use crate::models::Finding;
use tracing::warn;

pub struct LongDripScheduler;

impl LongDripScheduler {
    /// Scan findings and return IDs requiring alerts.
    pub fn scan(findings: &[Finding]) -> Vec<String> {
        findings
            .iter()
            .filter(|f| ReviewEngine::is_long_drip_alert_due(f))
            .map(|f| f.id.clone())
            .collect()
    }

    /// Emit a warning-level log per alert (real impl would publish Kafka event).
    pub fn emit_alerts(alert_ids: &[String]) {
        for id in alert_ids {
            warn!(finding_id = %id, "long_drip_alert: HIGH finding silent past 30 days");
        }
    }
}
