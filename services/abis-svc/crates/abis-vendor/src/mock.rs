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

//! MockVendor: deterministic adapter for tests.
//!
//! Identical templates (same data bytes) score 1.0; different templates score ~0.1.

use async_trait::async_trait;
use sha2::{Digest, Sha256};

use crate::adapter::{
	FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
	IrisTemplate, Score, Template, VendorAdapter, VendorCapabilities,
};
use crate::error::VendorError;

/// Deterministic mock vendor for tests.
pub struct MockVendor;

fn mock_fingerprint_data(image: &FingerprintImage) -> Vec<u8> {
	let mut h = Sha256::new();
	h.update(&image.data);
	h.finalize().to_vec()
}

fn mock_face_data(image: &FaceImage) -> Vec<u8> {
	let mut h = Sha256::new();
	h.update(&image.data);
	h.finalize().to_vec()
}

fn mock_iris_data(image: &IrisImage) -> Vec<u8> {
	let mut h = Sha256::new();
	h.update(&image.data);
	h.finalize().to_vec()
}

fn compare_templates(probe: &Template, gallery: &Template) -> Score {
	let (p_data, g_data) = match (probe, gallery) {
		(Template::Fingerprint(p), Template::Fingerprint(g)) => (&p.data, &g.data),
		(Template::Face(p), Template::Face(g)) => (&p.data, &g.data),
		(Template::Iris(p), Template::Iris(g)) => (&p.data, &g.data),
		_ => return Score::new(0.0),
	};
	if p_data == g_data {
		Score::new(1.0)
	} else {
		Score::new(0.1)
	}
}

#[async_trait]
impl VendorAdapter for MockVendor {
	async fn extract_fingerprint(
		&self,
		image: &FingerprintImage,
	) -> Result<FingerprintTemplate, VendorError> {
		Ok(FingerprintTemplate {
			vendor: "mock".into(),
			version: 1,
			data: mock_fingerprint_data(image),
			quality: 0.85,
		})
	}

	async fn extract_face(&self, image: &FaceImage) -> Result<FaceTemplate, VendorError> {
		let data = mock_face_data(image);
		// Build a deterministic 16-element embedding from hash bytes.
		let embedding: Vec<f32> = data.iter().take(16).map(|b| *b as f32 / 255.0).collect();
		Ok(FaceTemplate {
			vendor: "mock".into(),
			version: 1,
			data,
			embedding: Some(embedding),
			quality: 0.9,
		})
	}

	async fn extract_iris(&self, image: &IrisImage) -> Result<IrisTemplate, VendorError> {
		Ok(IrisTemplate {
			vendor: "mock".into(),
			version: 1,
			data: mock_iris_data(image),
			quality: 0.88,
		})
	}

	async fn match_1_1(
		&self,
		probe: &Template,
		gallery: &Template,
	) -> Result<Score, VendorError> {
		Ok(compare_templates(probe, gallery))
	}

	async fn match_1_n(
		&self,
		probe: &Template,
		gallery_batch: &[Template],
	) -> Result<Vec<Score>, VendorError> {
		Ok(gallery_batch.iter().map(|g| compare_templates(probe, g)).collect())
	}

	fn capabilities(&self) -> VendorCapabilities {
		VendorCapabilities {
			name: "mock".into(),
			fingerprint: true,
			face: true,
			iris: true,
			one_to_n: true,
			max_gallery_size: None,
		}
	}

	async fn health(&self) -> Result<HealthStatus, VendorError> {
		Ok(HealthStatus::Healthy)
	}
}
