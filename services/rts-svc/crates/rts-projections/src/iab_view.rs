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

use eel_eventstore_rs::StoredEvent;
use sqlx::PgPool;
use tracing::info;
use crate::projector::ProjectionError;

pub struct IabViewProjector { pub pool: PgPool }
impl IabViewProjector {
    pub fn new(pool: PgPool) -> Self { Self { pool } }
    pub async fn project(&self, event: &StoredEvent) -> Result<(), ProjectionError> {
        if event.event_type == "eel.rts.tally_published.v1" {
            info!(stream_id=%event.stream_id, "iab_view: tally_published event received (stub)");
        }
        Ok(())
    }
}
