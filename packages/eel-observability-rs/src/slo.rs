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

use serde::{Deserialize, Serialize};

/// An SLO definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SloDefinition {
    /// Service name (e.g. "rts-svc").
    pub service: String,
    /// Indicator type (e.g. "availability", "latency_p99").
    pub indicator: String,
    /// Target value (e.g. 0.999 for 99.9%).
    pub target: f64,
    /// Prometheus recording rule name (auto-generated).
    pub recording_rule: String,
}

impl SloDefinition {
    pub fn new(service: &str, indicator: &str, target: f64) -> Self {
        let recording_rule = format!("slo:{}:{}:{}", service, indicator, (target * 1000.0) as u64);
        Self {
            service: service.into(),
            indicator: indicator.into(),
            target,
            recording_rule,
        }
    }
}

/// Register an SLO definition and emit a tracing event with its metadata.
///
/// Usage: `slo!(service: "rts-svc", indicator: "availability", target: 0.999)`
#[macro_export]
macro_rules! slo {
	(service: $svc:expr, indicator: $ind:expr, target: $tgt:expr) => {{
		let def = $crate::slo::SloDefinition::new($svc, $ind, $tgt);
		tracing::info!(
			service = %def.service,
			indicator = %def.indicator,
			target = def.target,
			recording_rule = %def.recording_rule,
			"slo_registered"
		);
		def
	}};
}
