//! Public contract handlers (crms/nomination.yaml quota + ballot-proofs).

use super::*;
use axum::extract::{Path, State};

/// POST /api/v1/crms/ballot-proofs — publish a signed ballot proof.
pub async fn publish_ballot_proof(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::spec::BallotProofRequest>,
) -> Result<Json<crate::spec::BallotProof>> {
    let proof = crate::spec::BallotProof {
        proof_id: Uuid::now_v7().to_string(),
        election_id: req.election_id,
        position_code: req.position_code,
        ballot_hash: req.ballot_hash,
        printer_sig: req.printer_sig,
        published_at: chrono::Utc::now().to_rfc3339(),
    };
    state
        .spec_ballot_proofs
        .lock()
        .map_err(|_| lock_err())?
        .insert(proof.proof_id.clone(), proof.clone());
    Ok(Json(proof))
}

/// GET /api/v1/crms/ballot-proofs/{proof_id} — fetch a published ballot proof.
pub async fn get_ballot_proof(
    State(state): State<Arc<AppState>>,
    Path(proof_id): Path<String>,
) -> Result<Json<crate::spec::BallotProof>> {
    let proof = state
        .spec_ballot_proofs
        .lock()
        .map_err(|_| lock_err())?
        .get(&proof_id)
        .cloned()
        .ok_or_else(|| ApiError::NotFound(format!("ballot proof {proof_id}")))?;
    Ok(Json(proof))
}

/// GET /api/v1/crms/quotas/{scope}/{code} — report quota compliance.
pub async fn get_quota_compliance(
    State(state): State<Arc<AppState>>,
    Path((scope, code)): Path<(String, String)>,
) -> Result<Json<crate::spec::QuotaStatus>> {
    let guard = state.candidates.lock().map_err(|_| lock_err())?;
    let candidates: Vec<&crms_domain::candidate::Candidate> = match scope.as_str() {
        "national" => guard.values().collect(),
        _ => guard
            .values()
            .filter(|c| c.constituency_code == code || c.constituency_code == scope)
            .collect(),
    };

    let total = candidates.len() as u32;
    let female = candidates
        .iter()
        .filter(|c| c.gender == crms_domain::candidate::Gender::Female)
        .count() as u32;
    let male = total - female;
    let gender_compliance = crms_domain::quota::check_two_thirds_rule(total, female, male).is_ok();
    let youth_compliance = candidates
        .iter()
        .any(|c| crms_domain::quota::is_youth(&c.date_of_birth, &chrono::Utc::now().to_rfc3339()));

    Ok(Json(crate::spec::QuotaStatus {
        scope,
        code,
        gender_compliance,
        youth_compliance,
        pwd_compliance: false,
    }))
}
