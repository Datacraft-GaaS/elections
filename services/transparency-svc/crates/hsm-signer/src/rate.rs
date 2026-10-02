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

//! Per-role token-bucket rate limiters (Tokio governor).
//!
//! Limits per ADR-0005:
//! - Sth: 60/s
//! - Declaration: 10/s
//! - KitCa: 100/s
//! - CredentialCa: 100/s
//! - CodeSigning: 5/s
//! - CycleKek: 1/hour
//!
//! Over-limit callers should receive `RESOURCE_EXHAUSTED` with `Retry-After` metadata.

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use governor::clock::DefaultClock;
use governor::middleware::NoOpMiddleware;
use governor::state::{InMemoryState, NotKeyed};
use governor::{Quota, RateLimiter};

use eel_crypto_rs::KeyRole;

pub type Limiter = Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>;

/// Set of per-role rate limiters.
pub struct RoleLimiters {
    pub sth: Limiter,
    pub declaration: Limiter,
    pub kit_ca: Limiter,
    pub credential_ca: Limiter,
    pub code_signing: Limiter,
    pub cycle_kek: Limiter,
}

impl RoleLimiters {
    /// Construct limiters with the configured rates.
    pub fn new() -> Self {
        Self {
            sth: per_second(60),
            declaration: per_second(10),
            kit_ca: per_second(100),
            credential_ca: per_second(100),
            code_signing: per_second(5),
            cycle_kek: per_hour(1),
        }
    }

    /// Returns the limiter for a given role.
    pub fn get(&self, role: KeyRole) -> &Limiter {
        match role {
            KeyRole::Sth => &self.sth,
            KeyRole::Declaration => &self.declaration,
            KeyRole::KitCa => &self.kit_ca,
            KeyRole::CredentialCa => &self.credential_ca,
            KeyRole::CodeSigning => &self.code_signing,
            KeyRole::CycleKek => &self.cycle_kek,
            _ => &self.sth, // future roles: default to Sth rate
        }
    }

    /// Try to consume one token for the role.
    ///
    /// Returns `Ok(())` if allowed, or `Err(retry_after_secs)` if rate-limited.
    pub fn check(&self, role: KeyRole) -> Result<(), u64> {
        let limiter = self.get(role);
        limiter.check().map_err(|not_until| {
            use governor::clock::Clock;
            let wait = not_until.wait_time_from(DefaultClock::default().now());
            wait.as_secs().max(1)
        })
    }
}

impl Default for RoleLimiters {
    fn default() -> Self {
        Self::new()
    }
}

fn per_second(n: u32) -> Limiter {
    let quota = Quota::per_second(NonZeroU32::new(n).expect("rate must be nonzero"));
    Arc::new(RateLimiter::direct(quota))
}

fn per_hour(n: u32) -> Limiter {
    let quota =
        Quota::with_period(Duration::from_secs(3600 / n as u64)).expect("rate must be nonzero");
    Arc::new(RateLimiter::direct(quota))
}
