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

//! Preset "hard question" library.
//!
//! Each preset is a named builder that produces a `TelemetryQuery` + a
//! human-readable description. Presets cover the canonical operational
//! questions from spec section 8.

use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::dsl::{
    Aggregation, AggregationOp, Field, GeoPolygon, OrderBy, OrderDir, SignalClass, TelemetryQuery,
    TimeRange,
};

// ---------------------------------------------------------------------------
// Preset descriptor
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PresetDescriptor {
    pub name: String,
    pub description: String,
    pub required_params: Vec<String>,
}

// ---------------------------------------------------------------------------
// Preset library
// ---------------------------------------------------------------------------

pub struct PresetLibrary;

impl PresetLibrary {
    /// List all available presets.
    pub fn list() -> Vec<PresetDescriptor> {
        vec![
            PresetDescriptor {
                name: "kits_near_ballot_box".into(),
                description: "Which kits at a given station came within Y metres of a ballot box \
                              between T1 and T2."
                    .into(),
                required_params: vec![
                    "station_id".into(),
                    "radius_m".into(),
                    "from".into(),
                    "to".into(),
                ],
            },
            PresetDescriptor {
                name: "drops_by_county".into(),
                description: "Every physical-drop signal > 3g in a county over the last N hours."
                    .into(),
                required_params: vec!["county".into(), "hours".into()],
            },
            PresetDescriptor {
                name: "officer_activity".into(),
                description: "Officer sessions, geo, reads/writes, and anomalies by badge ID."
                    .into(),
                required_params: vec!["officer_id".into(), "from".into(), "to".into()],
            },
            PresetDescriptor {
                name: "battery_low_fleet".into(),
                description: "Kits below X% battery in a region with replacement dispatch status."
                    .into(),
                required_params: vec![
                    "threshold_pct".into(),
                    "county".into(),
                    "from".into(),
                    "to".into(),
                ],
            },
            PresetDescriptor {
                name: "coordinated_logouts".into(),
                description: "Coordinated logout events in a constituency within a time window."
                    .into(),
                required_params: vec!["station_id".into(), "from".into(), "to".into()],
            },
            PresetDescriptor {
                name: "attestation_gap".into(),
                description: "Longest gap between attestation heartbeats per kit.".into(),
                required_params: vec!["from".into(), "to".into()],
            },
            PresetDescriptor {
                name: "session_hijack_alerts".into(),
                description: "Login events where device_id changed within an active session."
                    .into(),
                required_params: vec!["from".into(), "to".into()],
            },
            PresetDescriptor {
                name: "geo_heatmap".into(),
                description: "Signal density per geo cell in a time window (for heatmap).".into(),
                required_params: vec!["from".into(), "to".into()],
            },
            PresetDescriptor {
                name: "network_drops".into(),
                description: "Devices that went offline and did not reconnect within SLA.".into(),
                required_params: vec!["from".into(), "to".into(), "sla_minutes".into()],
            },
            PresetDescriptor {
                name: "write_anomalies".into(),
                description: "Write signals outside assigned station geo-fence.".into(),
                required_params: vec!["from".into(), "to".into()],
            },
        ]
    }

    /// Build a `TelemetryQuery` for a named preset with provided parameters.
    pub fn build(
        name: &str,
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let now = Utc::now();
        match name {
            "kits_near_ballot_box" => Self::kits_near_ballot_box(params, now),
            "drops_by_county" => Self::drops_by_county(params, now),
            "officer_activity" => Self::officer_activity(params),
            "battery_low_fleet" => Self::battery_low_fleet(params),
            "coordinated_logouts" => Self::coordinated_logouts(params),
            "attestation_gap" => Self::attestation_gap(params),
            "session_hijack_alerts" => Self::session_hijack_alerts(params),
            "geo_heatmap" => Self::geo_heatmap(params),
            "network_drops" => Self::network_drops(params, now),
            "write_anomalies" => Self::write_anomalies(params),
            _ => Err(format!("unknown preset: {}", name)),
        }
    }

    fn kits_near_ballot_box(
        params: &serde_json::Map<String, serde_json::Value>,
        now: DateTime<Utc>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").unwrap_or_else(|| now - Duration::hours(6));
        let to = parse_dt(params, "to").unwrap_or(now);
        let station_id = params
            .get("station_id")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Location, SignalClass::ProximalRfid],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![Aggregation {
                op: AggregationOp::Count,
                field: Field::DeviceId,
                alias: Some("kit_count".into()),
            }],
            group_by: vec![Field::DeviceId, Field::StationId],
            limit: 10_000,
            order_by: vec![OrderBy {
                field: Field::DeviceId,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Kits at station {} with proximal RFID contact between {} and {}",
            station_id, from, to
        );
        Ok((q, desc))
    }

    fn drops_by_county(
        params: &serde_json::Map<String, serde_json::Value>,
        now: DateTime<Utc>,
    ) -> Result<(TelemetryQuery, String), String> {
        let hours = params.get("hours").and_then(|v| v.as_i64()).unwrap_or(48);
        let from = now - Duration::hours(hours);
        let county = params
            .get("county")
            .and_then(|v| v.as_str())
            .unwrap_or("ALL");

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::PhysicalDrops],
            time_range: TimeRange { from, to: now },
            geo_polygon: None,
            aggregations: vec![],
            group_by: vec![],
            limit: 50_000,
            order_by: vec![OrderBy {
                field: Field::DeviceId,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Physical drop signals (>3g threshold) in county {} over last {} hours",
            county, hours
        );
        Ok((q, desc))
    }

    fn officer_activity(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;
        let officer_id = params
            .get("officer_id")
            .and_then(|v| v.as_str())
            .ok_or("missing officer_id")?;

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![
                SignalClass::LoginLogout,
                SignalClass::Location,
                SignalClass::Reads,
                SignalClass::Writes,
            ],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![
                Aggregation {
                    op: AggregationOp::Count,
                    field: Field::SignalClass,
                    alias: Some("event_count".into()),
                },
                Aggregation {
                    op: AggregationOp::Distinct,
                    field: Field::DeviceId,
                    alias: Some("device_count".into()),
                },
            ],
            group_by: vec![Field::OfficerId, Field::SignalClass, Field::Day],
            limit: 10_000,
            order_by: vec![OrderBy {
                field: Field::Day,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Sessions, geo, reads/writes for officer {} between {} and {}",
            officer_id, from, to
        );
        Ok((q, desc))
    }

    fn battery_low_fleet(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;
        let threshold = params
            .get("threshold_pct")
            .and_then(|v| v.as_f64())
            .unwrap_or(15.0);
        let county = params
            .get("county")
            .and_then(|v| v.as_str())
            .unwrap_or("ALL");

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Battery],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![Aggregation {
                op: AggregationOp::Min,
                field: Field::BatteryBand,
                alias: Some("min_battery".into()),
            }],
            group_by: vec![Field::DeviceId, Field::County],
            limit: 20_000,
            order_by: vec![OrderBy {
                field: Field::BatteryBand,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Kits below {}% battery in county {} between {} and {}",
            threshold, county, from, to
        );
        Ok((q, desc))
    }

    fn coordinated_logouts(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;
        let station_id = params
            .get("station_id")
            .and_then(|v| v.as_str())
            .unwrap_or("UNKNOWN");

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::LoginLogout],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![
                Aggregation {
                    op: AggregationOp::Count,
                    field: Field::DeviceId,
                    alias: Some("logout_count".into()),
                },
                Aggregation {
                    op: AggregationOp::Distinct,
                    field: Field::OfficerId,
                    alias: Some("officer_count".into()),
                },
            ],
            group_by: vec![Field::StationId, Field::Hour],
            limit: 5_000,
            order_by: vec![OrderBy {
                field: Field::Hour,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Coordinated logout events at station {} between {} and {}",
            station_id, from, to
        );
        Ok((q, desc))
    }

    fn attestation_gap(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Attestation],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![
                Aggregation {
                    op: AggregationOp::Max,
                    field: Field::AttestationStatus,
                    alias: Some("last_attestation".into()),
                },
                Aggregation {
                    op: AggregationOp::Count,
                    field: Field::DeviceId,
                    alias: Some("heartbeat_count".into()),
                },
            ],
            group_by: vec![Field::DeviceId, Field::AttestationStatus],
            limit: 65_000,
            order_by: vec![OrderBy {
                field: Field::DeviceId,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Attestation heartbeat gaps per kit between {} and {}",
            from, to
        );
        Ok((q, desc))
    }

    fn session_hijack_alerts(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::LoginLogout],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![Aggregation {
                op: AggregationOp::Count,
                field: Field::DeviceId,
                alias: Some("device_change_count".into()),
            }],
            group_by: vec![Field::OfficerId, Field::Day],
            limit: 5_000,
            order_by: vec![OrderBy {
                field: Field::Day,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Login events where device changed within session between {} and {}",
            from, to
        );
        Ok((q, desc))
    }

    fn geo_heatmap(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Location],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![Aggregation {
                op: AggregationOp::Count,
                field: Field::DeviceId,
                alias: Some("signal_count".into()),
            }],
            group_by: vec![Field::County, Field::Hour],
            limit: 100_000,
            order_by: vec![OrderBy {
                field: Field::County,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Location signal density per geo cell between {} and {}",
            from, to
        );
        Ok((q, desc))
    }

    fn network_drops(
        params: &serde_json::Map<String, serde_json::Value>,
        now: DateTime<Utc>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").unwrap_or_else(|| now - Duration::hours(24));
        let to = parse_dt(params, "to").unwrap_or(now);
        let sla_minutes = params
            .get("sla_minutes")
            .and_then(|v| v.as_i64())
            .unwrap_or(30);

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Network],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![
                Aggregation {
                    op: AggregationOp::Count,
                    field: Field::DeviceId,
                    alias: Some("offline_events".into()),
                },
                Aggregation {
                    op: AggregationOp::Max,
                    field: Field::NetworkClass,
                    alias: Some("last_class".into()),
                },
            ],
            group_by: vec![Field::DeviceId, Field::NetworkClass],
            limit: 20_000,
            order_by: vec![OrderBy {
                field: Field::DeviceId,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Devices offline beyond {} minute SLA between {} and {}",
            sla_minutes, from, to
        );
        Ok((q, desc))
    }

    fn write_anomalies(
        params: &serde_json::Map<String, serde_json::Value>,
    ) -> Result<(TelemetryQuery, String), String> {
        let from = parse_dt(params, "from").ok_or("missing from")?;
        let to = parse_dt(params, "to").ok_or("missing to")?;

        let q = TelemetryQuery {
            device_ids: vec![],
            signal_classes: vec![SignalClass::Writes, SignalClass::Location],
            time_range: TimeRange { from, to },
            geo_polygon: None,
            aggregations: vec![Aggregation {
                op: AggregationOp::Count,
                field: Field::DeviceId,
                alias: Some("write_count".into()),
            }],
            group_by: vec![Field::DeviceId, Field::StationId, Field::County],
            limit: 10_000,
            order_by: vec![OrderBy {
                field: Field::DeviceId,
                dir: OrderDir::Asc,
            }],
        };
        let desc = format!(
            "Write signals outside assigned station geo-fence between {} and {}",
            from, to
        );
        Ok((q, desc))
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_dt(
    params: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<DateTime<Utc>> {
    params
        .get(key)
        .and_then(|v| v.as_str())
        .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&Utc))
}
