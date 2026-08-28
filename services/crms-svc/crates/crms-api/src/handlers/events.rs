//! Domain event handlers.

use super::*;

/// List domain events.
pub async fn list_events(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DomainEvent>>> {
    let all = state.events.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}
