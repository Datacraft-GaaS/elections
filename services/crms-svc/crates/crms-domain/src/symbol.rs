//! Party symbol and colour registry — WP-4.11, M67.
//! Enforces uniqueness of symbol names and colour assignments per cycle.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle of a registered party symbol (M67).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SymbolStatus {
    Draft,
    UnderReview,
    Approved,
    Rejected,
    Reassigned,
}

impl Default for SymbolStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// A registered party symbol, colour, and motto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartySymbol {
    pub id: Uuid,
    pub party_id: Uuid,
    pub symbol_name: String,
    pub symbol_url: String,
    pub colour_hex: String,
    pub status: SymbolStatus,
    pub assigned_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl PartySymbol {
    pub fn new(
        party_id: Uuid,
        symbol_name: String,
        symbol_url: String,
        colour_hex: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            party_id,
            symbol_name,
            symbol_url,
            colour_hex,
            status: SymbolStatus::Draft,
            assigned_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn approve(&mut self) {
        self.status = SymbolStatus::Approved;
        self.assigned_at = Some(Utc::now());
    }

    pub fn reject(&mut self) {
        self.status = SymbolStatus::Rejected;
    }
}

/// Availability check (FR-M67): a symbol name and colour are each unique
/// across registered parties in a cycle.
pub fn check_symbol_availability(
    existing: &[PartySymbol],
    symbol_name: &str,
    colour_hex: &str,
) -> Result<(), String> {
    if existing
        .iter()
        .any(|s| s.symbol_name.eq_ignore_ascii_case(symbol_name))
    {
        return Err(format!("symbol name already registered: {symbol_name}"));
    }
    if existing
        .iter()
        .any(|s| s.colour_hex.eq_ignore_ascii_case(colour_hex))
    {
        return Err(format!("colour already assigned: {colour_hex}"));
    }
    Ok(())
}
