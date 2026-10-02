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

//! `boundary-geofence` crate.
//!
//! Exposes `Geofence.Check` over gRPC. Tiny, fast, no PostGIS runtime dep on
//! the client side. The polygon-in-polygon test runs purely in Rust.
//!
//! p99 target: < 20 ms local (verified by the `polygon_check` bench).
#![warn(missing_docs)]

pub mod geo_check;
pub mod server;

// Re-export generated proto types.
pub mod proto {
    tonic::include_proto!("eel.boundary");
}
