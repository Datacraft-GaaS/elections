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

//! Domain event handlers.

use super::*;

/// List domain events.
pub async fn list_events(State(state): State<Arc<AppState>>) -> Result<Json<Vec<DomainEvent>>> {
    let all = state.events.lock().map_err(|_| lock_err())?.clone();
    Ok(Json(all))
}
