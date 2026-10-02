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

use chrono::{DateTime, Utc};

pub const MAX_REQUESTS: u32 = 3;
pub const LOCKOUT_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
pub struct OtpRequest {
    pub session_id: String,
    pub attempt_count: u32,
    pub locked_until: Option<DateTime<Utc>>,
}

impl OtpRequest {
    pub fn is_locked(&self) -> bool {
        self.locked_until.map(|t| t > Utc::now()).unwrap_or(false)
    }

    pub fn is_rate_limited(&self) -> bool {
        self.attempt_count >= MAX_REQUESTS
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtpVerifyResult {
    Ok,
    RateLimited,
    Invalid,
    Locked,
}
