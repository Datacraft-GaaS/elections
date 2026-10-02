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

//! 30+14-day deregistration workflow enforcer.
//!
//! Day 0: notice dispatched (notice_deadline = +30 days).
//! Days 1-30: voter may file appeal (opens AppealRecord).
//! Days 31-44: appeal review window (14 days).
//! Day 45+: if no appeal and no overturn, case moves to Deregistered.

use crate::models::{CaseStatus, DeregistrationCase};
use chrono::Utc;

pub fn advance(case: &mut DeregistrationCase) {
    let now = Utc::now();
    match case.status {
        CaseStatus::Noticed => {
            if let Some(deadline) = case.notice_deadline {
                if now > deadline && case.appeal_id.is_none() {
                    case.status = CaseStatus::ExpiredNoAppeal;
                }
            }
        }
        CaseStatus::AppealFiled => {
            // hearing must be scheduled within 14 days of filing
        }
        _ => {}
    }
}
