//! Candidate registration handlers.

use super::*;

/// Create a candidate aggregate.
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

/// List registered candidates.
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

/// Fetch a candidate by id.
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

/// Submit a candidate for returning-officer review.
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

/// Mark a candidate's statutory documents as uploaded.
pub async fn mark_docs_uploaded(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.mark_docs_uploaded())
}

/// Begin returning-officer review.
pub async fn begin_ro_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.begin_ro_review())
}

/// Record returning-officer clearance for a candidate.
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

/// Reject a candidate.
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

/// Move a candidate into the gazette stage.
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

/// Withdraw a candidate.
pub async fn withdraw_candidate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Candidate>> {
    with_candidate(&state, id, |c| c.withdraw())
}
