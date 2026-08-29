//! tlog-api — REST (axum + utoipa) + gRPC (tonic) server for the transparency log.
#![warn(missing_docs)]

pub mod grpc;
pub mod rest;
pub mod state;

pub use state::ApiState;
