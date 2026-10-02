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

//! Parallel manual count service (HARD-20): 1% stratified station sample,
//! manual submissions, and divergence reporting against platform tallies.
//!
//! The sample plan is generated at startup from `EEL_PMC_SEED` (hex u64; a
//! fixed default keeps local runs reproducible) so the draw is verifiable by
//! anyone holding the seed.

use std::sync::Arc;

use eel_governance_parallel_manual_count::api::{AppState, PmcState};
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".parse().unwrap()),
        )
        .json()
        .init();

    let http_addr = std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:8097".to_string());
    let seed = std::env::var("EEL_PMC_SEED")
        .ok()
        .and_then(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16).ok())
        .unwrap_or(0x5EED_2026_08_09); // deterministic dev default
    let election_day =
        std::env::var("EEL_ELECTION_DAY").unwrap_or_else(|_| "2027-08-09".to_string());

    let plan = Arc::new(eel_governance_parallel_manual_count::SamplePlan::generate(
        &election_day, seed,
    ));
    info!(
        plan_id = %plan.plan_id,
        total_sampled = plan.total_sampled,
        %seed,
        "parallel-manual-count plan generated"
    );

    let state = AppState {
        plan,
        state: Arc::new(PmcState::new()),
    };

    let app = eel_governance_parallel_manual_count::api::router(state);
    let listener = tokio::net::TcpListener::bind(&http_addr).await?;
    info!(addr = %http_addr, "parallel-manual-count listening");
    axum::serve(listener, app).await?;
    Ok(())
}
