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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VeinType {
    FingerVein,
    PalmVein,
    WristVein,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BodySide {
    R,
    L,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VeinTemplate {
    pub template_id: String,
    pub vein_type: VeinType,
    pub body_side: BodySide,
    pub nir_wavelength_nm: u16,
    pub width: u16,
    pub height: u16,
    /// Grayscale NIR image bytes (width * height).
    pub pattern_bitmap: Vec<u8>,
    pub captured_at: DateTime<Utc>,
}
