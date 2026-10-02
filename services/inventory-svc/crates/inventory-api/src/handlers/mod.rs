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
