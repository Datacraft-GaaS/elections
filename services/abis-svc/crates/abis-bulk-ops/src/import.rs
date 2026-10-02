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

//! Bulk import from watchlist CSV.

use crate::error::BulkOpsError;
use csv::Reader;

/// A watchlist entry parsed from CSV.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WatchlistEntry {
    pub subject_id: String,
    pub name: String,
    pub national_id: String,
    pub risk_level: String,
}

/// Parse watchlist entries from CSV bytes.
pub fn parse_watchlist_csv(data: &[u8]) -> Result<Vec<WatchlistEntry>, BulkOpsError> {
    let mut rdr = Reader::from_reader(data);
    let mut entries = Vec::new();
    for result in rdr.deserialize() {
        let entry: WatchlistEntry = result?;
        entries.push(entry);
    }
    Ok(entries)
}
