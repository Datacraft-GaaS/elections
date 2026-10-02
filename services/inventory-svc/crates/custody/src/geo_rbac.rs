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

/// Hierarchical geographic scope for role-based access control.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "type", content = "code")]
pub enum GeographicScope {
    /// variant.
    National,
    /// variant.
    County(String),
    /// variant.
    Constituency(String),
    /// variant.
    Station(String),
}

/// Returns true when the actor's scope permits access to the given station code.
///
/// Rules:
/// - National: access to any station.
/// - County(c): access when the station code starts with the county prefix.
/// - Constituency(c): access when the station code starts with the constituency prefix.
/// - Station(s): access only to the exact station.
pub fn check_access(actor_scope: &GeographicScope, asset_station: &str) -> bool {
    match actor_scope {
        GeographicScope::National => true,
        GeographicScope::County(county) => asset_station.starts_with(county.as_str()),
        GeographicScope::Constituency(const_) => asset_station.starts_with(const_.as_str()),
        GeographicScope::Station(station) => asset_station == station.as_str(),
    }
}
