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
