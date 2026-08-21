//! eel-abis-vendor: vendor SDK abstraction for the ABIS service.
//!
//! Provides a unified `VendorAdapter` trait over Neurotechnology, Innovatrics,
//! NEC, Idemia, and other biometric SDKs. Concrete adapters:
//! - `MockVendor`    -- deterministic, for tests
//! - `PythonVendor`  -- HTTP bridge to py/abis-vendor sidecar
//! - `GrpcVendor`    -- gRPC bridge to a vendor sidecar (stub)
//! - `VendorRouter`  -- per-modality routing with health-based failover

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
