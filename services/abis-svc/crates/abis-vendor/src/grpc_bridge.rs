//! GrpcVendor: calls a vendor sidecar via gRPC (proto/vendor.proto).
//!
//! The gRPC service is defined in proto/vendor.proto. Types are defined
//! manually here to avoid a build.rs/protoc dependency in the default build.
//! Enable the `grpc` feature to wire in a real tonic channel.
//!
//! In the default (no-grpc) build this module provides a no-op stub that
//! returns `VendorError::Unavailable` for every call, allowing the router
//! to fall back to another adapter.

use async_trait::async_trait;

use crate::adapter::{
	FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
	IrisTemplate, Score, Template, VendorAdapter, VendorCapabilities,
};
use crate::error::VendorError;

/// gRPC-based vendor adapter.
///
/// Connects to a vendor sidecar that implements the eel.abis.vendor.v1
/// Biometrics service. Falls back gracefully when the sidecar is absent.
pub struct GrpcVendor {
	pub sidecar_url: String,
}

impl GrpcVendor {
	pub fn new(sidecar_url: impl Into<String>) -> Self {
		Self { sidecar_url: sidecar_url.into() }
	}

	pub fn from_env() -> Self {
		let url = std::env::var("EEL_GRPC_VENDOR_URL")
			.unwrap_or_else(|_| "http://localhost:9090".to_string());
		Self::new(url)
	}
}

#[async_trait]
impl VendorAdapter for GrpcVendor {
	async fn extract_fingerprint(
		&self,
		_image: &FingerprintImage,
	) -> Result<FingerprintTemplate, VendorError> {
		Err(VendorError::Unavailable(format!(
			"gRPC vendor at {} not yet connected (enable grpc feature)",
			self.sidecar_url
		)))
	}

	async fn extract_face(&self, _image: &FaceImage) -> Result<FaceTemplate, VendorError> {
		Err(VendorError::Unavailable(format!(
			"gRPC vendor at {} not yet connected",
			self.sidecar_url
		)))
	}

	async fn extract_iris(&self, _image: &IrisImage) -> Result<IrisTemplate, VendorError> {
		Err(VendorError::Unavailable(format!(
			"gRPC vendor at {} not yet connected",
			self.sidecar_url
		)))
	}

	async fn match_1_1(
		&self,
		_probe: &Template,
		_gallery: &Template,
	) -> Result<Score, VendorError> {
		Err(VendorError::Unavailable(format!(
			"gRPC vendor at {} not yet connected",
			self.sidecar_url
		)))
	}

	async fn match_1_n(
		&self,
		_probe: &Template,
		_gallery_batch: &[Template],
	) -> Result<Vec<Score>, VendorError> {
		Err(VendorError::Unavailable(format!(
			"gRPC vendor at {} not yet connected",
			self.sidecar_url
		)))
	}

	fn capabilities(&self) -> VendorCapabilities {
		VendorCapabilities {
			name: "grpc".into(),
			fingerprint: true,
			face: true,
			iris: true,
			one_to_n: true,
			max_gallery_size: None,
		}
	}

	async fn health(&self) -> Result<HealthStatus, VendorError> {
		Ok(HealthStatus::Unhealthy)
	}
}
