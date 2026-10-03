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

use crate::error::SecretsError;
use chrono::{DateTime, Duration, Utc};
use tracing::info;
use uuid::Uuid;

const MAX_ADMIN_TTL_HOURS: i64 = 4;

/// JIT admin credentials with 4h cap enforced client-side.
#[derive(Debug, Clone)]
pub struct AdminCred {
    pub cred_id: String,
    pub username: String,
    pub password: String,
    pub role: String,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
}

/// Issues admin credentials from OpenBao with a hard 4h TTL cap.
pub async fn issue_admin_cred(
    openbao_address: &str,
    token: &str,
    role: &str,
    requested_ttl_hours: i64,
) -> Result<AdminCred, SecretsError> {
    let ttl = requested_ttl_hours.min(MAX_ADMIN_TTL_HOURS);
    if ttl <= 0 {
        return Err(SecretsError::AdminCapExceeded);
    }

    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| SecretsError::VaultClient(e.to_string()))?;

    let url = format!("{}/v1/database/creds/{}", openbao_address, role);
    let resp = client
        .get(&url)
        .header("X-Vault-Token", token)
        .send()
        .await
        .map_err(|e| SecretsError::VaultClient(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(SecretsError::VaultClient(format!("HTTP {}", resp.status())));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SecretsError::VaultClient(e.to_string()))?;

    let username = json["data"]["username"]
        .as_str()
        .ok_or_else(|| SecretsError::VaultClient("missing username".into()))?
        .to_owned();
    let password = json["data"]["password"]
        .as_str()
        .ok_or_else(|| SecretsError::VaultClient("missing password".into()))?
        .to_owned();

    let issued_at = Utc::now();
    let cred = AdminCred {
        cred_id: Uuid::new_v4().to_string(),
        username,
        password,
        role: role.to_owned(),
        issued_at,
        expires_at: issued_at + Duration::hours(ttl),
    };

    info!(
        cred_id = %cred.cred_id,
        role = %role,
        ttl_hours = ttl,
        expires_at = %cred.expires_at,
        "admin_cred_issued"
    );

    Ok(cred)
}
