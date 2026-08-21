//! gRPC server implementation for `Geofence.Check`.
//!
//! Called by kit-side and bvi-svc. The server validates that the requesting
//! caller holds a SPIFFE certificate for spiffe://eel.local/service/boundary-svc
//! (enforced at the transport layer by tonic mTLS; this code trusts the
//! connection-level identity).

use std::sync::Arc;

use tonic::{Request, Response, Status};
use tracing::{info, warn};

use crate::geo_check;
use crate::proto::{
    geofence_server::{Geofence, GeofenceServer},
    CheckRequest, CheckResponse,
};
use boundary_domain::types::{Coordinate, GeofencePolygon, Polygon};

/// In-memory geofence store. In production this is backed by the projections
/// read from PostgreSQL and refreshed every 15 minutes by boundary-worker.
#[derive(Clone)]
pub struct InMemoryGeofenceStore {
    /// (station_code, polygon + signature). In a real deployment this is an
    /// `Arc<RwLock<HashMap<String, GeofencePolygon>>>` refreshed periodically.
    polygons: Arc<Vec<GeofencePolygon>>,
}

impl InMemoryGeofenceStore {
    pub fn new(polygons: Vec<GeofencePolygon>) -> Self {
        Self {
            polygons: Arc::new(polygons),
        }
    }

    pub fn lookup(&self, station_code: &str) -> Option<&GeofencePolygon> {
        self.polygons
            .iter()
            .find(|p| p.station_code == station_code)
    }
}

#[derive(Clone)]
pub struct GeofenceService {
    store: InMemoryGeofenceStore,
}

impl GeofenceService {
    pub fn new(store: InMemoryGeofenceStore) -> Self {
        Self { store }
    }

    pub fn into_server(self) -> GeofenceServer<Self> {
        GeofenceServer::new(self)
    }
}

#[tonic::async_trait]
impl Geofence for GeofenceService {
    async fn check(
        &self,
        request: Request<CheckRequest>,
    ) -> Result<Response<CheckResponse>, Status> {
        let req = request.into_inner();

        if req.station_code.is_empty() {
            warn!(station_code = %req.station_code, "empty station_code in Check request");
            return Err(Status::invalid_argument("station_code must not be empty"));
        }

        let geofence = match self.store.lookup(&req.station_code) {
            Some(g) => g,
            None => {
                warn!(station_code = %req.station_code, "station not found");
                return Err(Status::not_found(format!(
                    "station {} not found",
                    req.station_code
                )));
            }
        };

        let result = geo_check::check(&geofence.polygon, req.latitude, req.longitude);
        let polygon_version = geofence.effective_from.to_string();

        info!(
            station_code = %req.station_code,
            in_boundary = result.in_boundary,
            distance_m = result.distance_from_boundary_m,
            "Geofence.Check"
        );

        Ok(Response::new(CheckResponse {
            in_boundary: result.in_boundary,
            distance_from_boundary_m: result.distance_from_boundary_m,
            polygon_version,
            signed_polygon: geofence.signature.clone(),
        }))
    }
}

/// Build a demo store with a single square station polygon for testing.
pub fn demo_store() -> InMemoryGeofenceStore {
    use chrono::NaiveDate;
    use uuid::Uuid;

    let polygon = Polygon {
        rings: vec![vec![
            Coordinate {
                longitude: 36.8,
                latitude: -1.3,
            },
            Coordinate {
                longitude: 36.9,
                latitude: -1.3,
            },
            Coordinate {
                longitude: 36.9,
                latitude: -1.2,
            },
            Coordinate {
                longitude: 36.8,
                latitude: -1.2,
            },
            Coordinate {
                longitude: 36.8,
                latitude: -1.3,
            },
        ]],
    };
    let gf = GeofencePolygon {
        id: Uuid::now_v7(),
        station_code: "KE001PS001".to_owned(),
        polygon,
        signed_by: "eel-hsm-signer".to_owned(),
        // Stub signature: 64 zero bytes represent a placeholder Ed25519 sig.
        signature: vec![0u8; 64],
        effective_from: NaiveDate::from_ymd_opt(2027, 8, 9).unwrap(),
    };
    InMemoryGeofenceStore::new(vec![gf])
}
