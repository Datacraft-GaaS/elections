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
