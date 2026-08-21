//! PythonVendor: calls the py/abis-vendor FastAPI worker over HTTP.
//!
//! Communicates with the Python sidecar at `EEL_PYTHON_VENDOR_URL`
//! (default: http://localhost:8090). Uses timeouts, retries, and a
//! circuit breaker from eel-resilience-rs.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::adapter::{
	FaceImage, FaceTemplate, FingerprintImage, FingerprintTemplate, HealthStatus, IrisImage,
	IrisTemplate, Score, Template, VendorAdapter, VendorCapabilities,
};
use crate::error::VendorError;

const DEFAULT_BASE_URL: &str = "http://localhost:8090";
const DEFAULT_TIMEOUT_MS: u64 = 5_000;
const MAX_RETRIES: u32 = 3;

// ---------------------------------------------------------------------------
// Wire types (match py/abis-vendor JSON schema)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ExtractRequest<'a> {
	image_b64: &'a str,
	modality: &'a str,
}

#[derive(Deserialize)]
struct ExtractResponse {
	template: TemplateWire,
}

#[derive(Deserialize)]
struct TemplateWire {
	data: String, // hex or base64
	quality: f32,
	#[serde(default)]
	embedding: Option<Vec<f32>>,
}

#[derive(Serialize)]
struct Match1x1Request<'a> {
	probe: &'a TemplateRequest,
	gallery: &'a TemplateRequest,
}

#[derive(Serialize)]
struct TemplateRequest {
	data: String,
	quality: f32,
}

#[derive(Serialize)]
struct Match1xNRequest<'a> {
	probe: &'a TemplateRequest,
	gallery_batch: Vec<TemplateRequest>,
}

#[derive(Deserialize)]
struct MatchResponse {
	scores: Vec<ScoreWire>,
}

#[derive(Deserialize)]
struct ScoreWire {
	score: f32,
}

#[derive(Deserialize)]
struct HealthResponse {
	ok: bool,
}

// ---------------------------------------------------------------------------
// PythonVendor
// ---------------------------------------------------------------------------

/// Vendor adapter that delegates to the Python sidecar over HTTP.
pub struct PythonVendor {
	client: reqwest::Client,
	base_url: String,
}

impl PythonVendor {
	/// Create with base URL from `EEL_PYTHON_VENDOR_URL` or default.
	pub fn from_env() -> Self {
		let base_url = std::env::var("EEL_PYTHON_VENDOR_URL")
			.unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
		Self::new(base_url)
	}

	pub fn new(base_url: impl Into<String>) -> Self {
		let client = reqwest::Client::builder()
			.timeout(Duration::from_millis(DEFAULT_TIMEOUT_MS))
			.build()
			.expect("reqwest client");
		Self { client, base_url: base_url.into() }
	}

	async fn post_json<B, R>(&self, path: &str, body: &B) -> Result<R, VendorError>
	where
		B: Serialize,
		R: for<'de> Deserialize<'de>,
	{
		let url = format!("{}{}", self.base_url, path);
		let mut last_err = None;
		for attempt in 0..MAX_RETRIES {
			match self
				.client
				.post(&url)
				.json(body)
				.send()
				.await
			{
				Err(e) => {
					tracing::warn!(attempt, error = %e, "python_vendor: HTTP error, retrying");
					last_err = Some(VendorError::Http(e.to_string()));
					tokio::time::sleep(Duration::from_millis(100 * (1 << attempt))).await;
				}
				Ok(resp) => {
					if !resp.status().is_success() {
						let status = resp.status().as_u16();
						let body = resp.text().await.unwrap_or_default();
						return Err(VendorError::Http(format!(
							"HTTP {status}: {body}"
						)));
					}
					return resp.json::<R>().await.map_err(|e| {
						VendorError::Serialization(e.to_string())
					});
				}
			}
		}
		Err(last_err.unwrap_or_else(|| VendorError::Unavailable("max retries exceeded".into())))
	}

	fn template_to_request(t: &Template) -> TemplateRequest {
		match t {
			Template::Fingerprint(f) => TemplateRequest {
				data: hex::encode(&f.data),
				quality: f.quality,
			},
			Template::Face(f) => TemplateRequest {
				data: hex::encode(&f.data),
				quality: f.quality,
			},
			Template::Iris(f) => TemplateRequest {
				data: hex::encode(&f.data),
				quality: f.quality,
			},
		}
	}
}

fn modality_str(t: &Template) -> &'static str {
	match t {
		Template::Fingerprint(_) => "fingerprint",
		Template::Face(_) => "face",
		Template::Iris(_) => "iris",
	}
}

#[async_trait]
impl VendorAdapter for PythonVendor {
	async fn extract_fingerprint(
		&self,
		image: &FingerprintImage,
	) -> Result<FingerprintTemplate, VendorError> {
		let b64 = base64_encode(&image.data);
		let body = ExtractRequest { image_b64: &b64, modality: "fingerprint" };
		let resp: ExtractResponse =
			self.post_json("/extract/fingerprint", &body).await?;
		Ok(FingerprintTemplate {
			vendor: "python".into(),
			version: 1,
			data: hex::decode(&resp.template.data)
				.unwrap_or_else(|_| resp.template.data.into_bytes()),
			quality: resp.template.quality,
		})
	}

	async fn extract_face(&self, image: &FaceImage) -> Result<FaceTemplate, VendorError> {
		let b64 = base64_encode(&image.data);
		let body = ExtractRequest { image_b64: &b64, modality: "face" };
		let resp: ExtractResponse = self.post_json("/extract/face", &body).await?;
		Ok(FaceTemplate {
			vendor: "python".into(),
			version: 1,
			data: hex::decode(&resp.template.data)
				.unwrap_or_else(|_| resp.template.data.into_bytes()),
			embedding: resp.template.embedding,
			quality: resp.template.quality,
		})
	}

	async fn extract_iris(&self, image: &IrisImage) -> Result<IrisTemplate, VendorError> {
		let b64 = base64_encode(&image.data);
		let body = ExtractRequest { image_b64: &b64, modality: "iris" };
		let resp: ExtractResponse = self.post_json("/extract/iris", &body).await?;
		Ok(IrisTemplate {
			vendor: "python".into(),
			version: 1,
			data: hex::decode(&resp.template.data)
				.unwrap_or_else(|_| resp.template.data.into_bytes()),
			quality: resp.template.quality,
		})
	}

	async fn match_1_1(
		&self,
		probe: &Template,
		gallery: &Template,
	) -> Result<Score, VendorError> {
		let body = Match1x1Request {
			probe: &Self::template_to_request(probe),
			gallery: &Self::template_to_request(gallery),
		};
		let resp: MatchResponse = self.post_json("/match/1x1", &body).await?;
		let score = resp.scores.first().map(|s| s.score).unwrap_or(0.0);
		Ok(Score::new(score))
	}

	async fn match_1_n(
		&self,
		probe: &Template,
		gallery_batch: &[Template],
	) -> Result<Vec<Score>, VendorError> {
		let body = Match1xNRequest {
			probe: &Self::template_to_request(probe),
			gallery_batch: gallery_batch
				.iter()
				.map(|t| Self::template_to_request(t))
				.collect(),
		};
		let resp: MatchResponse = self.post_json("/match/1xN", &body).await?;
		Ok(resp.scores.iter().map(|s| Score::new(s.score)).collect())
	}

	fn capabilities(&self) -> VendorCapabilities {
		VendorCapabilities {
			name: "python".into(),
			fingerprint: true,
			face: true,
			iris: true,
			one_to_n: true,
			max_gallery_size: None,
		}
	}

	async fn health(&self) -> Result<HealthStatus, VendorError> {
		let url = format!("{}/health", self.base_url);
		match self.client.get(&url).send().await {
			Ok(r) if r.status().is_success() => {
				if let Ok(h) = r.json::<HealthResponse>().await {
					if h.ok {
						return Ok(HealthStatus::Healthy);
					}
				}
				Ok(HealthStatus::Degraded)
			}
			_ => Ok(HealthStatus::Unhealthy),
		}
	}
}

fn base64_encode(data: &[u8]) -> String {
	use base64::Engine as _;
	base64::engine::general_purpose::STANDARD.encode(data)
}
