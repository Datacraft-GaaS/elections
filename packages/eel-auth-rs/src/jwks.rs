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

use crate::{error::AuthError, subject::Subject};
use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use moka::future::Cache;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Jwk {
    pub kid: String,
    pub kty: String,
    pub alg: Option<String>,
    pub n: Option<String>,
    pub e: Option<String>,
    pub x: Option<String>,
    pub y: Option<String>,
    pub crv: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct JwksResponse {
    pub keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeycloakClaims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub azp: Option<String>,
    pub aud: Option<AudValue>,
    pub sid: Option<String>,
    pub scope: Option<String>,
    pub realm_access: Option<RealmAccess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RealmAccess {
    pub roles: Vec<String>,
}

/// JWT `aud` may be a single string or an array; Keycloak uses both.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum AudValue {
    One(String),
    Many(Vec<String>),
}

impl AudValue {
    fn contains(&self, aud: &str) -> bool {
        match self {
            AudValue::One(a) => a == aud,
            AudValue::Many(xs) => xs.iter().any(|a| a == aud),
        }
    }
}

/// Cached JWKS verifier with automatic kid-miss refetch.
pub struct JwksVerifier {
    jwks_url: String,
    http: Client,
    cache: Cache<String, Arc<Jwk>>,
    realm: String,
    /// Expected `aud`/`azp` claim. When set, tokens minted for any other
    /// audience are rejected -- preventing cross-service replay of operator
    /// credentials. Unset preserves legacy behaviour (any audience).
    expected_audience: Option<String>,
}

impl JwksVerifier {
    pub fn new(jwks_url: impl Into<String>, realm: impl Into<String>) -> Self {
        Self::with_audience(jwks_url, realm, None)
    }

    /// Build a verifier pinned to one audience. Services SHOULD pass their
    /// own audience (e.g. from `EEL_OIDC_AUDIENCE`) so tokens issued for a
    /// different service are refused.
    pub fn with_audience(
        jwks_url: impl Into<String>,
        realm: impl Into<String>,
        audience: Option<String>,
    ) -> Self {
        let cache = Cache::builder()
            .max_capacity(64)
            .time_to_live(Duration::from_secs(300))
            .build();
        Self {
            jwks_url: jwks_url.into(),
            http: Client::builder()
                .use_rustls_tls()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("http client"),
            cache,
            realm: realm.into(),
            expected_audience: audience,
        }
    }

    async fn fetch_jwks(&self) -> Result<Vec<Jwk>, AuthError> {
        let resp = self
            .http
            .get(&self.jwks_url)
            .send()
            .await
            .map_err(|e| AuthError::JwksFetch(e.to_string()))?
            .json::<JwksResponse>()
            .await
            .map_err(|e| AuthError::JwksFetch(e.to_string()))?;
        Ok(resp.keys)
    }

    async fn get_key(&self, kid: &str) -> Result<Arc<Jwk>, AuthError> {
        if let Some(jwk) = self.cache.get(kid).await {
            return Ok(jwk);
        }
        let keys = self.fetch_jwks().await?;
        for k in &keys {
            let arc = Arc::new(k.clone());
            self.cache.insert(k.kid.clone(), arc).await;
        }
        self.cache
            .get(kid)
            .await
            .ok_or_else(|| AuthError::UnknownKid {
                kid: kid.to_owned(),
            })
    }

    pub async fn verify(&self, token: &str) -> Result<Subject, AuthError> {
        let header = decode_header(token).map_err(|e| AuthError::JwtVerification(e.to_string()))?;
        let kid = header
            .kid
            .ok_or_else(|| AuthError::JwtVerification("missing kid".into()))?;
        let jwk = self.get_key(&kid).await?;

        let decoding_key = build_decoding_key(&jwk)?;
        let mut validation = Validation::new(Algorithm::RS256);
        let issuer_base = self
            .jwks_url
            .replace("/.well-known/openid-configuration/jwks", "")
            .replace("/protocol/openid-connect/certs", "");
        validation.set_issuer(&[&issuer_base]);

        // Audience policy: when a pin is configured, require the token to
        // carry it; otherwise keep the historical any-audience behaviour.
        match self.expected_audience.as_deref() {
            Some(aud) => {
                validation.validate_aud = true;
                validation.set_audience(&[aud]);
            }
            None => validation.validate_aud = false,
        }

        let token_data =
            decode::<KeycloakClaims>(token, &decoding_key, &validation).map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::JwtVerification(e.to_string()),
                }
            })?;

        if let Some(aud) = self.expected_audience.as_deref() {
            // Keycloak service accounts may omit aud and rely on azp; accept
            // azp == pinned audience as an alternative proof of intent.
            let azp_ok = token_data.claims.azp.as_deref() == Some(aud);
            let aud_ok = token_data
                .claims
                .aud
                .as_ref()
                .map(|a| a.contains(aud))
                .unwrap_or(false);
            if !aud_ok && !azp_ok {
                return Err(AuthError::JwtVerification(format!(
                    "token audience mismatch: expected {aud}"
                )));
            }
        }

        let claims = token_data.claims;
        let roles = claims.realm_access.map(|r| r.roles).unwrap_or_default();
        let purpose_scopes = claims
            .scope
            .map(|s| s.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_default();

        Ok(Subject {
            sub: claims.sub,
            realm: self.realm.clone(),
            session_id: claims.sid,
            roles,
            purpose_scopes,
        })
    }
}

fn build_decoding_key(jwk: &Jwk) -> Result<DecodingKey, AuthError> {
    match jwk.kty.as_str() {
        "RSA" => {
            let n = jwk
                .n
                .as_deref()
                .ok_or_else(|| AuthError::JwksFetch("missing n".into()))?;
            let e = jwk
                .e
                .as_deref()
                .ok_or_else(|| AuthError::JwksFetch("missing e".into()))?;
            DecodingKey::from_rsa_components(n, e).map_err(|e| AuthError::JwksFetch(e.to_string()))
        }
        "EC" => {
            let x = jwk
                .x
                .as_deref()
                .ok_or_else(|| AuthError::JwksFetch("missing x".into()))?;
            let y = jwk
                .y
                .as_deref()
                .ok_or_else(|| AuthError::JwksFetch("missing y".into()))?;
            DecodingKey::from_ec_components(x, y).map_err(|e| AuthError::JwksFetch(e.to_string()))
        }
        kty => Err(AuthError::JwksFetch(format!("unsupported kty: {kty}"))),
    }
}
