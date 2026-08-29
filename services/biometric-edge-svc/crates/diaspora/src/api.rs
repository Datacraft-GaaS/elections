//! Axum HTTP router for the diaspora module (WP-5.8, M24).
//!
//! All paths are mounted under /diaspora. Staff endpoints require an
//! Authorization header; public read endpoints are open.

use axum::{
    Router,
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, patch, post},
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::domain::{CustodyEvent, DiasporaRoll, EmbassyStation, Form34ADiaspora, PouchCustody};
use crate::import::{ImportProfile, import_roll_csv};
use crate::roll::roll_from_voter_ids;
use crate::transmission::{BatchStatus, TransmissionBatch, batch_transmit};

/// In-memory state for development / testing. Replace with sqlx pool in prod.
/// Postgres-backed for embassies/ballot_dispatches/return_dispatches via `Store` (no in-memory).
pub struct DiasporaState {
    pub stations: Mutex<Vec<EmbassyStation>>,
    pub rolls: Mutex<Vec<DiasporaRoll>>,
    pub forms: Mutex<Vec<Form34ADiaspora>>,
    pub pouches: Mutex<Vec<PouchCustody>>,
    pub batches: Mutex<Vec<TransmissionBatch>>,
    pub diaspora_store: crate::store::Store,
}

impl Default for DiasporaState {
    fn default() -> Self {
        Self {
            stations: Mutex::new(Vec::new()),
            rolls: Mutex::new(Vec::new()),
            forms: Mutex::new(Vec::new()),
            pouches: Mutex::new(Vec::new()),
            batches: Mutex::new(Vec::new()),
            diaspora_store: crate::store::Store::mem(),
        }
    }
}

impl DiasporaState {
    /// Create with postgres pool (production — no in-memory).
    pub fn from_pool(pool: sqlx::PgPool) -> Self {
        Self {
            stations: Mutex::new(Vec::new()),
            rolls: Mutex::new(Vec::new()),
            forms: Mutex::new(Vec::new()),
            pouches: Mutex::new(Vec::new()),
            batches: Mutex::new(Vec::new()),
            diaspora_store: crate::store::Store::pg(pool),
        }
    }
}

pub type SharedState = Arc<DiasporaState>;

pub fn router() -> Router<SharedState> {
    Router::new()
        // Stations
        .route("/stations", post(create_station).get(list_stations))
        .route("/stations/{id}/open", patch(open_station))
        .route("/stations/{id}/close", patch(close_station))
        // Rolls
        .route("/rolls/import", post(import_roll))
        .route("/rolls/{id}/freeze", post(freeze_roll))
        .route("/rolls/{id}", get(get_roll))
        // Forms
        .route("/forms/34a", post(create_form34a))
        .route("/forms/34a/{id}", get(get_form34a))
        // Batches
        .route("/transmission/batches", post(create_batch))
        .route("/transmission/batches/{id}", get(get_batch))
        // Pouches
        .route("/pouches", post(create_pouch))
        .route("/pouches/{id}/transfer", post(transfer_pouch))
        .route("/pouches/{id}/receive", post(receive_pouch))
        // Public contract endpoints (diaspora-logistics/logistics.yaml)
        .route("/api/v1/diaspora/embassies", get(crate::spec::list_embassies))
        .route("/api/v1/diaspora/embassies/{embassy_code}", get(crate::spec::get_embassy))
        .route("/api/v1/diaspora/ballot-dispatches", get(crate::spec::list_ballot_dispatches).post(crate::spec::create_ballot_dispatch))
        .route("/api/v1/diaspora/ballot-dispatches/{dispatch_id}", get(crate::spec::get_ballot_dispatch))
        .route("/api/v1/diaspora/ballot-dispatches/{dispatch_id}/delivery-confirmation", post(crate::spec::confirm_ballot_delivery))
        .route("/api/v1/diaspora/return-dispatches", get(crate::spec::list_return_dispatches))
}

// ---- Station handlers -------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateStationBody {
    pub country_iso: String,
    pub city: String,
    pub timezone: String,
    pub address: String,
    pub station_code: String,
    pub kits_deployed: u32,
}

async fn create_station(
    State(state): State<SharedState>,
    Json(body): Json<CreateStationBody>,
) -> impl IntoResponse {
    let station = EmbassyStation::new(
        body.country_iso,
        body.city,
        body.timezone,
        body.address,
        body.station_code,
        body.kits_deployed,
    );
    let mut lock = state.stations.lock().expect("lock poisoned");
    lock.push(station.clone());
    (StatusCode::CREATED, Json(station))
}

async fn list_stations(State(state): State<SharedState>) -> impl IntoResponse {
    let lock = state.stations.lock().expect("lock poisoned");
    Json(lock.clone())
}

async fn open_station(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut lock = state.stations.lock().expect("lock poisoned");
    if let Some(s) = lock.iter_mut().find(|s| s.id == id) {
        s.opened_at = Some(Utc::now());
        s.closed_at = None;
        return (StatusCode::OK, Json(serde_json::json!({"status": "opened"}))).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "station not found"}))).into_response()
}

async fn close_station(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut lock = state.stations.lock().expect("lock poisoned");
    if let Some(s) = lock.iter_mut().find(|s| s.id == id) {
        s.closed_at = Some(Utc::now());
        return (StatusCode::OK, Json(serde_json::json!({"status": "closed"}))).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "station not found"}))).into_response()
}

// ---- Roll handlers ----------------------------------------------------------

async fn import_roll(
    State(state): State<SharedState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut profile = ImportProfile::RestOfWorld;
    let mut csv_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        match field.name() {
            Some("profile") => {
                if let Ok(text) = field.text().await {
                    profile = serde_json::from_str(&format!("\"{text}\""))
                        .unwrap_or(ImportProfile::RestOfWorld);
                }
            }
            Some("file") => {
                csv_bytes = field.bytes().await.ok().map(|b| b.to_vec());
            }
            _ => {}
        }
    }

    let csv = csv_bytes
        .as_deref()
        .and_then(|b| std::str::from_utf8(b).ok())
        .unwrap_or("");

    let report = import_roll_csv(profile, csv);

    // Build a roll from accepted voter IDs.
    let voter_ids: Vec<String> = report
        .rows
        .iter()
        .filter(|r| r.ok)
        .filter_map(|r| r.row.as_ref())
        .map(|r| r.id.clone())
        .collect();

    if !voter_ids.is_empty() {
        // Associate with first open station for the profile; default to nil.
        let station_id = state
            .stations
            .lock()
            .unwrap()
            .iter()
            .find(|s| s.is_open())
            .map(|s| s.id)
            .unwrap_or_else(Uuid::now_v7);

        let roll = roll_from_voter_ids(station_id, voter_ids);
        state.rolls.lock().expect("lock poisoned").push(roll);
    }

    (StatusCode::OK, Json(report))
}

async fn freeze_roll(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut lock = state.rolls.lock().expect("lock poisoned");
    if let Some(pos) = lock.iter().position(|r| r.id == id) {
        let roll = lock.remove(pos).freeze(Utc::now());
        lock.push(roll.clone());
        return (StatusCode::OK, Json(roll)).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "roll not found"}))).into_response()
}

async fn get_roll(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let lock = state.rolls.lock().expect("lock poisoned");
    if let Some(roll) = lock.iter().find(|r| r.id == id) {
        return (StatusCode::OK, Json(roll.clone())).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "roll not found"}))).into_response()
}

// ---- Form 34A handlers ------------------------------------------------------

#[derive(Deserialize)]
pub struct CreateForm34ABody {
    pub embassy_station_id: Uuid,
    pub form_id: String,
    pub results: HashMap<String, u64>,
    pub rejected: u64,
    pub disputed: u64,
    /// Optional base64-encoded HSM signature supplied by the kit.
    pub hsm_signature: Option<String>,
}

#[derive(Serialize)]
pub struct Form34AResponse {
    pub form: Form34ADiaspora,
    pub event_type: &'static str,
}

async fn create_form34a(
    State(state): State<SharedState>,
    Json(body): Json<CreateForm34ABody>,
) -> impl IntoResponse {
    let mut form = Form34ADiaspora::new(
        body.embassy_station_id,
        body.form_id,
        body.results,
        body.rejected,
        body.disputed,
    );
    form.hsm_signature = body.hsm_signature;

    state.forms.lock().expect("lock poisoned").push(form.clone());

    let response = Form34AResponse {
        form,
        event_type: crate::events::FORM_CAPTURED,
    };
    (StatusCode::CREATED, Json(response))
}

async fn get_form34a(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let lock = state.forms.lock().expect("lock poisoned");
    if let Some(form) = lock.iter().find(|f| f.id == id) {
        return (StatusCode::OK, Json(form.clone())).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "form not found"}))).into_response()
}

// ---- Transmission batch handlers --------------------------------------------

#[derive(Deserialize)]
pub struct CreateBatchBody {
    pub form_ids: Vec<Uuid>,
}

async fn create_batch(
    State(state): State<SharedState>,
    Json(body): Json<CreateBatchBody>,
) -> impl IntoResponse {
    let forms_lock = state.forms.lock().expect("lock poisoned");
    let forms: Vec<Form34ADiaspora> = forms_lock
        .iter()
        .filter(|f| body.form_ids.contains(&f.id))
        .cloned()
        .collect();
    drop(forms_lock);

    let (batch, _deduped) = batch_transmit(forms, Utc::now());
    state.batches.lock().expect("lock poisoned").push(batch.clone());

    (StatusCode::CREATED, Json(serde_json::json!({
        "id": batch.id,
        "form_ids": batch.form_ids,
        "status": format!("{:?}", batch.status),
    })))
}

async fn get_batch(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let lock = state.batches.lock().expect("lock poisoned");
    if let Some(b) = lock.iter().find(|b| b.id == id) {
        return (StatusCode::OK, Json(serde_json::json!({
            "id": b.id,
            "form_ids": b.form_ids,
            "status": format!("{:?}", b.status),
            "attempts": b.attempts,
        }))).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "batch not found"}))).into_response()
}

// ---- Pouch handlers ---------------------------------------------------------

#[derive(Deserialize)]
pub struct CreatePouchBody {
    pub form_ids: Vec<Uuid>,
    pub seal_id: Option<String>,
}

async fn create_pouch(
    State(state): State<SharedState>,
    Json(body): Json<CreatePouchBody>,
) -> impl IntoResponse {
    let mut pouch = PouchCustody::new(body.form_ids);
    if let Some(seal_id) = body.seal_id {
        pouch.seal(seal_id, Utc::now());
    }
    state.pouches.lock().expect("lock poisoned").push(pouch.clone());
    (StatusCode::CREATED, Json(pouch))
}

#[derive(Deserialize)]
pub struct TransferPouchBody {
    pub from_person: String,
    pub to_person: String,
    pub location: String,
    pub gps_lat: f64,
    pub gps_lon: f64,
    pub signature: String,
}

async fn transfer_pouch(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
    Json(body): Json<TransferPouchBody>,
) -> impl IntoResponse {
    let mut lock = state.pouches.lock().expect("lock poisoned");
    if let Some(pouch) = lock.iter_mut().find(|p| p.id == id) {
        let event = CustodyEvent {
            transferred_at: Utc::now(),
            from_person: body.from_person,
            to_person: body.to_person,
            location: body.location,
            gps_lat: body.gps_lat,
            gps_lon: body.gps_lon,
            signature: body.signature,
        };
        pouch.transfer(event);
        return (StatusCode::OK, Json(pouch.clone())).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "pouch not found"}))).into_response()
}

async fn receive_pouch(
    State(state): State<SharedState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    let mut lock = state.pouches.lock().expect("lock poisoned");
    if let Some(pouch) = lock.iter_mut().find(|p| p.id == id) {
        pouch.receive_at_nairobi(Utc::now());
        return (StatusCode::OK, Json(pouch.clone())).into_response();
    }
    (StatusCode::NOT_FOUND, Json(serde_json::json!({"error": "pouch not found"}))).into_response()
}
