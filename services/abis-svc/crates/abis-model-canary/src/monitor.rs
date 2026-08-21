//! Canary SLA monitor: checks metrics and flags auto-rollback.

use tracing::warn;

use crate::domain::{CanaryMetrics, CanaryRecord, RollbackReason};

/// Returns Some(reason) if any SLA is breached, None if healthy.
pub fn check_sla(record: &CanaryRecord, metrics: &CanaryMetrics) -> Option<RollbackReason> {
    if metrics.fmr > record.sla_fmr_limit {
        warn!(
            fmr = metrics.fmr,
            limit = record.sla_fmr_limit,
            "FMR breach"
        );
        return Some(RollbackReason::FmrBreach {
            observed: metrics.fmr,
            limit: record.sla_fmr_limit,
        });
    }
    if metrics.fnmr > record.sla_fnmr_limit {
        warn!(
            fnmr = metrics.fnmr,
            limit = record.sla_fnmr_limit,
            "FNMR breach"
        );
        return Some(RollbackReason::FnmrBreach {
            observed: metrics.fnmr,
            limit: record.sla_fnmr_limit,
        });
    }
    if metrics.latency_p99_ms > record.sla_latency_p99_ms {
        warn!(
            p99 = metrics.latency_p99_ms,
            limit = record.sla_latency_p99_ms,
            "latency breach"
        );
        return Some(RollbackReason::LatencyBreach {
            p99_ms: metrics.latency_p99_ms,
            limit_ms: record.sla_latency_p99_ms,
        });
    }
    if metrics.error_rate > record.sla_error_rate_limit {
        warn!(
            rate = metrics.error_rate,
            limit = record.sla_error_rate_limit,
            "error rate breach"
        );
        return Some(RollbackReason::ErrorRateBreach {
            rate: metrics.error_rate,
            limit: record.sla_error_rate_limit,
        });
    }
    None
}
