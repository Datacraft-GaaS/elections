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

//! HTTP handlers for the CRMS API.
//!
//! Every handler reads or drives a crms-domain aggregate and returns real
//! JSON. State is an in-memory `AppState` (see `crate::state`); the handler
//! surface is identical against the future persistence layer.

use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::Json;
use eel_api_server_rs::Problem;
use uuid::Uuid;

use crms_domain::ballot::BallotProof;
use crms_domain::candidate::Candidate;
use crms_domain::candidate::CandidateError;
use crms_domain::clearance::ClearanceDecision;
use crms_domain::clearance::ClearanceSource;
use crms_domain::clearance::ClearanceVerdict;
use crms_domain::coalition::CoalitionRegistration;
use crms_domain::events::DomainEvent;
use crms_domain::events::EventType;
use crms_domain::gazette::Gazette;
use crms_domain::nomination::DocumentRef;
use crms_domain::nomination::Nomination;
use crms_domain::party::PoliticalParty;
use crms_domain::payment::FeePayment;
use crms_domain::payment::FeeSchedule;
use crms_domain::primaries::PrimaryDispute;
use crms_domain::primaries::PrimaryResult;
use crms_domain::quota::evaluate_quota;
use crms_domain::quota::QuotaStatus;
use crms_domain::symbol::PartySymbol;
use crms_domain::symbol::check_symbol_availability;

use crate::models::*;
use crate::state::AppState;

/// API error mapped to an RFC 9457 problem response.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// Requested resource does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// Resource state conflicts with the requested transition.
    #[error("conflict: {0}")]
    Conflict(String),
    /// Request is well-formed but semantically invalid.
    #[error("unprocessable: {0}")]
    Unprocessable(String),
    /// Unexpected internal failure.
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let problem = match &self {
            ApiError::NotFound(d) => Problem::new(
                eel_api_server_rs::ProblemType::not_found(),
                "Not Found",
                404,
            )
            .with_detail(d),
            ApiError::Conflict(d) => Problem::new(
                eel_api_server_rs::ProblemType::conflict(),
                "Conflict",
                409,
            )
            .with_detail(d),
            ApiError::Unprocessable(d) => Problem::new(
                eel_api_server_rs::ProblemType::unprocessable(),
                "Unprocessable",
                422,
            )
            .with_detail(d),
            ApiError::Internal(d) => Problem::new(
                eel_api_server_rs::ProblemType::internal(),
                "Internal Error",
                500,
            )
            .with_detail(d),
        };
        problem.to_axum_response()
    }
}

fn lock_err() -> ApiError {
    ApiError::Internal("state lock poisoned".into())
}

type Result<T> = std::result::Result<T, ApiError>;

/// Mutate a candidate aggregate by id, applying a transition.
fn with_candidate(
    state: &AppState,
    id: Uuid,
    apply: impl FnOnce(&mut Candidate) -> std::result::Result<(), CandidateError>,
) -> Result<Json<Candidate>> {
    let mut guard = state.candidates.lock().map_err(|_| lock_err())?;
    let cand = guard
        .get_mut(&id)
        .ok_or_else(|| ApiError::NotFound(format!("candidate {id}")))?;
    apply(cand).map_err(|e| ApiError::Conflict(e.to_string()))?;
    Ok(Json(cand.clone()))
}

fn not_found(kind: &str, id: Uuid) -> ApiError {
    ApiError::NotFound(format!("{kind} {id}"))
}


pub mod candidate;
pub mod nomination;
pub mod clearance;
pub mod quota;
pub mod public;
pub mod spec;
pub mod gazette;
pub mod ballot;
pub mod party;
pub mod coalition;
pub mod primaries;
pub mod payment;
pub mod symbol;
pub mod events;

pub use candidate::*;
pub use nomination::*;
pub use clearance::*;
pub use quota::*;
pub use gazette::*;
pub use ballot::*;
pub use party::*;
pub use coalition::*;
pub use primaries::*;
pub use payment::*;
pub use symbol::*;
pub use events::*;
