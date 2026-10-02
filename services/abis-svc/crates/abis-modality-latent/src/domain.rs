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

//! Domain types for latent biometric templates.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Origin of a latent print.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LatentSource {
    CrimeScene,
    Lifted,
    Powdered,
    Chemical,
    Digital,
}

/// A single ridge feature point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Minutia {
    pub x: u16,
    pub y: u16,
    /// Orientation angle 0-179 degrees.
    pub angle: u8,
    /// Quality estimate 0-100.
    pub quality: u8,
    pub minutia_type: MinutiaType,
}

/// Classification of a minutia feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MinutiaType {
    RidgeEnding,
    Bifurcation,
}

/// A chain-of-custody event recording who touched the evidence and when.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustodyEvent {
    pub event_id: String,
    pub timestamp: DateTime<Utc>,
    pub actor_id: String,
    pub action: String,
    /// Base64-encoded signature, if present.
    pub signature: Option<String>,
}

/// A latent fingerprint biometric template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatentTemplate {
    pub template_id: String,
    pub source: LatentSource,
    pub lift_id: Option<String>,
    pub case_ref: Option<String>,
    /// Distortion score 0.0 (none) to 1.0 (fully distorted).
    pub distortion_score: f32,
    /// Fraction of the print with usable ridge detail.
    pub usable_ridge_pct: f32,
    pub examiner_id: Option<String>,
    pub chain_of_custody: Vec<CustodyEvent>,
    pub minutiae: Vec<Minutia>,
    pub captured_at: DateTime<Utc>,
}
