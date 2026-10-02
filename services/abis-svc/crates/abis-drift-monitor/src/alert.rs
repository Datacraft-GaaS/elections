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

//! Alert emission on drift threshold breach.

use chrono::Utc;
use tracing::warn;
use uuid::Uuid;

use crate::domain::{DriftAlert, DriftReport, DriftSeverity};

/// Create a DriftAlert from a report if severity is Warning or Critical.
pub fn maybe_alert(model_id: Uuid, report: &DriftReport) -> Option<DriftAlert> {
    if report.severity == DriftSeverity::Info {
        return None;
    }
    warn!(
        feature = %report.feature_name,
        psi = report.psi,
        severity = ?report.severity,
        "drift alert triggered"
    );
    Some(DriftAlert {
        id: Uuid::now_v7(),
        model_id,
        feature_name: report.feature_name.clone(),
        severity: report.severity.clone(),
        psi: report.psi,
        ks_stat: report.ks_stat,
        message: format!(
            "Drift detected: PSI={:.4} KS={:.4} severity={:?}",
            report.psi, report.ks_stat, report.severity
        ),
        triggered_at: Utc::now(),
        resolved_at: None,
    })
}
