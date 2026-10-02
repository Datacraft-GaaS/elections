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
