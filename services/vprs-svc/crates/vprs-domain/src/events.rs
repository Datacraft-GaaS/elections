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
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSubmitted {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl ApplicationSubmitted {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.application_submitted.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationCancelled {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl ApplicationCancelled {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.application_cancelled.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffRedeemed {
    pub event_type: String,
    pub app_id: String,
    pub occurred_at: DateTime<Utc>,
}

impl HandoffRedeemed {
    pub fn new(app_id: String) -> Self {
        Self {
            event_type: "eel.vprs.handoff_redeemed.v1".to_string(),
            app_id,
            occurred_at: Utc::now(),
        }
    }
}
