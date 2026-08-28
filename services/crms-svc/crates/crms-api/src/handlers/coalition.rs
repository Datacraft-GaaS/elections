//! Coalition handlers.

use super::*;

/// Register a coalition.
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

/// Confirm a member party.
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

/// Dissolve a coalition.
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
