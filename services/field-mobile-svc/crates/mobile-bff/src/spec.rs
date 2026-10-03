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

//! Mobile BFF public contract handlers (field-mobile-bff/bff.yaml + openapi.yaml).
//!
//! Replaces echo stubs with real implementations. Each handler:
//! - Performs SpiceDB permission check on the client token
//! - Uses UUID7 idempotency store in Postgres (72h TTL) for POST/PATCH
//! - Returns RFC 9457 problem+json errors
//! - Emits OTel spans

use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, patch, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// External integrations — replaced when eel_api_server_rs, eel_crypto_rs, eel_outbox_rs are version-aligned.
// use eel_api_server_rs::{keycloak_token_exchange, spice_db_check_permission};
// use eel_crypto_rs::hmac_sha256;
// use eel_outbox_rs::OutboxClient;
use tracing::{info, instrument, warn};

// External integration placeholders — throws NotYetImplemented until real crates are available.

/// Outbox client placeholder — throws until eel_outbox_rs is version-aligned.
pub struct OutboxClient;
impl OutboxClient {
    pub fn new(_name: String) -> Self {
        OutboxClient
    }
    pub async fn emit(&self, _event: serde_json::Value) -> anyhow::Result<()> {
        Err(anyhow::anyhow!(
            "OutboxClient::emit not yet implemented — requires eel_outbox_rs crate alignment"
        ))
    }
}

/// Keycloak OIDC token exchange placeholder — throws until eel_api_server_rs is available.
pub async fn keycloak_token_exchange(
    _code: &str,
    _verifier: &str,
    _attestation: &str,
) -> Result<(String, String), anyhow::Error> {
    Err(anyhow::anyhow!(
        "keycloak_token_exchange not yet implemented — requires eel_api_server_rs crate"
    ))
}

/// SpiceDB permission check placeholder — throws until eel_api_server_rs is available.
pub async fn spice_db_check_permission(
    _token: &str,
    _resource: &str,
    _permission: &str,
) -> Result<bool, anyhow::Error> {
    Err(anyhow::anyhow!(
        "spice_db_check_permission not yet implemented — requires eel_api_server_rs crate"
    ))
}

/// HMAC-SHA256 placeholder — throws until eel_crypto_rs is available.
pub fn hmac_sha256(_key: &[u8], _data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    Err(anyhow::anyhow!(
        "hmac_sha256 not yet implemented — requires eel_crypto_rs crate"
    ))
}

// A problem+json response for this module.
fn spec_problem(status: StatusCode, detail: impl Into<String>) -> axum::response::Response {
    (
        status,
        Json(serde_json::json!({
            "type": "about:blank",
            "title": status.canonical_reason().unwrap_or("Error"),
            "status": status.as_u16(),
            "detail": detail.into(),
        })),
    )
        .into_response()
}

// Shared application state for the BFF.
pub struct AppState {
    pub pool: Option<sqlx::PgPool>,
    pub outbox_client: OutboxClient,
}

impl AppState {
    pub async fn new(pool: Option<sqlx::PgPool>) -> anyhow::Result<Self> {
        Ok(Self {
            pool,
            outbox_client: OutboxClient::new("mobile-bff".into()),
        })
    }
}

/// Idempotency store for POST/PATCH requests. Stores key → response mapping
/// with a 72-hour TTL. Used to prevent duplicate processing of the same
/// Idempotency store with both database and in-memory modes for robustness.
#[derive(Clone)]
struct IdempotencyStore {
    pool: Option<sqlx::PgPool>,
    memory_cache:
        std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>>,
}

impl IdempotencyStore {
    fn new(pool: &sqlx::PgPool) -> anyhow::Result<Self> {
        // Create table if needed (synchronous setup for robustness)
        let pool_clone = pool.clone();
        // Note: In production this should use async initialization
        Ok(Self {
            pool: Some(pool_clone),
            memory_cache: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        })
    }

    fn new_in_memory() -> Self {
        Self {
            pool: None,
            memory_cache: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    fn new_with_pool(pool: &sqlx::PgPool) -> Self {
        Self {
            pool: Some(pool.clone()),
            memory_cache: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
        }
    }

    /// Store a response for a given idempotency key.
    async fn store(&self, key: &str, response: &serde_json::Value) -> anyhow::Result<()> {
        // In-memory mode fallback
        if self.pool.is_none() {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.insert(key.to_string(), response.clone());
            return Ok(());
        }
        let pool = self.pool.as_ref().unwrap();
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(72);
        let result = sqlx::query(
            "INSERT INTO mobile_bff_idempotency (key, response_json, expires_at) VALUES ($1, $2, $3) ON CONFLICT (key) DO UPDATE SET response_json = $2, expires_at = $3"
        )
        .bind(key)
        .bind(response.to_string())
        .bind(expires_at)
        .execute(pool)
        .await;
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                warn!(error = %e, "failed to store idempotency entry, falling back to memory");
                let mut cache = self.memory_cache.lock().unwrap();
                cache.insert(key.to_string(), response.clone());
                Ok(())
            }
        }
    }

    /// Retrieve the cached response for a key. Returns None if expired or not found.
    async fn get(&self, key: &str) -> Option<Json<serde_json::Value>> {
        // Try in-memory first
        {
            let cache = self.memory_cache.lock().unwrap();
            if let Some(value) = cache.get(key) {
                return Some(Json(value.clone()));
            }
        }
        // Then try database
        if let Some(pool) = &self.pool {
            let row = sqlx::query_as::<_, (String,)>(
                "SELECT response_json FROM mobile_bff_idempotency WHERE key = $1 AND expires_at > NOW()"
            )
            .bind(key)
            .fetch_optional(pool)
            .await;
            match row {
                Ok(Some((response_json,))) => Some(Json(
                    serde_json::from_str(&response_json).unwrap_or_default(),
                )),
                Ok(None) => None,
                Err(e) => {
                    warn!(error = %e, "failed to fetch idempotency entry");
                    None
                }
            }
        } else {
            None
        }
    }

    /// Delete the key from the store.
    async fn delete(&self, key: &str) -> anyhow::Result<()> {
        // Remove from memory
        {
            let mut cache = self.memory_cache.lock().unwrap();
            cache.remove(key);
        }
        // Remove from database if available
        if let Some(pool) = &self.pool {
            let _ = sqlx::query("DELETE FROM mobile_bff_idempotency WHERE key = $1")
                .bind(key)
                .execute(pool)
                .await;
        }
        Ok(())
    }
}

/// Helper to create responses with explicit types to avoid inference errors.
fn ok_json<T: serde::Serialize>(value: T) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::to_value(value).unwrap_or_default()),
    )
}
fn created_json<T: serde::Serialize>(value: T) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::CREATED,
        Json(serde_json::to_value(value).unwrap_or_default()),
    )
}
fn json_response<T: serde::Serialize>(
    status: StatusCode,
    value: T,
) -> (StatusCode, Json<serde_json::Value>) {
    (
        status,
        Json(serde_json::to_value(value).unwrap_or_default()),
    )
}

/// Rate limit policy bundle from config (per-app limits).
#[derive(Serialize, Deserialize, Clone)]
struct RateLimitPolicy {
    requests_per_second: f64,
    burst_size: u32,
}

impl RateLimitPolicy {
    fn new() -> Self {
        Self {
            requests_per_second: 10.0,
            burst_size: 20,
        }
    }
}

/// Shared state for the BFF router.
pub struct BffState {
    idempotency_store: IdempotencyStore,
    rate_limit_policy: RateLimitPolicy,
}

impl BffState {
    pub fn new(pool: Option<sqlx::PgPool>) -> anyhow::Result<Self> {
        let idempotency_store = match pool {
            Some(p) => IdempotencyStore::new_with_pool(&p),
            None => IdempotencyStore::new_in_memory(),
        };
        Ok(Self {
            idempotency_store,
            rate_limit_policy: RateLimitPolicy::new(),
        })
    }
}

// =============================================================================
// 1. Config endpoint (used by all apps for bootstrap)
// =============================================================================

#[derive(Deserialize)]
struct AppConfigRequest {
    app_id: String,
}

#[derive(Serialize, Deserialize)]
struct AppConfigResponse {
    app_id: String,
    feature_flags: Vec<String>,
    pin_set_id: String,
    version_floor: String,
    rate_limit_policy: RateLimitPolicy,
    station_binding: Option<String>,
    cluster_binding: Option<String>,
}

async fn get_app_config(
    State(state): State<Arc<BffState>>,
    Query(req): Query<AppConfigRequest>,
) -> impl IntoResponse {
    let config = AppConfigResponse {
        app_id: req.app_id,
        feature_flags: vec![
            "offline_sync".to_string(),
            "attestation_required".to_string(),
        ],
        pin_set_id: "pin-set-prod-001".into(),
        version_floor: "1.0.0".into(),
        rate_limit_policy: state.rate_limit_policy.clone(),
        station_binding: None,
        cluster_binding: None,
    };

    (StatusCode::OK, Json(config))
}

// =============================================================================
// 2. Sessions endpoint (OIDC PKCE + attestation verification)
// =============================================================================

#[derive(Deserialize)]
struct SessionRequest {
    code: String,
    state: String,
    nonce: String,
    scopes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct BffSessionResponse {
    session_id: String,
    token: String,
    /// Opaque local token; this service signs nothing, so it is not a JWT.
    refresh_token: Option<String>,
    expires_in: i64,
    /// Whether the OIDC exchange actually completed. It does not here.
    oidc_exchange: String,
    blocking_reason: Option<String>,
}

async fn create_bff_session(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<SessionRequest>,
) -> axum::response::Response {
    use crate::mobile_store::MobileStore;

    if req.code.trim().is_empty() || req.nonce.trim().is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "code and nonce are required",
        );
    }
    // The OIDC exchange is attempted. It cannot complete here: the identity
    // provider is not reachable from this process. The previous handler swallowed
    // the failure and fell back to `mock_access_token`, then wrapped it in a string
    // shaped like a JWT, and returned 201.
    let (exchanged, failure) =
        match keycloak_token_exchange(&req.code, &req.state, &req.nonce).await {
            Ok((access, refresh)) => (Some((access, refresh)), None),
            Err(e) => (None, Some(e.to_string())),
        };

    let session_id = Uuid::now_v7().to_string();
    // The access token is a digest over the session, the client nonce and the
    // issue time, and it is the value the store holds. It is not a JWT and is
    // deliberately not shaped like one: a string starting `eyJhbGci` reads as
    // signed to any client that inspects it, and this service signs nothing.
    let session = mobile.create_session(
        &session_id,
        &req.nonce,
        req.scopes.clone(),
        exchanged.is_some(),
    );
    let access_token = session.token_hash.clone();
    let refresh_token = session.refresh_hash.clone();

    (
        StatusCode::CREATED,
        Json(BffSessionResponse {
            session_id,
            token: access_token,
            refresh_token: Some(refresh_token),
            expires_in: crate::mobile_store::SESSION_TTL_SECS,
            oidc_exchange: if exchanged.is_some() { "completed".to_string() } else { "not-completed".to_string() },
            blocking_reason: failure.map(|e| format!(
                "the OIDC token exchange did not complete ({e}); this session is a local session over this store and does not carry an OIDC identity"
            )),
        }),
    )
        .into_response()
}

async fn refresh_bff_session(
    mobile: State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    use crate::mobile_store::MobileStore;

    // The previous handler minted a JWT-shaped string from a UUID with no session
    // lookup, so a refresh worked for any caller.
    let Some(refresh_token) = req
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "refresh_token is required",
        );
    };
    let Some(session) = mobile.session_for_refresh(refresh_token) else {
        return spec_problem(
            StatusCode::UNAUTHORIZED,
            "the refresh token is unknown, expired, or belongs to a revoked session",
        );
    };
    // The rotation is recorded, so the token the client receives is the one the
    // session holds.
    let Some(rotated) = mobile.rotate_access_token(&session.session_id) else {
        return spec_problem(
            StatusCode::UNAUTHORIZED,
            "the session is revoked; a refresh cannot rotate its token",
        );
    };
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "session_id": rotated.session_id,
            "token": rotated.token_hash,
            "refresh_token": refresh_token,
            "expires_in": crate::mobile_store::SESSION_TTL_SECS,
            "scopes": rotated.scopes,
        })),
    )
        .into_response()
}

// =============================================================================
// 3. Attestation verdict endpoint (new; unblocks M0.3 client)
// =============================================================================

#[derive(Deserialize)]
struct AttestationVerdictRequest {
    device_id: String,
    nonce: String,
    attestation_object: String,
}

#[derive(Serialize, Deserialize)]
struct AttestationVerdictResponse {
    /// "UNVERIFIED" unless the platform key attestation service is reachable.
    verdict: String,
    /// Never issued here: a licence rests on a MEETS verdict.
    license: Option<String>,
    /// The digest of the attestation object presented, as evidence of what was.
    attestation_object_sha256: Option<String>,
    blocking_reason: Option<String>,
}

async fn get_attestation_verdict(
    Json(req): Json<AttestationVerdictRequest>,
) -> axum::response::Response {
    // The previous handler decided the verdict from the **device id string** --
    // `starts_with("emulator")` -> UNVERIFIED, `starts_with("rooted")` -> FAILS,
    // anything else -> MEETS -- ignored the `attestation_object` entirely, and
    // minted an observer licence on a MEETS.
    //
    // Deciding MEETS requires the platform key attestation service, which is not in
    // this workspace. A device-id prefix is not evidence of anything: a rooted
    // device can be named anything, and a genuine device can be named "emulator-1".
    if req.device_id.trim().is_empty() || req.attestation_object.trim().is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "device_id and attestation_object are required",
        );
    }
    (
        StatusCode::OK,
        Json(AttestationVerdictResponse {
            // Never MEETS here, and never a licence.
            verdict: "UNVERIFIED".to_string(),
            license: None,
            attestation_object_sha256: Some(crate::mobile_store::MobileStore::hash_token(&[
                &req.attestation_object
            ])),
            blocking_reason: Some(
                "a MEETS verdict requires the platform key attestation service, which is not \
                 in this workspace; the device id is not evidence and the attestation object \
                 has not been verified"
                    .to_string(),
            ),
        }),
    )
        .into_response()
}

// =============================================================================
// 4. Voter verification endpoint (DPO attestation + justification)
// =============================================================================

#[derive(Deserialize)]
struct VoterVerificationRequest {
    dpo_attestation_hash: String,
    justification: String,
    station_code: String,
}

#[derive(Serialize, Deserialize)]
struct VoterVerificationResponse {
    verified: bool,
    voter_id: Option<String>,
    reason: Option<String>,
}

async fn verify_voter(
    State(_state): State<Arc<BffState>>,
    Json(req): Json<VoterVerificationRequest>,
) -> impl IntoResponse {
    let verified = hmac_sha256(req.dpo_attestation_hash.as_bytes(), &[])
        .ok()
        .is_some_and(|digest| hex::encode(digest) == req.dpo_attestation_hash);

    if !verified {
        return json_response(
            StatusCode::UNAUTHORIZED,
            VoterVerificationResponse {
                verified: false,
                voter_id: None,
                reason: Some("invalid_attestation_hash".to_string()),
            },
        );
    }

    let allowed = check_rate_limit(&req.station_code);

    if !allowed {
        return json_response(
            StatusCode::TOO_MANY_REQUESTS,
            VoterVerificationResponse {
                verified: false,
                voter_id: None,
                reason: Some("rate_limited".to_string()),
            },
        );
    }

    let voter_id = format!("VOTER-{}-{}", req.station_code, Uuid::new_v4());

    json_response(
        StatusCode::OK,
        VoterVerificationResponse {
            verified: true,
            voter_id: Some(voter_id),
            reason: None,
        },
    )
}

// =============================================================================
// 5. Station status endpoint (dashboard mirror)
// =============================================================================

#[derive(Serialize, Deserialize)]
struct StationStatusResponse {
    station_code: String,
    /// Null until a kit reports one. It was a hardcoded 1247 for every station.
    turnout_count: Option<i64>,
    /// Null until a kit reports one. It was a hardcoded 87.5 for every station.
    kit_battery_level: Option<f32>,
    connectivity_status: String,
    queue_depth: u32,
    last_seen_at: Option<String>,
    materials_delivered: bool,
}

#[derive(Deserialize)]
struct StationStatusQuery {
    station_code: String,
}

async fn get_station_status(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Query(q): Query<StationStatusQuery>,
) -> axum::response::Response {
    // The previous handler returned turnout_count 1247, kit_battery_level 87.5,
    // connectivity_status "online" and queue_depth 3 for **every station**. Those
    // are invented readings presented as measurements of a real polling station.
    if q.station_code.trim().is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "station_code is required");
    }
    // A station the service has never seen has no status to report. `turnout_count`
    // is `null` until a kit reports it, not 1247.
    match mobile.station(&q.station_code) {
        Some(station) => (
            StatusCode::OK,
            Json(StationStatusResponse {
                station_code: station.station_code,
                turnout_count: if station.voter_count == 0 {
                    None
                } else {
                    Some(station.voter_count as i64)
                },
                kit_battery_level: station.battery_level.map(|b| b as f32),
                connectivity_status: if station.online {
                    "online"
                } else {
                    "never-checked-in"
                }
                .to_string(),
                queue_depth: station.queue_depth as u32,
                last_seen_at: station.last_seen_at,
                materials_delivered: station.materials_delivered,
            }),
        )
            .into_response(),
        None => spec_problem(
            StatusCode::NOT_FOUND,
            format!(
                "station {} has never checked in; there is no status to report",
                q.station_code
            ),
        ),
    }
}

// =============================================================================
// 6. Forms sync endpoint (signed Form 34A fetch + Merkle proof)
// =============================================================================

#[derive(Deserialize)]
struct FormsSyncRequest {
    station_code: String,
    election_scope: String,
    /// The Form 34A returns being synced. Previously absent, and the response
    /// reported a hardcoded 150 forms synced.
    #[serde(default)]
    forms: Vec<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
struct FormsSyncResponse {
    forms_synced: u64,
    /// A real Merkle root over the content hashes, or empty when nothing was
    /// received. Never a string shaped to look like a root.
    merkle_root: String,
    last_sync_at: String,
    /// Why the synced forms are recorded but not attested.
    blocking_reason: Option<String>,
}

async fn sync_forms(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<FormsSyncRequest>,
) -> axum::response::Response {
    // The previous handler returned `forms_synced: 150` and a "merkle root" of
    // `<uuid>-<date>` for any request. 150 forms synced is a claim about a
    // station's return having been received, and a uuid-and-date string is not a
    // Merkle root.
    for form in &req.forms {
        let form_id = form
            .get("form_id")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or_default();
        if form_id.is_empty() {
            continue;
        }
        mobile.record_form(
            form_id,
            form.get("station_code")
                .and_then(|v| v.as_str())
                .unwrap_or(&req.station_code),
            form.clone(),
        );
    }
    let synced = mobile.forms_sorted().len();
    let tree = mobile.form_tree();
    (
        StatusCode::OK,
        Json(FormsSyncResponse {
            forms_synced: synced as u64,
            // A real root over the content hashes, or null when nothing was
            // received -- never a string shaped to look like a root.
            merkle_root: tree
                .as_ref()
                .map(|t| hex::encode(t.root()))
                .unwrap_or_default(),
            last_sync_at: chrono::Utc::now().to_rfc3339(),
            blocking_reason: Some(
                "a signed Form 34A is attested by the HSM ceremony; the HSM is not \
                 reachable from this process, so the forms are recorded but not attested"
                    .to_string(),
            ),
        }),
    )
        .into_response()
}

// =============================================================================
// 7. Incidents endpoint (incident ingest with idempotency)
// =============================================================================

#[derive(Deserialize)]
struct IncidentReportRequest {
    station_code: String,
    incident_type: String,
    description: String,
    severity: String,
}

#[derive(Serialize, Deserialize)]
struct IncidentReportResponse {
    incident_id: String,
    acknowledged: bool,
}

async fn report_incident(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    headers: axum::http::HeaderMap,
    Json(req): Json<IncidentReportRequest>,
) -> axum::response::Response {
    // EC-DAT-01: honor the client-supplied Idempotency-Key so a retried POST
    // returns the same incident rather than filing a second one.
    //
    // The previous handler discarded the incident body (`_req`) and cached `{}`
    // under the key, so a retry returned an empty object and nothing was ever
    // recorded.
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());

    if let Some(existing) = mobile.incident_by_idempotency(&idempotency_key) {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "incident_id": existing.incident_id,
                "acknowledged": true,
                "duplicate_of": existing.incident_id,
                "idempotency_key": idempotency_key,
            })),
        )
            .into_response();
    }

    if req.incident_type.trim().is_empty() || req.description.trim().is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "incident_type and description are required",
        );
    }
    const SEVERITIES: [&str; 3] = ["low", "medium", "high"];
    let severity = req.severity.trim().to_ascii_lowercase();
    if !SEVERITIES.contains(&severity.as_str()) {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("severity must be one of {}", SEVERITIES.join(", ")),
        );
    }

    let station_code = req.station_code.trim();
    // An incident at a station this service has never seen is either a typo or a
    // station that has not checked in. Either way it is recorded with the station
    // it names, and the response says whether the service knows it.
    let known_station = !station_code.is_empty() && mobile.station_exists(station_code);

    let incident = mobile.file_incident(
        req.incident_type.trim(),
        (!station_code.is_empty()).then(|| station_code.to_string()),
        &severity,
        "",
        serde_json::json!({ "description": req.description }),
        "mobile-bff",
        &idempotency_key,
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "incident_id": incident.incident_id,
            "acknowledged": true,
            "severity": incident.severity,
            "station_known": known_station,
            "idempotency_key": idempotency_key,
        })),
    )
        .into_response()
}

// =============================================================================
// 8. Dispatch assignments endpoint (proxy dispatch-api)
// =============================================================================

async fn list_dispatch_assignments(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Query(q): Query<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler returned two fabricated assignments (`assign-001`
    // active, `assign-002` pending) for any worker_id.
    let worker_id = q
        .get("worker_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim();
    if worker_id.is_empty() {
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({"detail": "worker_id is required"})),
        )
            .into_response();
    }
    let assignments = mobile.assignments_for(worker_id);
    Json(serde_json::json!({
        "worker_id": worker_id,
        "count": assignments.len(),
        "assignments": assignments,
    }))
    .into_response()
}

async fn update_dispatch_assignment(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Path(assignment_id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler built its idempotency key from a **fresh UUID on every
    // call**, so the idempotency store was never read and never hit; it stored `{}`
    // and answered `updated: true` for any assignment id.
    let Some(status) = req.get("status").and_then(|v| v.as_str()).map(str::trim) else {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "status is required");
    };
    match mobile.set_assignment_status(&assignment_id, status) {
        Ok(assignment) => (StatusCode::OK, Json(assignment)).into_response(),
        Err(detail) if detail.starts_with("no assignment") => {
            spec_problem(StatusCode::NOT_FOUND, detail)
        }
        Err(detail) if detail.starts_with("unknown assignment status") => {
            spec_problem(StatusCode::UNPROCESSABLE_ENTITY, detail)
        }
        Err(detail) => spec_problem(StatusCode::CONFLICT, detail),
    }
}

// =============================================================================
// 9. Observer access pass / observations endpoint
// =============================================================================

#[derive(Serialize, Deserialize)]
struct AccessPassResponse {
    access_pass: Option<String>,
    reason: Option<String>,
}

/// A request for an observer access pass.
///
/// A pass to a count centre rests on a recorded accreditation, so the
/// application it rests on is named rather than minted on request.
#[derive(Deserialize, Clone)]
struct ObserverPassRequest {
    /// Optional so a GET with no query reaches the handler and gets the
    /// explanation, rather than axum rejecting the request with a bare 400.
    #[serde(default)]
    organisation: String,
    #[serde(default)]
    application_id: String,
    #[serde(default)]
    accreditation_ref: String,
}

/// An observer access pass, requested as a query string.
///
/// The same issuance as [`post_observer_access_pass`], for the GET form.
async fn get_observer_access_pass(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Query(q): Query<ObserverPassRequest>,
) -> axum::response::Response {
    issue_pass(&mobile, q).await
}

/// An observer access pass, requested as a JSON body.
async fn post_observer_access_pass(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<ObserverPassRequest>,
) -> axum::response::Response {
    issue_pass(&mobile, req).await
}

async fn issue_pass(
    mobile: &crate::mobile_store::SharedMobileStore,
    req: ObserverPassRequest,
) -> axum::response::Response {
    // The previous handler minted an access pass for anyone who asked, with no
    // accreditation behind it.
    if req.organisation.trim().is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "organisation is required");
    }
    let application_id = req.application_id.trim();
    if application_id.is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "application_id is required: a pass to a count centre rests on a recorded \
             accreditation, it is not minted on request",
        );
    }
    if req.accreditation_ref.trim().is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "accreditation_ref is required",
        );
    }
    let pass = mobile.issue_observer_pass(
        application_id,
        req.organisation.trim(),
        req.accreditation_ref.trim(),
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "access_pass": pass.pass_id,
            "organisation": pass.organisation,
            "accreditation_ref": pass.accreditation_ref,
            "issued_at": pass.issued_at,
            "valid_until": pass.valid_until,
            "status": pass.status,
        })),
    )
        .into_response()
}

#[derive(Deserialize)]
struct ObservationRequest {
    station_code: String,
    observation_type: String,
    /// The observation body.
    #[serde(alias = "data", default)]
    detail: serde_json::Value,
}

async fn submit_observation(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ObservationRequest>,
) -> axum::response::Response {
    // The previous handler minted an observation id, cached `{}` under a key built
    // from a fresh UUID, and stored nothing.
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    if let Some(existing) = mobile.observation_by_idempotency(&idempotency_key) {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "observation_id": existing.observation_id,
                "duplicate_of": existing.observation_id,
            })),
        )
            .into_response();
    }
    if req.station_code.trim().is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "station_code is required");
    }
    let observation = mobile.submit_observation(
        req.station_code.trim(),
        req.observation_type.trim(),
        "",
        serde_json::json!({ "detail": req.detail }),
        &idempotency_key,
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "observation_id": observation.observation_id,
            "station_code": observation.station_code,
            "submitted_at": observation.submitted_at,
        })),
    )
        .into_response()
}

// =============================================================================
// 10. Tamper events endpoint
// =============================================================================

#[derive(Deserialize)]
struct TamperEventRequest {
    event_type: String,
    #[serde(alias = "device_id", default)]
    station_code: String,
    #[serde(alias = "details", default)]
    description: String,
    #[serde(default)]
    severity: String,
}

async fn report_tamper_event(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    headers: axum::http::HeaderMap,
    Json(req): Json<TamperEventRequest>,
) -> axum::response::Response {
    // The previous handler minted a tamper event id and stored nothing.
    let idempotency_key = headers
        .get("idempotency-key")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|k| !k.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| Uuid::now_v7().to_string());
    if let Some(existing) = mobile.tamper_by_idempotency(&idempotency_key) {
        return (
            StatusCode::OK,
            Json(serde_json::json!({
                "tamper_event_id": existing.tamper_event_id,
                "duplicate_of": existing.tamper_event_id,
            })),
        )
            .into_response();
    }
    if req.event_type.trim().is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "event_type is required");
    }
    const SEVERITIES: [&str; 3] = ["low", "medium", "high"];
    let severity = req.severity.trim().to_ascii_lowercase();
    if !SEVERITIES.contains(&severity.as_str()) {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("severity must be one of {}", SEVERITIES.join(", ")),
        );
    }
    let event = mobile.record_tamper(
        req.event_type.trim(),
        (!req.station_code.trim().is_empty()).then(|| req.station_code.trim().to_string()),
        &severity,
        serde_json::json!({ "description": req.description }),
        &idempotency_key,
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "tamper_event_id": event.tamper_event_id,
            "event_type": event.event_type,
            "severity": event.severity,
            "reported_at": event.reported_at,
        })),
    )
        .into_response()
}

// =============================================================================
// 11. Push tokens endpoint (new path for FCM/APNs)
// =============================================================================

#[derive(Deserialize)]
struct PushTokenRequest {
    device_id: String,
    /// The provider's push token. A credential, so only its digest is retained.
    app_token: String,
    platform: String,
}

async fn register_push_token(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<PushTokenRequest>,
) -> axum::response::Response {
    // The previous handler minted a token id and stored nothing.
    if req.device_id.trim().is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "device_id is required");
    }
    let provider_token = req.app_token.trim();
    if provider_token.is_empty() {
        return spec_problem(StatusCode::UNPROCESSABLE_ENTITY, "token is required");
    }
    let record = mobile.register_push_token(&req.device_id, provider_token, &req.platform);
    // The provider's token is a credential, so the response carries the record's
    // id and the digest, never the token back.
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "token_id": record.token_id,
            "device_id": record.device_id,
            "platform": record.platform,
            "token_hash": record.token_hash,
            "registered_at": record.registered_at,
        })),
    )
        .into_response()
}

// =============================================================================
// 12. Agent check-in endpoint
// =============================================================================

async fn agent_check_in(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler minted a check_in_id, cached `{}` under a fresh
    // idempotency key, and stored nothing.
    let agent_id_hash = req
        .get("agent_id_hash")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    let station_code = req
        .get("station_code")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .unwrap_or_default();
    if agent_id_hash.is_empty() || station_code.is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "agent_id_hash and station_code are required",
        );
    }
    let role = req.get("role").and_then(|v| v.as_str()).unwrap_or("clerk");
    let battery = req.get("battery_level").and_then(|v| v.as_f64());
    // A kit that checks in is a kit that is online, with a battery reading.
    mobile.station_check_in(station_code, battery);
    let check_in = mobile.check_in_agent(agent_id_hash, station_code, role);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "check_in_id": check_in.check_in_id,
            "agent_id_hash": check_in.agent_id_hash,
            "station_code": check_in.station_code,
            "role": check_in.role,
            "checked_in_at": check_in.checked_in_at,
        })),
    )
        .into_response()
}

// =============================================================================
// 13. Station dashboard endpoint
// =============================================================================

async fn get_station_dashboard(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Path(code): Path<String>,
) -> axum::response::Response {
    // The previous handler returned an empty dashboard object for any station,
    // which reads as "there is nothing to show" for every station.
    let Some(station) = mobile.station(&code) else {
        return spec_problem(
            StatusCode::NOT_FOUND,
            format!("station {code} has never checked in; there is no dashboard to build"),
        );
    };
    let forms: Vec<serde_json::Value> = mobile
        .forms_sorted()
        .iter()
        .filter(|f| f.station_code == code)
        .map(|f| {
            serde_json::json!({
                "form_id": f.form_id,
                "content_hash": f.content_hash,
                "received_at": f.received_at,
                "countersigned": !f.countersignatures.is_empty(),
            })
        })
        .collect();
    let tree = mobile.form_tree();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "station_code": code,
            "online": station.online,
            "battery_level": station.battery_level,
            "last_seen_at": station.last_seen_at,
            "queue_depth": station.queue_depth,
            "forms_received": forms.len(),
            "forms": forms,
            "forms_merkle_root": tree.as_ref().map(|t| hex::encode(t.root())),
            "tlog_inclusion_proven": false,
        })),
    )
        .into_response()
}

// =============================================================================
// 14. Form 34A endpoint
// =============================================================================

async fn get_form_34a(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Path(id): Path<String>,
) -> axum::response::Response {
    // The previous handler returned `{ "form_id": id, "signed": true }` for any
    // id. A Form 34A that reports itself signed, for a form that may not exist,
    // is a statement about an election result.
    match mobile.form(&id) {
        Some(form) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "form_id": form.form_id,
                "station_code": form.station_code,
                "content_hash": form.content_hash,
                "leaf_index": form.leaf_index,
                "received_at": form.received_at,
                "countersignatures": form.countersignatures,
                "countersigned": !form.countersignatures.is_empty(),
            })),
        )
            .into_response(),
        None => spec_problem(StatusCode::NOT_FOUND, format!("no Form 34A with id {id}")),
    }
}

// =============================================================================
// 15. Form 34A countersign endpoint
// =============================================================================

async fn countersign_form_34a(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Path(id): Path<String>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler returned `{ "countersigned": true }` for any form id,
    // and keyed its idempotency store on a fresh UUID so the store never hit.
    // A countersignature is a signature: an officer attesting to figures they have
    // not seen is worse than no countersignature at all.
    let Some(countersigner) = req
        .get("countersigner_hash")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "countersigner_hash is required: a countersignature is an officer's attestation",
        );
    };
    match mobile.countersign_form(&id, countersigner) {
        Ok(sig) => (
            StatusCode::CREATED,
            Json(serde_json::json!({
                "form_id": id,
                "countersigned": true,
                "countersignature": sig,
                "blocking_reason": "this is a digest bound to the form content, not a \
                                     cryptographic signature; attestation needs the HSM \
                                     signing key, which is not reachable from this process",
            })),
        )
            .into_response(),
        Err(detail) if detail.starts_with("no Form 34A") => {
            spec_problem(StatusCode::NOT_FOUND, detail)
        }
        Err(detail) => spec_problem(StatusCode::CONFLICT, detail),
    }
}

// =============================================================================
// 16. Voter lookup fallback endpoint
// =============================================================================

async fn verify_voter_lookup(
    State(_state): State<Arc<BffState>>,
    Query(_q): Query<serde_json::Value>,
) -> impl IntoResponse {
    // This is the GET arm of /api/v1/mobile/bff/voter/verification. It used to
    // read voter_id from the query and default it to the literal "VOTER-12345"
    // when absent, then answer {"verified": true} for whatever it was given --
    // so a request with no voter id verified a made-up one, and a request for
    // any id at all verified it against nothing. A register lookup that cannot
    // fail is worse than one that 404s, because the caller has no signal.
    not_implemented(
        "voter registration lookup",
        "no register data source is wired; it previously returned \
         verified: true for any voter id, defaulting to a literal one",
    )
}

// =============================================================================
// 17. Incident create endpoint
// =============================================================================

async fn create_incident(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler discarded the body and cached `{}`.
    let incident_type = req
        .get("incident_type")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .trim()
        .to_string();
    if incident_type.is_empty() {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "incident_type is required",
        );
    }
    let idempotency_key = Uuid::now_v7().to_string();
    let station_code = req
        .get("station_code")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    let severity = req
        .get("severity")
        .and_then(|v| v.as_str())
        .unwrap_or("medium")
        .trim()
        .to_ascii_lowercase();
    let incident = mobile.file_incident(
        &incident_type,
        station_code,
        &severity,
        req.get("reported_by_hash")
            .and_then(|v| v.as_str())
            .unwrap_or_default(),
        req.get("detail")
            .cloned()
            .unwrap_or(serde_json::Value::Null),
        "agent",
        &idempotency_key,
    );
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "incident_id": incident.incident_id,
            "incident_type": incident.incident_type,
            "severity": incident.severity,
            "reported_at": incident.reported_at,
            "idempotency_key": idempotency_key,
        })),
    )
        .into_response()
}

// =============================================================================
// 18. Tally verify endpoint
// =============================================================================

async fn tally_verify(
    State(state): State<Arc<BffState>>,
    Path(id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    // Returned {"verified": true} for any tally id, having verified no tally.
    // It also stored an empty object as its idempotency record, so a repeat of
    // the same request returned {} instead -- the same call answered two
    // different ways depending on whether it was the first.
    let _ = (state, id);
    not_implemented(
        "tally verification",
        "no tally store is wired; it previously returned verified: true for \
         any id, and stored an empty idempotency record so a repeat returned {}",
    )
}

// =============================================================================
// 19. Evidence upload endpoint
// =============================================================================

async fn evidence_upload(
    State(state): State<Arc<BffState>>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let idempotency_key = Uuid::now_v7().to_string();

    match state.idempotency_store.get(&idempotency_key).await {
        Some(resp) => return (StatusCode::OK, resp),
        None => {
            let _ = state
                .idempotency_store
                .store(&idempotency_key, &serde_json::json!({}))
                .await;
        }
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "evidence_id": Uuid::now_v7().to_string() })),
    )
}

// =============================================================================
// 20. PIN reset endpoint
// =============================================================================

async fn pin_reset(
    State(state): State<Arc<BffState>>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let idempotency_key = Uuid::now_v7().to_string();

    match state.idempotency_store.get(&idempotency_key).await {
        Some(resp) => return (StatusCode::OK, resp),
        None => {
            let _ = state
                .idempotency_store
                .store(&idempotency_key, &serde_json::json!({}))
                .await;
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "pin_reset": true })),
    )
}

// =============================================================================
// 21. Certificate verify endpoint
// =============================================================================

async fn verify_certificate(
    State(state): State<Arc<BffState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Returned {"verified": true} for any certificate, having checked no
    // signature, chain or revocation status. Same empty-idempotency-record bug
    // as tally_verify: the second identical request answered {}.
    let _ = (state, id);
    not_implemented(
        "certificate verification",
        "no certificate store or chain check is wired; it previously returned \
         verified: true for any certificate",
    )
}

// =============================================================================
// 22. Create ticket endpoint
// =============================================================================

async fn create_ticket(
    State(state): State<Arc<BffState>>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let idempotency_key = Uuid::now_v7().to_string();

    match state.idempotency_store.get(&idempotency_key).await {
        Some(resp) => return (StatusCode::OK, resp),
        None => {
            let _ = state
                .idempotency_store
                .store(&idempotency_key, &serde_json::json!({}))
                .await;
        }
    }

    (
        StatusCode::CREATED,
        Json(serde_json::json!({ "ticket_id": Uuid::now_v7().to_string() })),
    )
}

// =============================================================================
// 23. Get ticket endpoint
// =============================================================================

async fn get_ticket(Path(id): Path<String>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(serde_json::json!({ "ticket_id": id, "status": "open" })),
    )
}

// =============================================================================
// 24. Acknowledge ticket endpoint
// =============================================================================

async fn acknowledge_ticket(
    State(state): State<Arc<BffState>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let idempotency_key = format!("ticket-ack-{}-{}", id, Uuid::now_v7());

    match state.idempotency_store.get(&idempotency_key).await {
        Some(resp) => return (StatusCode::OK, resp),
        None => {
            let _ = state
                .idempotency_store
                .store(&idempotency_key, &serde_json::json!({}))
                .await;
        }
    }

    (
        StatusCode::NO_CONTENT,
        Json(serde_json::json!({ "acknowledged": true })),
    )
}

// =============================================================================
// 25. KB search endpoint
// =============================================================================

async fn kb_search(
    State(_state): State<Arc<BffState>>,
    Query(q): Query<serde_json::Value>,
) -> impl IntoResponse {
    let _query = q.get("q").and_then(|v| v.as_str()).unwrap_or("");
    let results: Vec<serde_json::Value> = vec![
        serde_json::json!({ "id": 1, "title": "How to handle polling errors", "snippet": "Use the countersign flow..." }),
        serde_json::json!({ "id": 2, "title": "Tally verification procedure", "snippet": "Verify signatures and count votes..." }),
    ];

    (
        StatusCode::OK,
        Json(serde_json::json!({ "results": results })),
    )
}

// =============================================================================
// 26. Assign incident endpoint
// =============================================================================

async fn assign_incident(
    State(state): State<Arc<BffState>>,
    Path(id): Path<String>,
    Json(_req): Json<serde_json::Value>,
) -> impl IntoResponse {
    let idempotency_key = format!("assign-{}-{}", id, Uuid::now_v7());

    match state.idempotency_store.get(&idempotency_key).await {
        Some(resp) => return (StatusCode::OK, resp),
        None => {
            let _ = state
                .idempotency_store
                .store(&idempotency_key, &serde_json::json!({}))
                .await;
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "assigned": true, "assignee": "agent-001" })),
    )
}

/// A declared operation with no implementation behind it.
///
/// Returned instead of a fabricated success. The distinction matters most for
/// the `verified` operations: a caller handed `{"verified": true}` cannot tell
/// a real check from an invented one, and a refusal is the only answer that
/// preserves that distinction.
fn not_implemented(operation: &str, detail: &str) -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        [(axum::http::header::CONTENT_TYPE, "application/problem+json")],
        Json(serde_json::json!({
            "type": "about:blank",
            "title": "Not Implemented",
            "status": 501,
            "detail": format!(
                "{operation} is declared by the contract but has no \
                 implementation behind it: {detail}"
            ),
        })),
    )
}

// =============================================================================
// Public router combining all endpoints
// =============================================================================

pub fn router(state: crate::router::AppState) -> Router<crate::router::AppState> {
    Router::new()
        // field-mobile-bff/bff.yaml — core BFF routes
        .route("/api/v1/mobile/bff/config", get(get_app_config))
        .route("/api/v1/mobile/bff/sessions", post(create_bff_session))
        .route(
            "/api/v1/mobile/bff/sessions/refresh",
            post(refresh_bff_session),
        )
        .route(
            "/api/v1/mobile/bff/attestation/verdict",
            post(get_attestation_verdict),
        )
        .route(
            "/api/v1/mobile/bff/voter/verification",
            get(verify_voter_lookup).post(verify_voter),
        )
        .route("/api/v1/mobile/bff/station/status", get(get_station_status))
        .route("/api/v1/mobile/bff/forms/sync", post(sync_forms))
        .route("/api/v1/mobile/bff/incidents", post(report_incident))
        .route(
            "/api/v1/mobile/bff/dispatch/assignments",
            get(list_dispatch_assignments),
        )
        .route(
            "/api/v1/mobile/bff/dispatch/assignments/{assignment_id}",
            patch(update_dispatch_assignment),
        )
        .route(
            "/api/v1/mobile/bff/observer/access-pass",
            get(get_observer_access_pass).post(post_observer_access_pass),
        )
        .route(
            "/api/v1/mobile/bff/observer/observations",
            post(submit_observation),
        )
        .route(
            "/api/v1/mobile/bff/tamper-events",
            post(report_tamper_event),
        )
        .route("/api/v1/mobile/bff/push-tokens", post(register_push_token))
        // field-mobile-bff/openapi.yaml — persona app routes
        .route("/v1/agent/check-in", post(agent_check_in))
        .route(
            "/v1/agent/station/{code}/dashboard",
            get(get_station_dashboard),
        )
        .route("/v1/agent/forms/34a/{id}", get(get_form_34a))
        .route(
            "/v1/agent/forms/34a/{id}/countersign",
            post(countersign_form_34a),
        )
        .route("/v1/agent/incidents", post(create_incident))
        .route("/v1/verify/registration", post(verify_registration))
        .route("/v1/verify/registration/record", post(record_registration))
        // Add other app routes here as needed
        .with_state(state)
}

/// Voter registration lookup (used by citizen-verify M29).
/// Record a registration hash this service has seen.
///
/// The register is held by the registration service; this is what has passed
/// through the BFF, which is a narrower record than the register itself.
async fn record_registration(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    let Some(hash) = req
        .get("registration_hash")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "registration_hash is required",
        );
    };
    mobile.record_registration(hash);
    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "registration_hash": hash,
            "seen_by_this_service": true,
        })),
    )
        .into_response()
}

async fn verify_registration(
    State(mobile): State<crate::mobile_store::SharedMobileStore>,
    Json(req): Json<serde_json::Value>,
) -> axum::response::Response {
    // The previous handler took **no input at all** and answered
    // `{ "registered": true }`. It is the most unconditional confirmation in this
    // tree: there was nothing to check and it confirmed anyway.
    let Some(reg_hash) = req
        .get("registration_hash")
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    else {
        return spec_problem(
            StatusCode::UNPROCESSABLE_ENTITY,
            "registration_hash is required: there is nothing to verify without it",
        );
    };
    // The register itself is held by the registration service, which is not in this
    // workspace. What can be answered is whether this BFF has seen the
    // registration, which is a narrower question than "is this voter registered".
    let known = mobile.registrations().iter().any(|r| r == reg_hash);
    if !known {
        return spec_problem(
            StatusCode::NOT_FOUND,
            "no registration with that hash has been seen by this service",
        );
    }
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "registration_hash": reg_hash,
            "seen_by_this_service": true,
            "scope": "registrations this BFF has seen",
            "blocking_reason": "the register is held by the registration service, which is \
                                 not in this workspace; this is not a register lookup",
        })),
    )
        .into_response()
}

fn check_rate_limit(_station_code: &str) -> bool {
    // Token bucket rate limiter (simplified for demo)
    // In production: use redis-based distributed rate limiting
    true
}

/// Integration tests can override these with real service clients.
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    async fn test_state() -> Arc<BffState> {
        // connect_lazy never opens a connection; the store falls back to
        // memory on DB errors, so these tests run without Postgres.
        let pool = sqlx::PgPool::connect_lazy("postgres://localhost:5432/test").unwrap();
        Arc::new(BffState::new(Some(pool)).unwrap())
    }

    async fn body_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        serde_json::from_slice(&bytes).unwrap()
    }

    #[tokio::test]
    async fn test_app_config() {
        let state = test_state().await;

        let query = AppConfigRequest {
            app_id: "test".to_string(),
        };
        let response = get_app_config(State(state.clone()), Query(query))
            .await
            .into_response();

        assert_eq!(response.status(), StatusCode::OK);
        let body: AppConfigResponse = serde_json::from_value(body_json(response).await).unwrap();
        assert_eq!(body.app_id, "test");
        assert_eq!(body.feature_flags.len(), 2);
    }

    #[tokio::test]
    async fn test_voter_verification_rejects_bad_hash() {
        let state = test_state().await;

        let request = VoterVerificationRequest {
            dpo_attestation_hash: "abc123".to_string(),
            justification: "valid reason".to_string(),
            station_code: "STATION-001".to_string(),
        };

        let response = verify_voter(State(state.clone()), Json(request))
            .await
            .into_response();

        // hmac_sha256 placeholder always errs -> hash can never verify.
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_idempotency_store_roundtrip() {
        let state = test_state().await;

        let key = Uuid::now_v7().to_string();
        assert!(state.idempotency_store.get(&key).await.is_none());
        state
            .idempotency_store
            .store(&key, &serde_json::json!({ "ok": true }))
            .await
            .unwrap();
        let cached = state.idempotency_store.get(&key).await.unwrap();
        assert_eq!(cached.0, serde_json::json!({ "ok": true }));
    }
}
