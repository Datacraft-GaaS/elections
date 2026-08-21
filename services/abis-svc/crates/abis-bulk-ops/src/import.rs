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
