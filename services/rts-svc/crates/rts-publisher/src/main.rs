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

use std::env;
use tracing::{error, info};
use rts_tally::tlog_client::TlogClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter(
        tracing_subscriber::EnvFilter::from_default_env()
            .add_directive("rts_publisher=info".parse().unwrap()),
    ).json().init();
    let db_url = env::var("DB_URL").expect("DB_URL must be set");
    let interval_secs: u64 = env::var("PUBLICATION_INTERVAL_SECS").ok().and_then(|v| v.parse().ok()).unwrap_or(300);
    let tlog = match TlogClient::from_env() {
        Ok(client) => Some(client),
        Err(e) => {
            error!("transparency-log anchoring disabled at startup: {e}");
            None
        }
    };
    info!(interval_secs, anchoring = tlog.is_some(), "rts-publisher starting");
    let pool = sqlx::PgPool::connect(&db_url).await.expect("failed to connect to DB");
    rts_publisher::scheduler::run_publication_loop(&pool, interval_secs, tlog).await;
}
