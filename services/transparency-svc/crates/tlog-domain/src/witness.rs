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
use uuid::Uuid;

/// A cosignature produced by a witness on an STH.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WitnessCosignature {
    pub id: Uuid,
    pub sth_id: Uuid,
    pub witness_id: String,
    /// SPIFFE URI of the witness service.
    pub witness_spiffe_id: String,
    /// Ed25519 or HSM signature over the STH bytes (hex-encoded).
    pub signature: String,
    pub cosigned_at: DateTime<Utc>,
    /// Whether this was produced by an internal (IAB) or external witness.
    pub is_internal: bool,
}
