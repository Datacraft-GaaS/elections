//! DSL safety validator.
//!
//! Validates a `TelemetryQuery` against an allow-list of safe operations
//! before it touches the database. Returns structured `Diagnostic` values
//! so callers can surface actionable error messages.

use std::collections::HashSet;

use chrono::Duration;
use thiserror::Error;

use crate::dsl::{AggregationOp, Field, GeoPolygon, SignalClass, TelemetryQuery};

// ---------------------------------------------------------------------------
// Error types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagnostic {
    pub field: String,
    pub code: String,
    pub message: String,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("query validation failed with {} error(s)", .0.len())]
    Invalid(Vec<Diagnostic>),
}

// ---------------------------------------------------------------------------
// Validator
// ---------------------------------------------------------------------------

/// Validates a `TelemetryQuery` for safety by construction.
pub struct TelemetryQueryValidator;

impl TelemetryQueryValidator {
    /// Validate the query, returning `Ok(())` or `Err(ValidationError::Invalid)`.
    pub fn validate(q: &TelemetryQuery) -> Result<(), ValidationError> {
        let mut diags: Vec<Diagnostic> = vec![];

        // Time range present and non-inverted
        if q.time_range.from >= q.time_range.to {
            diags.push(Diagnostic {
                field: "time_range".into(),
                code: "TIME_RANGE_INVERTED".into(),
                message: "`from` must be strictly before `to`".into(),
            });
        }

        // Time range <= 90 days
        let span = q.time_range.to - q.time_range.from;
        if span > Duration::days(90) {
            diags.push(Diagnostic {
                field: "time_range".into(),
                code: "TIME_RANGE_TOO_WIDE".into(),
                message: "time range must not exceed 90 days".into(),
            });
        }

        // Signal classes allow-list
        let allowed_classes: HashSet<&SignalClass> = [
            SignalClass::Location,
            SignalClass::Time,
            SignalClass::Activity,
            SignalClass::ProximalRfid,
            SignalClass::Movements,
            SignalClass::LoginLogout,
            SignalClass::Reads,
            SignalClass::Writes,
            SignalClass::Battery,
            SignalClass::LocksUnlocks,
            SignalClass::PhysicalDrops,
            SignalClass::Network,
            SignalClass::Attestation,
        ]
        .iter()
        .collect();

        for sc in &q.signal_classes {
            if !allowed_classes.contains(sc) {
                diags.push(Diagnostic {
                    field: "signal_classes".into(),
                    code: "UNKNOWN_SIGNAL_CLASS".into(),
                    message: format!("unknown signal class: {:?}", sc),
                });
            }
        }

        // Aggregation ops allow-list
        let allowed_ops: HashSet<AggregationOp> = [
            AggregationOp::Count,
            AggregationOp::Sum,
            AggregationOp::Avg,
            AggregationOp::Min,
            AggregationOp::Max,
            AggregationOp::Distinct,
        ]
        .into_iter()
        .collect();

        for agg in &q.aggregations {
            if !allowed_ops.contains(&agg.op) {
                diags.push(Diagnostic {
                    field: "aggregations".into(),
                    code: "UNKNOWN_AGG_OP".into(),
                    message: format!("unknown aggregation op: {:?}", agg.op),
                });
            }
        }

        // group_by without aggregations is a logic error
        if !q.group_by.is_empty() && q.aggregations.is_empty() {
            diags.push(Diagnostic {
                field: "group_by".into(),
                code: "GROUP_BY_WITHOUT_AGG".into(),
                message: "group_by requires at least one aggregation".into(),
            });
        }

        // group_by field allow-list
        let allowed_fields: HashSet<&Field> = [
            Field::DeviceId,
            Field::OfficerId,
            Field::SignalClass,
            Field::Severity,
            Field::County,
            Field::StationId,
            Field::Hour,
            Field::Day,
            Field::MotionClass,
            Field::NetworkClass,
            Field::BatteryBand,
            Field::AttestationStatus,
        ]
        .iter()
        .collect();

        for f in &q.group_by {
            if !allowed_fields.contains(f) {
                diags.push(Diagnostic {
                    field: "group_by".into(),
                    code: "UNKNOWN_GROUP_BY_FIELD".into(),
                    message: format!("unknown group_by field: {:?}", f),
                });
            }
        }

        // Geo polygon: <= 100 vertices
        if let Some(GeoPolygon { coordinates }) = &q.geo_polygon {
            if coordinates.len() > 100 {
                diags.push(Diagnostic {
                    field: "geo_polygon".into(),
                    code: "GEO_POLYGON_TOO_MANY_VERTICES".into(),
                    message: format!("geo polygon has {} vertices; max is 100", coordinates.len()),
                });
            }
            if coordinates.len() < 3 {
                diags.push(Diagnostic {
                    field: "geo_polygon".into(),
                    code: "GEO_POLYGON_TOO_FEW_VERTICES".into(),
                    message: "geo polygon requires at least 3 vertices".into(),
                });
            }
        }

        // Limit: 1 <= limit <= 100000
        if q.limit == 0 {
            diags.push(Diagnostic {
                field: "limit".into(),
                code: "LIMIT_ZERO".into(),
                message: "limit must be at least 1".into(),
            });
        }
        if q.limit > 100_000 {
            diags.push(Diagnostic {
                field: "limit".into(),
                code: "LIMIT_EXCEEDS_CAP".into(),
                message: format!("limit {} exceeds hard cap of 100000", q.limit),
            });
        }

        if diags.is_empty() {
            Ok(())
        } else {
            Err(ValidationError::Invalid(diags))
        }
    }
}
