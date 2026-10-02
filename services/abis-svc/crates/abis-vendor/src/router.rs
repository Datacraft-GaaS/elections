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

//! VendorRouter: selects an adapter per modality with health-based failover.

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use crate::adapter::{
	FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
	IrisTemplate, Score, Template, VendorAdapter, VendorCapabilities,
};
use crate::error::VendorError;

/// Routing configuration: which adapter handles which modality.
#[derive(Clone)]
pub struct ModalityRoutes {
	/// Adapter for fingerprint extraction and matching.
	pub fingerprint: Arc<dyn VendorAdapter>,
	/// Adapter for face extraction and matching.
	pub face: Arc<dyn VendorAdapter>,
	/// Adapter for iris extraction and matching.
	pub iris: Arc<dyn VendorAdapter>,
}

/// Per-adapter health cache.
struct HealthCache {
	fingerprint: HealthStatus,
	face: HealthStatus,
	iris: HealthStatus,
}

/// Routes biometric operations to the appropriate vendor adapter,
/// with health-based failover to a backup adapter.
pub struct VendorRouter {
	primary: ModalityRoutes,
	fallback: Option<Arc<dyn VendorAdapter>>,
	health_cache: RwLock<HealthCache>,
}

impl VendorRouter {
	pub fn new(primary: ModalityRoutes) -> Self {
		Self {
			primary,
			fallback: None,
			health_cache: RwLock::new(HealthCache {
				fingerprint: HealthStatus::Healthy,
				face: HealthStatus::Healthy,
				iris: HealthStatus::Healthy,
			}),
		}
	}

	pub fn with_fallback(mut self, fallback: Arc<dyn VendorAdapter>) -> Self {
		self.fallback = Some(fallback);
		self
	}

	/// Refresh the health cache for all adapters. Call periodically.
	pub async fn refresh_health(&self) {
		let fp_h = self.primary.fingerprint.health().await.unwrap_or(HealthStatus::Unhealthy);
		let fa_h = self.primary.face.health().await.unwrap_or(HealthStatus::Unhealthy);
		let ir_h = self.primary.iris.health().await.unwrap_or(HealthStatus::Unhealthy);
		let mut cache = self.health_cache.write().await;
		cache.fingerprint = fp_h;
		cache.face = fa_h;
		cache.iris = ir_h;
	}

	/// Check cached health for the fingerprint adapter.
	pub async fn fingerprint_healthy(&self) -> bool {
		self.health_cache.read().await.fingerprint == HealthStatus::Healthy
	}

	/// Check cached health for the face adapter.
	pub async fn face_healthy(&self) -> bool {
		self.health_cache.read().await.face == HealthStatus::Healthy
	}

	/// Check cached health for the iris adapter.
	pub async fn iris_healthy(&self) -> bool {
		self.health_cache.read().await.iris == HealthStatus::Healthy
	}

	async fn fingerprint_adapter(&self) -> Arc<dyn VendorAdapter> {
		if self.fingerprint_healthy().await {
			self.primary.fingerprint.clone()
		} else if let Some(fb) = &self.fallback {
			fb.clone()
		} else {
			self.primary.fingerprint.clone()
		}
	}

	async fn face_adapter(&self) -> Arc<dyn VendorAdapter> {
		if self.face_healthy().await {
			self.primary.face.clone()
		} else if let Some(fb) = &self.fallback {
			fb.clone()
		} else {
			self.primary.face.clone()
		}
	}

	async fn iris_adapter(&self) -> Arc<dyn VendorAdapter> {
		if self.iris_healthy().await {
			self.primary.iris.clone()
		} else if let Some(fb) = &self.fallback {
			fb.clone()
		} else {
			self.primary.iris.clone()
		}
	}

	fn template_adapter<'a>(
		&'a self,
		probe: &Template,
	) -> &'a Arc<dyn VendorAdapter> {
		match probe {
			Template::Fingerprint(_) => &self.primary.fingerprint,
			Template::Face(_) => &self.primary.face,
			Template::Iris(_) => &self.primary.iris,
		}
	}
}

#[async_trait]
impl VendorAdapter for VendorRouter {
	async fn extract_fingerprint(
		&self,
		image: &FingerprintImage,
	) -> Result<FingerprintTemplate, VendorError> {
		self.fingerprint_adapter().await.extract_fingerprint(image).await
	}

	async fn extract_face(&self, image: &FaceImage) -> Result<FaceTemplate, VendorError> {
		self.face_adapter().await.extract_face(image).await
	}

	async fn extract_iris(&self, image: &IrisImage) -> Result<IrisTemplate, VendorError> {
		self.iris_adapter().await.extract_iris(image).await
	}

	async fn match_1_1(
		&self,
		probe: &Template,
		gallery: &Template,
	) -> Result<Score, VendorError> {
		self.template_adapter(probe).match_1_1(probe, gallery).await
	}

	async fn match_1_n(
		&self,
		probe: &Template,
		gallery_batch: &[Template],
	) -> Result<Vec<Score>, VendorError> {
		self.template_adapter(probe).match_1_n(probe, gallery_batch).await
	}

	fn capabilities(&self) -> VendorCapabilities {
		VendorCapabilities {
			name: "router".into(),
			fingerprint: self.primary.fingerprint.capabilities().fingerprint,
			face: self.primary.face.capabilities().face,
			iris: self.primary.iris.capabilities().iris,
			one_to_n: true,
			max_gallery_size: None,
		}
	}

	async fn health(&self) -> Result<HealthStatus, VendorError> {
		let fp = self.primary.fingerprint.health().await?;
		let fa = self.primary.face.health().await?;
		let ir = self.primary.iris.health().await?;
		if fp == HealthStatus::Healthy && fa == HealthStatus::Healthy && ir == HealthStatus::Healthy {
			Ok(HealthStatus::Healthy)
		} else if fp == HealthStatus::Unhealthy
			&& fa == HealthStatus::Unhealthy
			&& ir == HealthStatus::Unhealthy
		{
			Ok(HealthStatus::Unhealthy)
		} else {
			Ok(HealthStatus::Degraded)
		}
	}
}
