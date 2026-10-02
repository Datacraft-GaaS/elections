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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{BoundaryKind, ChangeType, MetricName};

// ---------------------------------------------------------------------------
// Domain events  (carried by eel-outbox-rs to Kafka)
// ---------------------------------------------------------------------------

/// `eel.boundary.boundary_changed.v1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryChangedV1 {
    pub event_id: Uuid,
    pub boundary_id: Uuid,
    pub boundary_code: String,
    pub kind: BoundaryKind,
    pub change_type: ChangeType,
    pub old_version: Option<i32>,
    pub new_version: i32,
    pub gazette_ref: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

/// `eel.boundary.geofence_polygon_signed.v1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofencePolygonSignedV1 {
    pub event_id: Uuid,
    pub geofence_id: Uuid,
    pub station_code: String,
    pub polygon_version: String,
    pub signed_by: String,
    pub occurred_at: DateTime<Utc>,
}

/// `eel.boundary.shapefile_imported.v1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapefileImportedV1 {
    pub event_id: Uuid,
    pub import_id: Uuid,
    pub source: String,
    pub version: String,
    pub file_ref: String,
    pub imported_by: String,
    pub occurred_at: DateTime<Utc>,
}

/// `eel.boundary.fairness_metrics_computed.v1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetricsComputedV1 {
    pub event_id: Uuid,
    pub boundary_kind: BoundaryKind,
    pub metric_name: MetricName,
    pub value: f64,
    pub methodology_version: String,
    pub occurred_at: DateTime<Utc>,
}

/// `eel.boundary.submission_received.v1`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmissionReceivedV1 {
    pub event_id: Uuid,
    pub submission_id: Uuid,
    pub submitter_class: crate::types::SubmitterClass,
    pub occurred_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Envelope
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BoundaryEvent {
    BoundaryChanged(BoundaryChangedV1),
    GeofencePolygonSigned(GeofencePolygonSignedV1),
    ShapefileImported(ShapefileImportedV1),
    FairnessMetricsComputed(FairnessMetricsComputedV1),
    SubmissionReceived(SubmissionReceivedV1),
}

impl BoundaryEvent {
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::BoundaryChanged(_) => "eel.boundary.boundary_changed.v1",
            Self::GeofencePolygonSigned(_) => "eel.boundary.geofence_polygon_signed.v1",
            Self::ShapefileImported(_) => "eel.boundary.shapefile_imported.v1",
            Self::FairnessMetricsComputed(_) => "eel.boundary.fairness_metrics_computed.v1",
            Self::SubmissionReceived(_) => "eel.boundary.submission_received.v1",
        }
    }
}
