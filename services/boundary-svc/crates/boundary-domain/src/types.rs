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

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// Enumerations
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryKind {
    County,
    Constituency,
    Ward,
    PollingStation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BoundaryStatus {
    Active,
    Superseded,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitterClass {
    Public,
    PoliticalParty,
    CivilSociety,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionStatus {
    Pending,
    UnderReview,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricName {
    Compactness,
    PopulationEquality,
    VraAnalog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Created,
    BoundaryUpdated,
    NameChanged,
    Deactivated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeStatus {
    Pending,
    Approved,
    Rejected,
}

// ---------------------------------------------------------------------------
// Value objects
// ---------------------------------------------------------------------------

/// Coordinates stored as WGS-84 (EPSG:4326).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coordinate {
    pub longitude: f64,
    pub latitude: f64,
}

/// A polygon as a list of rings (first ring = exterior, rest = holes).
/// Each ring is a Vec of Coordinate closed on itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polygon {
    pub rings: Vec<Vec<Coordinate>>,
}

impl Polygon {
    /// Ray-casting point-in-polygon test. O(n) per ring.
    pub fn contains(&self, lat: f64, lon: f64) -> bool {
        let exterior = match self.rings.first() {
            Some(r) => r,
            None => return false,
        };
        point_in_ring(exterior, lat, lon)
    }
}

fn point_in_ring(ring: &[Coordinate], lat: f64, lon: f64) -> bool {
    let n = ring.len();
    let mut inside = false;
    let mut j = n.wrapping_sub(1);
    for i in 0..n {
        let xi = ring[i].longitude;
        let yi = ring[i].latitude;
        let xj = ring[j].longitude;
        let yj = ring[j].latitude;
        let intersects =
            ((yi > lat) != (yj > lat)) && (lon < (xj - xi) * (lat - yi) / (yj - yi) + xi);
        if intersects {
            inside = !inside;
        }
        j = i;
    }
    inside
}

// ---------------------------------------------------------------------------
// Domain structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FairnessMetric {
    pub id: Uuid,
    pub boundary_kind: BoundaryKind,
    pub metric_name: MetricName,
    pub value: f64,
    pub computed_at: DateTime<Utc>,
    pub methodology_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeofencePolygon {
    pub id: Uuid,
    pub station_code: String,
    pub polygon: Polygon,
    pub signed_by: String,
    pub signature: Vec<u8>,
    pub effective_from: NaiveDate,
}

impl GeofencePolygon {
    /// Reject polygons with no signature bytes. Hard invariant: forged (unsigned) polygons MUST
    /// be rejected at ingest. Call before persisting any GeofencePolygon.
    pub fn validate_signature(&self) -> Result<(), DomainError> {
        if self.signature.is_empty() {
            return Err(DomainError::UnsignedPolygon {
                station_code: self.station_code.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryChange {
    pub id: Uuid,
    pub boundary_id: Uuid,
    pub change_type: ChangeType,
    pub old_version: Option<i32>,
    pub new_version: i32,
    pub gazette_ref: Option<String>,
    pub submitted_by: String,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTime<Utc>>,
    pub status: ChangeStatus,
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("polygon for station {station_code} has no signature; ingest rejected")]
    UnsignedPolygon { station_code: String },

    #[error("boundary {code} not found")]
    NotFound { code: String },

    #[error("boundary code is empty")]
    EmptyCode,

    #[error("invalid geometry: {reason}")]
    InvalidGeometry { reason: String },
}
