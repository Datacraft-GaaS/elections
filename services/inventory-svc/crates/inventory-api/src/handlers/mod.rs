//! HTTP handlers for inventory-svc, split by domain.

pub mod custody;
pub mod transport;
pub mod warehouse;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;

/// Standard problem response body.
pub fn problem(status: StatusCode, msg: impl Into<String>) -> Response {
    (status, Json(serde_json::json!({ "error": msg.into() }))).into_response()
}

/// Map a storage error to a 500 problem response.
pub fn db_error(e: sqlx::Error) -> Response {
    problem(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("storage error: {e}"),
    )
}

/// Map a transport-domain error to an HTTP response.
pub fn transport_error(err: &::transport::domain::TransportError) -> Response {
    use ::transport::domain::TransportError::*;
    let status = match err {
        NotLoading(_) | EmptyLoad => StatusCode::CONFLICT,
        CapacityExceeded { .. }
        | LotAlreadyLoaded(_)
        | InvalidScan
        | InvalidWindow
        | AlreadyAssigned { .. } => StatusCode::UNPROCESSABLE_ENTITY,
    };
    problem(status, err.to_string())
}

/// Map a warehouse-domain error to an HTTP response.
pub fn warehouse_error(err: &::warehouse::domain::WarehouseError) -> Response {
    use ::warehouse::domain::WarehouseError::*;
    let status = match err {
        InvalidTransition { .. } | AlreadyResolved => StatusCode::CONFLICT,
        NoSelfTransfer | EmptySummary | EmptyBreakBulk | InvalidGeofence { .. } => {
            StatusCode::UNPROCESSABLE_ENTITY
        }
    };
    problem(status, err.to_string())
}
