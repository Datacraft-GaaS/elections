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

//! eel-abis-vendor: vendor SDK abstraction for the ABIS service.
//!
//! Provides a unified `VendorAdapter` trait over Neurotechnology, Innovatrics,
//! NEC, Idemia, and other biometric SDKs. Concrete adapters:
//! - `MockVendor`    -- deterministic, for tests
//! - `PythonVendor`  -- HTTP bridge to py/abis-vendor sidecar
//! - `GrpcVendor`    -- gRPC bridge to a vendor sidecar (stub)
//! - `VendorRouter`  -- per-modality routing with health-based failover
#![warn(missing_docs)]

pub mod adapter;
pub mod error;
pub mod grpc_bridge;
pub mod mock;
pub mod python_bridge;
pub mod router;

pub use adapter::{
	FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
	IrisTemplate, Score, Template, VendorAdapter, VendorCapabilities,
};
pub use error::VendorError;
pub use grpc_bridge::GrpcVendor;
pub use mock::MockVendor;
pub use python_bridge::PythonVendor;
pub use router::{ModalityRoutes, VendorRouter};
