//! Background workers for `boundary-svc`.
//!
//! Three jobs run on a Tokio interval:
//! 1. `shapefile_ingest` - polls MinIO for new NLC/KNBS shapefiles and imports
//!    them into the `shapefile_imports` table.
//! 2. `fairness_metrics` - recomputes compactness, population equality, and
//!    VRA-analog metrics for every active boundary kind every 15 minutes.
//! 3. `geojson_tarball` - signs and publishes a GeoJSON tarball to MinIO every
//!    15 minutes; consumers verify via verifier-rs.
#![warn(missing_docs)]

pub mod fairness;
pub mod geojson_producer;
pub mod shapefile;

use std::time::Duration;
use tokio::time;
use tracing::{error, info};

/// Run all workers forever. Call from the binary's `main` after spawning the
/// HTTP/gRPC servers.
pub async fn run_all_workers() {
    let mut interval = time::interval(Duration::from_secs(15 * 60));
    loop {
        interval.tick().await;
        info!("boundary-worker tick");

        if let Err(e) = shapefile::run_once().await {
            error!(error = %e, "shapefile ingest failed");
        }
        if let Err(e) = fairness::run_once().await {
            error!(error = %e, "fairness metrics computation failed");
        }
        if let Err(e) = geojson_producer::run_once().await {
            error!(error = %e, "GeoJSON tarball production failed");
        }
    }
}
