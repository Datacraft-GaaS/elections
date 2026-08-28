//! Nomination handlers.

use super::*;

/// Open a nomination for a candidate.
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

/// Fetch a nomination by id.
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

/// Append a statutory document to a nomination.
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
