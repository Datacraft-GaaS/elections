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

use std::sync::Arc;

use tokio::time::{interval, Duration};

/// Runs as a background tokio task. Purges expired previews every 5 minutes.
pub async fn run_preview_purger(pool: Arc<sqlx::PgPool>) {
    let mut tick = interval(Duration::from_secs(5 * 60));
    loop {
        tick.tick().await;
        match crate::repository::purge_expired_previews(&pool).await {
            Ok(n) => tracing::info!("purged {n} expired previews"),
            Err(e) => tracing::error!("preview purge error: {e}"),
        }
    }
}
