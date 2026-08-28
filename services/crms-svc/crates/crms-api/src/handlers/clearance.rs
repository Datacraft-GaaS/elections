//! Clearance handlers.

use super::*;

/// Record a clearance decision.
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

/// List clearance decisions.
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
/// Record an advisory clearance.
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
