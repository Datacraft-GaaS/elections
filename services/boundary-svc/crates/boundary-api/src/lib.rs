//! `boundary-api` HTTP + gRPC server for boundary-svc.
//!
//! Exposes:
//! - Public HTTP: constituencies, wards, counties, stations, changes, bulk GeoJSON,
//!   geofence, fairness metrics, impact simulator, submissions.
//! - Admin HTTP (JWT+SpiceDB): change approval, geofence upload, shapefile import.
//! - Internal gRPC: Geofence.Check (see boundary-geofence crate).

pub mod error;
pub mod handlers;
pub mod openapi;
pub mod routes;
