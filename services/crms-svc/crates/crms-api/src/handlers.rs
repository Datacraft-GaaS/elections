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
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unprocessable: {0}")]
    Unprocessable(String),
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

// ---------------------------------------------------------------------------
// Candidates
// ---------------------------------------------------------------------------

pub async fn create_candidate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCandidate>,
) -> Result<Json<Candidate>> {
    let cand = Candidate::new(
        Uuid::new_v4(),
        req.id_number_hash,
        req.name,
        req.party_id,
        req.position,
        req.constituency_code,
        req.gender,
        req.date_of_birth,
        req.is_independent,
    );
    state
        .candidates
        .lock()
        .map_err(|_| lock_err())?
        .insert(cand.id, cand.clone());
    Ok(Json(cand))
}

pub async fn list_candidates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Candidate>>> {
    let all = state
        .candidates
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    Ok(Json(all))
}

pub async fn get_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    let cand = state
        .candidates
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("candidate", id))?;
    Ok(Json(cand))
}

pub async fn submit_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    let cand = with_candidate(&state, id, |c| c.submit())?;
    state.record_event(DomainEvent::new(
        EventType::CandidateSubmitted,
        "candidate",
        id,
        serde_json::json!({ "candidate_id": id }),
    ));
    Ok(cand)
}

pub async fn mark_docs_uploaded(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.mark_docs_uploaded())
}

pub async fn begin_ro_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.begin_ro_review())
}

pub async fn clear_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    let cand = with_candidate(&state, id, |c| c.clear())?;
    state.record_event(DomainEvent::new(
        EventType::CandidateCleared,
        "candidate",
        id,
        serde_json::json!({ "candidate_id": id }),
    ));
    Ok(cand)
}

pub async fn reject_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    let cand = with_candidate(&state, id, |c| c.reject())?;
    state.record_event(DomainEvent::new(
        EventType::CandidateRejected,
        "candidate",
        id,
        serde_json::json!({ "candidate_id": id }),
    ));
    Ok(cand)
}

pub async fn gazette_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    let cand = with_candidate(&state, id, |c| c.gazette())?;
    state.record_event(DomainEvent::new(
        EventType::CandidateGazetted,
        "candidate",
        id,
        serde_json::json!({ "candidate_id": id }),
    ));
    Ok(cand)
}

pub async fn withdraw_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.withdraw())
}

// ---------------------------------------------------------------------------
// Nominations
// ---------------------------------------------------------------------------

pub async fn create_nomination(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateNomination>,
) -> Result<Json<Nomination>> {
    let exists = state
        .candidates
        .lock()
        .map_err(|_| lock_err())?
        .contains_key(&req.candidate_id);
    if !exists {
        return Err(not_found("candidate", req.candidate_id));
    }
    let nom = Nomination::new(Uuid::new_v4(), req.candidate_id);
    state
        .nominations
        .lock()
        .map_err(|_| lock_err())?
        .insert(nom.id, nom.clone());
    Ok(Json(nom))
}

pub async fn get_nomination(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Nomination>> {
    let nom = state
        .nominations
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("nomination", id))?;
    Ok(Json(nom))
}

pub async fn add_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<AddDocument>,
) -> Result<Json<Nomination>> {
    let mut guard = state.nominations.lock().map_err(|_| lock_err())?;
    let nom = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("nomination", id))?;
    nom.add_document(DocumentRef {
        doc_id: req.doc_id,
        doc_type: req.doc_type,
        storage_ref: req.storage_ref,
        uploaded_at: chrono::Utc::now(),
    });
    Ok(Json(nom.clone()))
}

// ---------------------------------------------------------------------------
// Clearance
// ---------------------------------------------------------------------------

pub async fn create_clearance(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateClearance>,
) -> Result<Json<ClearanceDecision>> {
    let decision = ClearanceDecision::new(
        req.nomination_id,
        req.source,
        req.verdict,
        req.evidence_ref,
        req.officer_id,
        req.override_reason,
    );
    decision.validate().map_err(ApiError::Unprocessable)?;
    state
        .clearances
        .lock()
        .map_err(|_| lock_err())?
        .push(decision.clone());
    Ok(Json(decision))
}

pub async fn list_clearances(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<ClearanceDecision>>> {
    let all = state
        .clearances
        .lock()
        .map_err(|_| lock_err())?
        .clone();
    Ok(Json(all))
}

/// Advisory clearance from a statutory source (KRA / EACC doubles).
pub async fn advisory_clearance(
    Json(req): Json<AdvisoryClearance>,
) -> Result<Json<ClearanceVerdict>> {
    let verdict = match req.source {
        ClearanceSource::Kra => crms_domain::clearance::kra_advisory_check(&req.id_number_hash).await,
        ClearanceSource::Eacc => {
            crms_domain::clearance::eacc_advisory_check(&req.id_number_hash).await
        }
        _ => return Err(ApiError::Unprocessable("advisory supported for KRA/EACC only".into())),
    };
    verdict.map(Json).map_err(ApiError::Unprocessable)
}

// ---------------------------------------------------------------------------
// Quota
// ---------------------------------------------------------------------------

pub async fn evaluate_quota_rule(
    Json(req): Json<EvaluateQuota>,
) -> Result<Json<QuotaStatus>> {
    let roster: Vec<(crms_domain::candidate::Gender, bool, &str)> = req
        .candidates
        .iter()
        .map(|c| (c.gender.clone(), c.is_pwd, c.date_of_birth.as_str()))
        .collect();
    let status = evaluate_quota(req.scope, req.position, &roster, &req.nomination_date);
    if !status.two_thirds_compliant {
        stateless_quota_event(&status);
    }
    Ok(Json(status))
}

fn stateless_quota_event(_status: &QuotaStatus) {
    // Quota evaluation is stateless; consumers read the flag from the response.
    // A wired deployment forwards this to the telemetry/ECC sink.
    let _ = _status;
}

// ---------------------------------------------------------------------------
// Gazette
// ---------------------------------------------------------------------------

pub async fn build_gazette(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildGazette>,
) -> Result<Json<Gazette>> {
    let gazette = Gazette::build(
        req.election_id,
        req.position_code,
        req.entries,
        req.officer_a_id,
        req.officer_b_id,
    );
    state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .insert(gazette.id, gazette.clone());
    Ok(Json(gazette))
}

pub async fn get_gazette(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Gazette>> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    Ok(Json(gazette))
}

pub async fn publish_gazette(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Gazette>> {
    let mut guard = state.gazettes.lock().map_err(|_| lock_err())?;
    let gazette = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("gazette", id))?;
    gazette.publish();
    let g = gazette.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::GazettePublished,
        "gazette",
        id,
        serde_json::json!({ "gazette_id": id, "content_hash": g.content_hash }),
    ));
    Ok(Json(g))
}

pub async fn gazette_csv(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Response> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    Ok(([(header::CONTENT_TYPE, "text/csv; charset=utf-8")], gazette.to_csv()).into_response())
}

pub async fn gazette_signed_json(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let gazette = state
        .gazettes
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("gazette", id))?;
    let parsed = serde_json::from_str(&gazette.to_signed_json())
        .map_err(|e| ApiError::Internal(e.to_string()))?;
    Ok(Json(parsed))
}

// ---------------------------------------------------------------------------
// Ballot proofs
// ---------------------------------------------------------------------------

pub async fn build_ballot(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildBallot>,
) -> Result<Json<BallotProof>> {
    let proof = BallotProof::build(
        req.election_id,
        req.position_code,
        req.constituency_code,
        req.positions,
    );
    state
        .ballots
        .lock()
        .map_err(|_| lock_err())?
        .insert(proof.id, proof.clone());
    Ok(Json(proof))
}

pub async fn get_ballot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BallotProof>> {
    let proof = state
        .ballots
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("ballot", id))?;
    Ok(Json(proof))
}

pub async fn ballot_party_sign_off(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<BallotProof>> {
    let mut guard = state.ballots.lock().map_err(|_| lock_err())?;
    let proof = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("ballot", id))?;
    proof.record_party_sign_off();
    Ok(Json(proof.clone()))
}

pub async fn approve_ballot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveBallot>,
) -> Result<Json<BallotProof>> {
    let mut guard = state.ballots.lock().map_err(|_| lock_err())?;
    let proof = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("ballot", id))?;
    proof.approve(req.printer_sig);
    let p = proof.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::BallotApproved,
        "ballot",
        id,
        serde_json::json!({ "ballot_id": id, "ballot_hash": p.ballot_hash }),
    ));
    Ok(Json(p))
}

// ---------------------------------------------------------------------------
// Parties
// ---------------------------------------------------------------------------

pub async fn create_party(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateParty>,
) -> Result<Json<PoliticalParty>> {
    let party = PoliticalParty::new(
        Uuid::new_v4(),
        req.name_en,
        req.name_sw,
        req.abbreviation,
        req.hq_address,
    );
    state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .insert(party.id, party.clone());
    Ok(Json(party))
}

pub async fn list_parties(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PoliticalParty>>> {
    let all = state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    Ok(Json(all))
}

pub async fn get_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let party = state
        .parties
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("party", id))?;
    Ok(Json(party))
}

pub async fn submit_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("party", id))?;
    party.submit();
    Ok(Json(party.clone()))
}

pub async fn approve_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ApproveParty>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("party", id))?;
    party.approve(req.registration_number, req.gazette_url);
    let p = party.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::PartyApproved,
        "party",
        id,
        serde_json::json!({ "party_id": id, "registration_number": p.registration_number }),
    ));
    Ok(Json(p))
}

pub async fn suspend_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("party", id))?;
    party.suspend();
    let p = party.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::PartySuspended,
        "party",
        id,
        serde_json::json!({ "party_id": id }),
    ));
    Ok(Json(p))
}

pub async fn deregister_party(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PoliticalParty>> {
    let mut guard = state.parties.lock().map_err(|_| lock_err())?;
    let party = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("party", id))?;
    party.deregister();
    Ok(Json(party.clone()))
}

// ---------------------------------------------------------------------------
// Coalitions
// ---------------------------------------------------------------------------

pub async fn create_coalition(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateCoalition>,
) -> Result<Json<CoalitionRegistration>> {
    let coalition = CoalitionRegistration::new(Uuid::new_v4(), req.name, req.member_party_ids);
    state
        .coalitions
        .lock()
        .map_err(|_| lock_err())?
        .insert(coalition.id, coalition.clone());
    Ok(Json(coalition))
}

pub async fn confirm_member(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ConfirmMember>,
) -> Result<Json<CoalitionRegistration>> {
    let mut guard = state.coalitions.lock().map_err(|_| lock_err())?;
    let coalition = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("coalition", id))?;
    let was_active = coalition.status == crms_domain::coalition::CoalitionStatus::Active;
    coalition.confirm_member(req.party_id);
    let active_now = coalition.status == crms_domain::coalition::CoalitionStatus::Active;
    let c = coalition.clone();
    drop(guard);
    if active_now && !was_active {
        state.record_event(DomainEvent::new(
            EventType::CoalitionActive,
            "coalition",
            id,
            serde_json::json!({ "coalition_id": id }),
        ));
    }
    Ok(Json(c))
}

pub async fn dissolve_coalition(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CoalitionRegistration>> {
    let mut guard = state.coalitions.lock().map_err(|_| lock_err())?;
    let coalition = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("coalition", id))?;
    coalition.dissolve();
    let c = coalition.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::CoalitionDissolved,
        "coalition",
        id,
        serde_json::json!({ "coalition_id": id }),
    ));
    Ok(Json(c))
}

// ---------------------------------------------------------------------------
// Primaries
// ---------------------------------------------------------------------------

pub async fn submit_primary_result(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SubmitPrimaryResult>,
) -> Result<Json<PrimaryResult>> {
    let result = PrimaryResult::build(
        req.party_id,
        req.constituency_code,
        req.position,
        req.submitted_by,
        req.candidate_tallies,
        req.spoilt_ballots,
        req.total_turnout,
    );
    state
        .primary_results
        .lock()
        .map_err(|_| lock_err())?
        .insert(result.id, result.clone());
    state.record_event(DomainEvent::new(
        EventType::PrimaryResultSubmitted,
        "primary_result",
        result.id,
        serde_json::json!({ "result_id": result.id, "result_hash": result.result_hash }),
    ));
    Ok(Json(result))
}

pub async fn get_primary_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PrimaryResult>> {
    let result = state
        .primary_results
        .lock()
        .map_err(|_| lock_err())?
        .get(&id)
        .cloned()
        .ok_or_else(|| not_found("primary result", id))?;
    Ok(Json(result))
}

pub async fn publish_primary_result(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PrimaryResult>> {
    let mut guard = state.primary_results.lock().map_err(|_| lock_err())?;
    let result = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("primary result", id))?;
    result.publish();
    Ok(Json(result.clone()))
}

pub async fn create_primary_dispute(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreatePrimaryDispute>,
) -> Result<Json<PrimaryDispute>> {
    let dispute = PrimaryDispute::new(
        req.result_id,
        req.complainant_name,
        req.complainant_id,
        req.grounds,
        req.evidence_urls,
    );
    state
        .disputes
        .lock()
        .map_err(|_| lock_err())?
        .push(dispute.clone());
    Ok(Json(dispute))
}

pub async fn list_primary_disputes(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PrimaryDispute>>> {
    let all = state.disputes.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}

// ---------------------------------------------------------------------------
// Payments / fee schedules
// ---------------------------------------------------------------------------

pub async fn create_fee_schedule(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateFeeSchedule>,
) -> Result<Json<FeeSchedule>> {
    let schedule = FeeSchedule::new(req.election_cycle, req.position, req.amount_cents, req.due_at);
    state
        .fee_schedules
        .lock()
        .map_err(|_| lock_err())?
        .push(schedule.clone());
    Ok(Json(schedule))
}

pub async fn list_fee_schedules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeeSchedule>>> {
    let all = state.fee_schedules.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}

pub async fn record_payment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RecordPayment>,
) -> Result<Json<FeePayment>> {
    let payment = FeePayment::new(req.schedule_id, req.candidate_id, req.amount_cents);
    state
        .payments
        .lock()
        .map_err(|_| lock_err())?
        .insert(payment.id, payment.clone());
    Ok(Json(payment))
}

pub async fn confirm_payment(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(req): Json<ConfirmPayment>,
) -> Result<Json<FeePayment>> {
    let mut guard = state.payments.lock().map_err(|_| lock_err())?;
    let payment = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("payment", id))?;
    let candidate_id = payment.candidate_id;
    payment.confirm(req.mpesa_ref);
    let p = payment.clone();
    drop(guard);

    // Confirm payment gates candidate submission (FR-M47-006).
    if let Ok(mut cands) = state.candidates.lock() {
        if let Some(cand) = cands.get_mut(&candidate_id) {
            cand.payment_confirmed = true;
        }
    }

    state.record_event(DomainEvent::new(
        EventType::PaymentConfirmed,
        "payment",
        id,
        serde_json::json!({ "payment_id": id, "candidate_id": candidate_id }),
    ));
    Ok(Json(p))
}

// ---------------------------------------------------------------------------
// Party symbols
// ---------------------------------------------------------------------------

pub async fn register_symbol(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterSymbol>,
) -> Result<Json<PartySymbol>> {
    let existing: Vec<PartySymbol> = state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    check_symbol_availability(&existing, &req.symbol_name, &req.colour_hex)
        .map_err(ApiError::Conflict)?;
    let symbol = PartySymbol::new(req.party_id, req.symbol_name, req.symbol_url, req.colour_hex);
    state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .insert(symbol.id, symbol.clone());
    Ok(Json(symbol))
}

pub async fn list_symbols(State(state): State<Arc<AppState>>) -> Result<Json<Vec<PartySymbol>>> {
    let all = state
        .symbols
        .lock()
        .map_err(|_| lock_err())?
        .values()
        .cloned()
        .collect();
    Ok(Json(all))
}

pub async fn approve_symbol(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PartySymbol>> {
    let mut guard = state.symbols.lock().map_err(|_| lock_err())?;
    let symbol = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("symbol", id))?;
    symbol.approve();
    let s = symbol.clone();
    drop(guard);
    state.record_event(DomainEvent::new(
        EventType::SymbolAssigned,
        "symbol",
        id,
        serde_json::json!({ "symbol_id": id, "party_id": s.party_id }),
    ));
    Ok(Json(s))
}

pub async fn reject_symbol(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<PartySymbol>> {
    let mut guard = state.symbols.lock().map_err(|_| lock_err())?;
    let symbol = guard
        .get_mut(&id)
        .ok_or_else(|| not_found("symbol", id))?;
    symbol.reject();
    Ok(Json(symbol.clone()))
}

// ---------------------------------------------------------------------------
// Domain events
// ---------------------------------------------------------------------------

pub async fn list_events(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DomainEvent>>> {
    let all = state.events.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}
