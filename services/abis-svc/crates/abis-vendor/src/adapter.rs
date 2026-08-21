//! VendorAdapter trait and supporting types.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::VendorError;

// ---------------------------------------------------------------------------
// Input image types
// ---------------------------------------------------------------------------

/// Raw fingerprint image bytes (grayscale, 8-bit, variable resolution).
#[derive(Debug, Clone)]
pub struct FingerprintImage {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub dpi: u32,
}

/// Raw face image bytes (JPEG or PNG).
#[derive(Debug, Clone)]
pub struct FaceImage {
	pub data: Vec<u8>,
	pub mime_type: String,
}

/// Raw iris image bytes (grayscale, 8-bit).
#[derive(Debug, Clone)]
pub struct IrisImage {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
}

// ---------------------------------------------------------------------------
// Template types
// ---------------------------------------------------------------------------

/// Opaque biometric template (ISO/IEC 19794 or vendor proprietary).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FingerprintTemplate {
	pub vendor: String,
	pub version: u8,
	pub data: Vec<u8>,
	pub quality: f32,
}

/// Face template (typically a 512-d embedding vector).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceTemplate {
	pub vendor: String,
	pub version: u8,
	pub data: Vec<u8>,
	/// L2-normalised 512-d embedding (present for embedding-based vendors).
	pub embedding: Option<Vec<f32>>,
	pub quality: f32,
}

/// Iris template (typically an IrisCode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrisTemplate {
	pub vendor: String,
	pub version: u8,
	pub data: Vec<u8>,
	pub quality: f32,
}

/// Unified template wrapper for routing decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "modality", rename_all = "snake_case")]
pub enum Template {
	Fingerprint(FingerprintTemplate),
	Face(FaceTemplate),
	Iris(IrisTemplate),
}

// ---------------------------------------------------------------------------
// Match score
// ---------------------------------------------------------------------------

/// Match score in [0.0, 1.0] where 1.0 is an identical match.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Score {
	pub value: f32,
}

impl Score {
	pub fn new(value: f32) -> Self {
		Self { value: value.clamp(0.0, 1.0) }
	}
}

// ---------------------------------------------------------------------------
// Vendor capabilities and health
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorCapabilities {
	pub name: String,
	pub fingerprint: bool,
	pub face: bool,
	pub iris: bool,
	/// Whether 1:N search is supported natively.
	pub one_to_n: bool,
	pub max_gallery_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
	Healthy,
	Degraded,
	Unhealthy,
}

// ---------------------------------------------------------------------------
// Trait
// ---------------------------------------------------------------------------

/// Async trait representing a biometric vendor SDK adapter.
#[async_trait]
pub trait VendorAdapter: Send + Sync {
	/// Extract a fingerprint template from a raw image.
	async fn extract_fingerprint(
		&self,
		image: &FingerprintImage,
	) -> Result<FingerprintTemplate, VendorError>;

	/// Extract a face template from a raw image.
	async fn extract_face(&self, image: &FaceImage) -> Result<FaceTemplate, VendorError>;

	/// Extract an iris template from a raw image.
	async fn extract_iris(&self, image: &IrisImage) -> Result<IrisTemplate, VendorError>;

	/// 1:1 verification match.
	async fn match_1_1(
		&self,
		probe: &Template,
		gallery: &Template,
	) -> Result<Score, VendorError>;

	/// 1:N identification match against a batch.
	async fn match_1_n(
		&self,
		probe: &Template,
		gallery_batch: &[Template],
	) -> Result<Vec<Score>, VendorError>;

	/// Return static capability description.
	fn capabilities(&self) -> VendorCapabilities;

	/// Return current health status.
	async fn health(&self) -> Result<HealthStatus, VendorError>;
}
