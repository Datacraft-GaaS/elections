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

//! Public contract models (gazette/calendar.yaml inspection surface).

use serde::{Deserialize, Serialize};

/// Public inspection-window view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionWindow {
    pub window_id: String,
    pub opens_at: String,
    pub closes_at: String,
    pub status: String,
    pub venues: Vec<String>,
}

/// Public list-objection view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListObjection {
    pub objection_id: String,
    pub reference_code: String,
    pub status: String,
    pub submitted_at: String,
}

/// Public objection status view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectionStatus {
    pub reference_code: String,
    pub status: String,
    pub updated_at: String,
}
