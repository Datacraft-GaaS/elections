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

use serde::{Deserialize, Serialize};

/// Extracted from a validated JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Keycloak `sub` claim (UUID string).
    pub sub: String,
    /// Keycloak realm name.
    pub realm: String,
    /// Keycloak session ID (`sid` claim).
    pub session_id: Option<String>,
    /// Roles from `realm_access.roles`.
    pub roles: Vec<String>,
    /// Purpose scopes parsed from `scope` claim.
    pub purpose_scopes: Vec<String>,
}
