use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::{BoundaryKind, BoundaryStatus, DomainError, Polygon};

/// `Boundary` is the aggregate root for a single administrative boundary
/// (county, constituency, ward, or polling station).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Boundary {
    pub id: Uuid,
    pub kind: BoundaryKind,
    /// Official code assigned by the electoral commission.
    pub code: String,
    pub name: String,
    pub geometry: Polygon,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub version: i32,
    pub status: BoundaryStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Boundary {
    /// Construct a new boundary, validating invariants at creation time.
    pub fn new(
        kind: BoundaryKind,
        code: impl Into<String>,
        name: impl Into<String>,
        geometry: Polygon,
        effective_from: NaiveDate,
    ) -> Result<Self, DomainError> {
        let code = code.into();
        if code.is_empty() {
            return Err(DomainError::EmptyCode);
        }
        if geometry.rings.is_empty() {
            return Err(DomainError::InvalidGeometry {
                reason: "boundary must have at least one ring".to_owned(),
            });
        }
        let now = Utc::now();
        Ok(Self {
            id: Uuid::now_v7(),
            kind,
            code,
            name: name.into(),
            geometry,
            effective_from,
            effective_to: None,
            version: 1,
            status: BoundaryStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    /// Supersede this boundary (e.g. after a delimitation).
    pub fn supersede(&mut self, effective_to: NaiveDate) {
        self.status = BoundaryStatus::Superseded;
        self.effective_to = Some(effective_to);
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.status == BoundaryStatus::Active
    }
}
