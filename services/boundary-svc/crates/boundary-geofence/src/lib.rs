//! `boundary-geofence` crate.
//!
//! Exposes `Geofence.Check` over gRPC. Tiny, fast, no PostGIS runtime dep on
//! the client side. The polygon-in-polygon test runs purely in Rust.
//!
//! p99 target: < 20 ms local (verified by the `polygon_check` bench).

pub mod geo_check;
pub mod server;

// Re-export generated proto types.
pub mod proto {
    tonic::include_proto!("eel.boundary");
}
