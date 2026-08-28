//! Fee schedule and payment handlers.

use super::*;

/// Declare a fee schedule.
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

/// List fee schedules.
pub async fn list_fee_schedules(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<FeeSchedule>>> {
    let all = state.fee_schedules.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}

/// Record a fee payment.
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

/// Confirm a payment via M-Pesa reference.
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
