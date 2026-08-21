use base64::{engine::general_purpose::STANDARD as B64, Engine};
use crate::error::SecretsError;

/// Transit encryption helper matching the ADR-0016 KEK-per-cycle model.
pub struct Transit {
	/// OpenBao address (e.g. "http://127.0.0.1:8200")
	address: String,
	/// Vault token for authentication
	token: String,
	/// Transit mount path (default "transit")
	mount: String,
}

impl Transit {
	pub fn new(address: impl Into<String>, token: impl Into<String>) -> Self {
		Self::with_mount(address, token, "transit")
	}

	pub fn with_mount(
		address: impl Into<String>,
		token: impl Into<String>,
		mount: impl Into<String>,
	) -> Self {
		Self { address: address.into(), token: token.into(), mount: mount.into() }
	}

	/// Encrypt plaintext under the named key.
	pub async fn encrypt(&self, key_name: &str, plaintext: &[u8]) -> Result<String, SecretsError> {
		let client = reqwest::Client::builder()
			.use_rustls_tls()
			.build()
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		let b64 = B64.encode(plaintext);
		let url = format!("{}/v1/{}/encrypt/{}", self.address, self.mount, key_name);
		let body = serde_json::json!({"plaintext": b64});

		let resp = client
			.post(&url)
			.header("X-Vault-Token", &self.token)
			.json(&body)
			.send()
			.await
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(SecretsError::Transit(format!("HTTP {}", resp.status())));
		}

		let json: serde_json::Value =
			resp.json().await.map_err(|e| SecretsError::Transit(e.to_string()))?;

		json["data"]["ciphertext"]
			.as_str()
			.map(str::to_owned)
			.ok_or_else(|| SecretsError::Transit("no ciphertext in response".into()))
	}

	/// Decrypt ciphertext, returning plaintext bytes.
	pub async fn decrypt(
		&self,
		key_name: &str,
		ciphertext: &str,
	) -> Result<Vec<u8>, SecretsError> {
		let client = reqwest::Client::builder()
			.use_rustls_tls()
			.build()
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		let url = format!("{}/v1/{}/decrypt/{}", self.address, self.mount, key_name);
		let body = serde_json::json!({"ciphertext": ciphertext});

		let resp = client
			.post(&url)
			.header("X-Vault-Token", &self.token)
			.json(&body)
			.send()
			.await
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(SecretsError::Transit(format!("HTTP {}", resp.status())));
		}

		let json: serde_json::Value =
			resp.json().await.map_err(|e| SecretsError::Transit(e.to_string()))?;

		let b64 = json["data"]["plaintext"]
			.as_str()
			.ok_or_else(|| SecretsError::Transit("no plaintext in response".into()))?;

		B64.decode(b64).map_err(|e| SecretsError::Transit(e.to_string()))
	}

	/// Cycle a named encryption key (ADR-0016 KEK-per-cycle model).
	pub async fn cycle_kek(&self, cycle_id: &str) -> Result<(), SecretsError> {
		let client = reqwest::Client::builder()
			.use_rustls_tls()
			.build()
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		let key_name = format!("kek-{cycle_id}");
		// Create key if not exists
		let url = format!("{}/v1/{}/keys/{}", self.address, self.mount, key_name);
		let _ = client
			.post(&url)
			.header("X-Vault-Token", &self.token)
			.json(&serde_json::json!({"type": "aes256-gcm96"}))
			.send()
			.await
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		// Rotate
		let rotate_url =
			format!("{}/v1/{}/keys/{}/rotate", self.address, self.mount, key_name);
		let resp = client
			.post(&rotate_url)
			.header("X-Vault-Token", &self.token)
			.send()
			.await
			.map_err(|e| SecretsError::Transit(e.to_string()))?;

		if !resp.status().is_success() && resp.status().as_u16() != 204 {
			return Err(SecretsError::Transit(format!("rotate HTTP {}", resp.status())));
		}
		Ok(())
	}
}
