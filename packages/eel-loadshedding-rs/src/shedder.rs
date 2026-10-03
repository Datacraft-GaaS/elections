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

use crate::{metrics::ShedMetrics, priority::Priority};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

/// Configuration for the CoDel shedder.
#[derive(Debug, Clone)]
pub struct CoDelConfig {
    /// Target sojourn time / max acceptable latency (per SLO).
    pub target_latency: Duration,
    /// Observation interval. If target breached continuously for this long, start shedding.
    pub interval: Duration,
    /// Maximum concurrent inflight requests.
    pub max_inflight: usize,
    /// Minimum priority to shed (requests at or below this are candidates).
    pub shed_below: Priority,
    /// Retry-After value in seconds returned on 503.
    pub retry_after_secs: u64,
}

impl Default for CoDelConfig {
    fn default() -> Self {
        Self {
            target_latency: Duration::from_millis(100),
            interval: Duration::from_millis(100),
            max_inflight: 500,
            shed_below: Priority::Anonymous,
            retry_after_secs: 1,
        }
    }
}

#[derive(Debug)]
struct CoDelState {
    /// When we first observed a target breach in this interval.
    first_above_time: Option<Instant>,
    /// Whether we are currently in shedding mode.
    shedding: bool,
    /// Count of consecutive drops.
    drop_count: u32,
}

/// CoDel shedder. Tracks inflight count and latency target.
#[derive(Clone)]
pub struct CoDelShedder {
    pub config: CoDelConfig,
    pub metrics: ShedMetrics,
    inflight: Arc<Semaphore>,
    state: Arc<Mutex<CoDelState>>,
}

impl CoDelShedder {
    pub fn new(config: CoDelConfig) -> Self {
        let inflight = Arc::new(Semaphore::new(config.max_inflight));
        Self {
            inflight,
            config,
            metrics: ShedMetrics::new(),
            state: Arc::new(Mutex::new(CoDelState {
                first_above_time: None,
                shedding: false,
                drop_count: 0,
            })),
        }
    }

    /// Attempt to admit a request of the given priority.
    /// Returns a permit if admitted, or an error (retry_after_secs) if shed.
    pub async fn admit(
        &self,
        priority: Priority,
        request_start: Instant,
    ) -> Result<CoDelPermit, u64> {
        let depth = self.config.max_inflight - self.inflight.available_permits();
        self.metrics.set_queue_depth(depth as u64);

        // Check if sojourn time exceeded target
        let sojourn = request_start.elapsed();
        let should_shed = self.should_shed(sojourn, priority).await;

        if should_shed {
            self.metrics.record_shed(priority);
            return Err(self.config.retry_after_secs);
        }

        // Try to acquire inflight slot
        match self.inflight.clone().try_acquire_owned() {
            Ok(permit) => Ok(CoDelPermit {
                _permit: permit,
                _start: Instant::now(),
            }),
            Err(_) => {
                // At max inflight - shed lowest priority
                self.metrics.record_shed(priority);
                Err(self.config.retry_after_secs)
            }
        }
    }

    async fn should_shed(&self, sojourn: Duration, priority: Priority) -> bool {
        let mut state = self.state.lock().await;
        let now = Instant::now();

        if sojourn < self.config.target_latency {
            // Below target, reset
            state.first_above_time = None;
            if state.shedding && state.drop_count > 2 {
                state.shedding = false;
                state.drop_count = 0;
            }
            return false;
        }

        // Above target
        self.metrics.record_breach();
        match state.first_above_time {
            None => {
                state.first_above_time = Some(now);
                false
            }
            Some(first) if now.duration_since(first) < self.config.interval => false,
            _ => {
                // Been above target for > interval
                state.shedding = true;
                // Only shed if priority is at or below shed_below threshold
                priority <= self.config.shed_below
            }
        }
    }

    pub fn inflight_count(&self) -> usize {
        self.config.max_inflight - self.inflight.available_permits()
    }
}

/// RAII inflight permit.
pub struct CoDelPermit {
    _permit: tokio::sync::OwnedSemaphorePermit,
    _start: Instant,
}
