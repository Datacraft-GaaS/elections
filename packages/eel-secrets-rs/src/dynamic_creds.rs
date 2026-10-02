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

use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use crate::error::SecretsError;

/// Dynamic PostgreSQL credentials leased from OpenBao.
#[derive(Debug, Clone)]
pub struct PgCred {
	pub username: String,
	pub password: String,
	pub lease_id: String,
	pub lease_duration_secs: u64,
	pub expires_at: DateTime<Utc>,
}

/// Manages a PgPool backed by dynamic credentials, re-issuing on lease expiry.
pub struct PgDynamicCred {
	openbao_address: String,
	token: String,
	db_role: String,
	/// Template with `{{username}}` and `{{password}}` placeholders.
	db_url_template: String,
	state: Arc<RwLock<Option<(PgCred, PgPool)>>>,
}

impl PgDynamicCred {
	pub fn new(
		openbao_address: impl Into<String>,
		token: impl Into<String>,
		db_role: impl Into<String>,
		db_url_template: impl Into<String>,
	) -> Self {
		Self {
			openbao_address: openbao_address.into(),
			token: token.into(),
			db_role: db_role.into(),
			db_url_template: db_url_template.into(),
			state: Arc::new(RwLock::new(None)),
		}
	}

	/// Issue or renew credentials and return a live pool.
	pub async fn pool(&self) -> Result<PgPool, SecretsError> {
		{
			let guard = self.state.read().await;
			if let Some((ref cred, ref pool)) = *guard {
				if cred.expires_at > Utc::now() + Duration::seconds(60) {
					return Ok(pool.clone());
				}
			}
		}

		let cred = self.issue_creds().await?;
		let url = self
			.db_url_template
			.replace("{{username}}", &cred.username)
			.replace("{{password}}", &cred.password);

		let pool =
			PgPool::connect(&url).await.map_err(|e| SecretsError::Database(e.to_string()))?;

		let mut guard = self.state.write().await;
		*guard = Some((cred, pool.clone()));
		Ok(pool)
	}

	async fn issue_creds(&self) -> Result<PgCred, SecretsError> {
		let client = reqwest::Client::builder()
			.use_rustls_tls()
			.build()
			.map_err(|e| SecretsError::VaultClient(e.to_string()))?;

		let url = format!("{}/v1/database/creds/{}", self.openbao_address, self.db_role);
		let resp = client
			.get(&url)
			.header("X-Vault-Token", &self.token)
			.send()
			.await
			.map_err(|e| SecretsError::VaultClient(e.to_string()))?;

		if !resp.status().is_success() {
			return Err(SecretsError::VaultClient(format!("HTTP {}", resp.status())));
		}

		let json: serde_json::Value =
			resp.json().await.map_err(|e| SecretsError::VaultClient(e.to_string()))?;

		let username = json["data"]["username"]
			.as_str()
			.ok_or_else(|| SecretsError::VaultClient("missing username".into()))?
			.to_owned();
		let password = json["data"]["password"]
			.as_str()
			.ok_or_else(|| SecretsError::VaultClient("missing password".into()))?
			.to_owned();
		let lease_id = json["lease_id"].as_str().unwrap_or("").to_owned();
		let lease_duration_secs = json["lease_duration"].as_u64().unwrap_or(3600);

		Ok(PgCred {
			username,
			password,
			lease_id,
			lease_duration_secs,
			expires_at: Utc::now() + Duration::seconds(lease_duration_secs as i64),
		})
	}
}
