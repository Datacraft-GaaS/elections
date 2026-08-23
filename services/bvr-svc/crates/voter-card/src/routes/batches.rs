use crate::{
    error::ApiError,
    models::{BatchStatus, CardBatch},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/v1/voter-cards/batches", post(create_batch))
        .route("/v1/voter-cards/batches/{id}", get(get_batch))
}

/// Wire representation of a BatchStatus (matches the SCREAMING_SNAKE_CASE
/// serde renaming and the status CHECK constraint in V053).
fn batch_status_str(status: &BatchStatus) -> &'static str {
    use BatchStatus::*;
    match status {
        Draft => "DRAFT",
        Submitted => "SUBMITTED",
        InProduction => "IN_PRODUCTION",
        Received => "RECEIVED",
        Distributed => "DISTRIBUTED",
    }
}

/// Inverse of `batch_status_str`.
fn batch_status_from_str(s: &str) -> Option<BatchStatus> {
    use BatchStatus::*;
    match s {
        "DRAFT" => Some(Draft),
        "SUBMITTED" => Some(Submitted),
        "IN_PRODUCTION" => Some(InProduction),
        "RECEIVED" => Some(Received),
        "DISTRIBUTED" => Some(Distributed),
        _ => None,
    }
}

/// Column order of every SELECT over card_batches.
type BatchRow = (
    Uuid,
    Uuid,
    String,
    i64,
    Option<String>,
    String,
    DateTime<Utc>,
    Option<DateTime<Utc>>,
);

const BATCH_COLUMNS: &str =
    "id, election_id, constituency_id, order_count, manufacturer_ref, status, ordered_at, received_at";

fn batch_from_row(row: BatchRow) -> Result<CardBatch, ApiError> {
    let (
        id,
        election_id,
        constituency_id,
        order_count,
        manufacturer_ref,
        status_s,
        ordered_at,
        received_at,
    ) = row;

    let status = batch_status_from_str(&status_s)
        .ok_or_else(|| ApiError::Internal(format!("unknown batch status {status_s}")))?;

    Ok(CardBatch {
        id,
        election_id,
        constituency_id,
        order_count: order_count.max(0) as u64,
        manufacturer_ref,
        status,
        ordered_at,
        received_at,
    })
}

/// POST /v1/voter-cards/batches
///
/// Creates a card production batch. The body is a CardBatch JSON document;
/// id defaults to a fresh UUIDv7 and status to DRAFT when absent.
async fn create_batch(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<(StatusCode, Json<CardBatch>), ApiError> {
    if !body.is_object() {
        return Err(ApiError::BadRequest("batch body must be a JSON object".into()));
    }

    let mut val = body;
    if val.get("id").is_none() {
        val["id"] = json!(Uuid::now_v7());
    }
    if val.get("status").is_none() {
        val["status"] = json!("DRAFT");
    }
    let batch: CardBatch = serde_json::from_value(val)
        .map_err(|e| ApiError::BadRequest(format!("invalid batch payload: {e}")))?;

    sqlx::query(
        "INSERT INTO card_batches (id, election_id, constituency_id, order_count, \
         manufacturer_ref, status, ordered_at, received_at) \
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(batch.id)
    .bind(batch.election_id)
    .bind(&batch.constituency_id)
    .bind(batch.order_count as i64)
    .bind(&batch.manufacturer_ref)
    .bind(batch_status_str(&batch.status))
    .bind(batch.ordered_at)
    .bind(batch.received_at)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::Internal(e.to_string()))?;

    tracing::info!(
        batch_id = %batch.id,
        election_id = %batch.election_id,
        "card batch created"
    );
    Ok((StatusCode::CREATED, Json(batch)))
}

/// GET /v1/voter-cards/batches/{id}
async fn get_batch(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CardBatch>, ApiError> {
    let row: Option<BatchRow> =
        sqlx::query_as(&format!("SELECT {BATCH_COLUMNS} FROM card_batches WHERE id = $1"))
            .bind(id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| ApiError::Internal(e.to_string()))?;

    match row {
        Some(row) => Ok(Json(batch_from_row(row)?)),
        None => Err(ApiError::NotFound(format!("batch {id} not found"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_status_round_trips_through_storage_form() {
        for status in [
            BatchStatus::Draft,
            BatchStatus::Submitted,
            BatchStatus::InProduction,
            BatchStatus::Received,
            BatchStatus::Distributed,
        ] {
            assert_eq!(batch_status_from_str(batch_status_str(&status)), Some(status));
        }
        assert_eq!(batch_status_from_str("REPRINTED"), None);
    }

    #[test]
    fn batch_status_storage_form_matches_serde_wire_form() {
        // The TEXT values stored in V053 must agree with the SCREAMING_SNAKE_CASE
        // serde renaming on CardBatch.
        for status in [
            BatchStatus::Draft,
            BatchStatus::Submitted,
            BatchStatus::InProduction,
            BatchStatus::Received,
            BatchStatus::Distributed,
        ] {
            let wire = serde_json::to_string(&status).unwrap();
            assert_eq!(wire, format!("\"{}\"", batch_status_str(&status)));
        }
    }
}
