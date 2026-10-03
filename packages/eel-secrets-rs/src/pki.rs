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
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeafCert {
    pub certificate: String,
    pub issuing_ca: String,
    pub private_key: String,
    pub serial_number: String,
    pub expires_at: DateTime<Utc>,
}

/// Request a leaf cert from OpenBao PKI mount.
pub async fn issue_leaf_cert(
    openbao_address: &str,
    token: &str,
    pki_mount: &str,
    pki_role: &str,
    common_name: &str,
    ttl: &str,
) -> Result<LeafCert, SecretsError> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .build()
        .map_err(|e| SecretsError::Pki(e.to_string()))?;

    let url = format!("{}/v1/{}/issue/{}", openbao_address, pki_mount, pki_role);
    let body = serde_json::json!({
        "common_name": common_name,
        "ttl": ttl,
    });

    let resp = client
        .post(&url)
        .header("X-Vault-Token", token)
        .json(&body)
        .send()
        .await
        .map_err(|e| SecretsError::Pki(e.to_string()))?;

    if !resp.status().is_success() {
        return Err(SecretsError::Pki(format!("HTTP {}", resp.status())));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| SecretsError::Pki(e.to_string()))?;

    let data = &json["data"];
    let expiration = data["expiration"].as_i64().unwrap_or(0);
    let expires_at =
        DateTime::from_timestamp(expiration, 0).unwrap_or_else(|| Utc::now() + Duration::hours(24));

    Ok(LeafCert {
        certificate: data["certificate"].as_str().unwrap_or("").to_owned(),
        issuing_ca: data["issuing_ca"].as_str().unwrap_or("").to_owned(),
        private_key: data["private_key"].as_str().unwrap_or("").to_owned(),
        serial_number: data["serial_number"].as_str().unwrap_or("").to_owned(),
        expires_at,
    })
}
