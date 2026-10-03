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

//! Tokio worker pool that polls for unclaimed jobs and drives them through the pipeline.
//!
//! Workers use SELECT FOR UPDATE SKIP LOCKED with a lease timeout so no two workers
//! process the same job concurrently. The claim query is in `repository::claim_next_job`.

use std::sync::Arc;
use std::time::Duration;

use sqlx::PgPool;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::EnrolmentStatus;
use crate::repository::claim_next_job;

/// Configuration for the orchestrator worker pool.
pub struct OrchestratorConfig {
    /// Number of parallel workers.
    pub worker_count: usize,
    /// Seconds a claimed job is locked before it can be re-claimed by another worker.
    pub lease_seconds: i64,
    /// How long each worker sleeps between poll cycles when no work was found.
    pub poll_interval: Duration,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            worker_count: 4,
            lease_seconds: 120,
            poll_interval: Duration::from_millis(500),
        }
    }
}

/// Start `config.worker_count` tokio tasks that each poll for unclaimed Received jobs
/// and advance them through the pipeline stages.
///
/// Returns a `JoinHandle` for each spawned task. In production, the supervisor
/// process waits on these handles. In tests, drop the handles to stop workers.
pub fn start_workers(
    pool: Arc<PgPool>,
    config: Arc<OrchestratorConfig>,
) -> Vec<tokio::task::JoinHandle<()>> {
    let statuses = vec![EnrolmentStatus::Received, EnrolmentStatus::DedupPending];

    (0..config.worker_count)
        .map(|i| {
            let pool = Arc::clone(&pool);
            let config = Arc::clone(&config);
            let worker_id = format!("worker-{}", Uuid::now_v7());
            let statuses = statuses.clone();

            tokio::spawn(async move {
                info!(worker = %worker_id, idx = i, "enrolment worker started");

                loop {
                    let mut claimed_any = false;

                    for status in &statuses {
                        match claim_next_job(&pool, status, &worker_id, config.lease_seconds).await
                        {
                            Ok(Some(row)) => {
                                claimed_any = true;
                                info!(
                                    worker = %worker_id,
                                    job = %row.id,
                                    status = %row.status,
                                    "claimed job"
                                );
                                // In a full implementation, the worker would call the matcher
                                // service here, apply quality/dedup results, and persist state.
                                // This skeleton logs the claim and releases back to the queue
                                // after lease expiry, making the orchestrator testable without
                                // a real matcher.
                            }
                            Ok(None) => {}
                            Err(e) => {
                                warn!(worker = %worker_id, err = %e, "claim_next_job failed");
                            }
                        }
                    }

                    if !claimed_any {
                        tokio::time::sleep(config.poll_interval).await;
                    }
                }
            })
        })
        .collect()
}
