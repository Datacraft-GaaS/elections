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

//! Embassy diaspora roll import profiles and CSV validation (WP-5.8, M24).
//!
//! Five predefined profiles control which embassy stations are active per
//! region. A CSV import validates each row against the expected schema and
//! returns a per-row report without panicking on bad data.

use serde::{Deserialize, Serialize};

/// One of the five official import profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImportProfile {
    Nairobi,
    RestOfWorld,
    Dubai,
    London,
    Washington,
}

impl ImportProfile {
    /// IANA timezone for this profile's primary time reference.
    pub fn timezone(&self) -> &'static str {
        match self {
            Self::Nairobi => "Africa/Nairobi",
            Self::RestOfWorld => "UTC",
            Self::Dubai => "Asia/Dubai",
            Self::London => "Europe/London",
            Self::Washington => "America/New_York",
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Nairobi => "Nairobi",
            Self::RestOfWorld => "Rest of World",
            Self::Dubai => "Dubai",
            Self::London => "London",
            Self::Washington => "Washington D.C.",
        }
    }

    /// All five profiles in canonical order.
    pub fn all() -> [Self; 5] {
        [
            Self::Nairobi,
            Self::RestOfWorld,
            Self::Dubai,
            Self::London,
            Self::Washington,
        ]
    }
}

/// A validated row from the diaspora roll CSV.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RollRow {
    pub id: String,
    /// Last 4 digits of national ID.
    pub national_id_last4: String,
    /// NFC-readable name (normalised unicode).
    pub name_nfc: String,
    /// Year of birth (yyyy).
    pub dob_yyyy: u16,
    /// Station code matching an EmbassyStation.
    pub station_code: String,
}

/// Outcome for a single CSV row during import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowReport {
    pub line: usize,
    pub ok: bool,
    pub row: Option<RollRow>,
    pub error: Option<String>,
}

/// Result of importing a full CSV file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportReport {
    pub profile: ImportProfile,
    pub total_rows: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub rows: Vec<RowReport>,
}

/// Parse and validate a diaspora roll CSV.
///
/// Expected columns (header required): id, national_id_last4, name_nfc, dob_yyyy, station_code
pub fn import_roll_csv(profile: ImportProfile, csv: &str) -> ImportReport {
    let mut rows = Vec::new();
    let mut lines = csv.lines();

    // Validate header.
    let header = match lines.next() {
        Some(h) => h.trim().to_ascii_lowercase(),
        None => {
            return ImportReport {
                profile,
                total_rows: 0,
                accepted: 0,
                rejected: 0,
                rows: vec![RowReport {
                    line: 0,
                    ok: false,
                    row: None,
                    error: Some("empty CSV: no header row".to_owned()),
                }],
            };
        }
    };

    let expected = "id,national_id_last4,name_nfc,dob_yyyy,station_code";
    if header != expected {
        return ImportReport {
            profile,
            total_rows: 0,
            accepted: 0,
            rejected: 1,
            rows: vec![RowReport {
                line: 1,
                ok: false,
                row: None,
                error: Some(format!(
                    "invalid header; expected '{expected}', got '{header}'"
                )),
            }],
        };
    }

    for (idx, line) in lines.enumerate() {
        let line_num = idx + 2; // 1-based, skipping header
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let report = parse_row(line_num, line);
        rows.push(report);
    }

    let accepted = rows.iter().filter(|r| r.ok).count();
    let rejected = rows.iter().filter(|r| !r.ok).count();
    ImportReport {
        profile,
        total_rows: rows.len(),
        accepted,
        rejected,
        rows,
    }
}

fn parse_row(line: usize, raw: &str) -> RowReport {
    let parts: Vec<&str> = raw.splitn(5, ',').collect();
    if parts.len() < 5 {
        return RowReport {
            line,
            ok: false,
            row: None,
            error: Some(format!("expected 5 columns, found {}", parts.len())),
        };
    }
    let id = parts[0].trim().to_owned();
    let national_id_last4 = parts[1].trim().to_owned();
    let name_nfc = parts[2].trim().to_owned();
    let dob_yyyy_str = parts[3].trim();
    let station_code = parts[4].trim().to_owned();

    // Validate id is non-empty
    if id.is_empty() {
        return RowReport {
            line,
            ok: false,
            row: None,
            error: Some("id is empty".to_owned()),
        };
    }

    // Validate national_id_last4 is exactly 4 digits
    if national_id_last4.len() != 4 || !national_id_last4.chars().all(|c| c.is_ascii_digit()) {
        return RowReport {
            line,
            ok: false,
            row: None,
            error: Some(format!(
                "national_id_last4 must be 4 digits, got '{national_id_last4}'"
            )),
        };
    }

    // Validate dob_yyyy
    let dob_yyyy: u16 = match dob_yyyy_str.parse() {
        Ok(y) if (1900..=2010).contains(&y) => y,
        Ok(y) => {
            return RowReport {
                line,
                ok: false,
                row: None,
                error: Some(format!("dob_yyyy {y} out of range 1900-2010")),
            };
        }
        Err(_) => {
            return RowReport {
                line,
                ok: false,
                row: None,
                error: Some(format!("dob_yyyy '{dob_yyyy_str}' is not a valid year")),
            };
        }
    };

    if station_code.is_empty() {
        return RowReport {
            line,
            ok: false,
            row: None,
            error: Some("station_code is empty".to_owned()),
        };
    }

    RowReport {
        line,
        ok: true,
        row: Some(RollRow {
            id,
            national_id_last4,
            name_nfc,
            dob_yyyy,
            station_code,
        }),
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_CSV: &str = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,1234,Jane Doe,1980,US-DC-001
V002,5678,John Smith,1975,UK-LON-001";

    #[test]
    fn valid_csv_all_accepted() {
        let report = import_roll_csv(ImportProfile::Washington, VALID_CSV);
        assert_eq!(report.accepted, 2);
        assert_eq!(report.rejected, 0);
    }

    #[test]
    fn invalid_national_id_rejected() {
        let csv = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,AB34,Jane Doe,1980,US-DC-001";
        let report = import_roll_csv(ImportProfile::Washington, csv);
        assert_eq!(report.rejected, 1);
        assert!(report.rows[0]
            .error
            .as_ref()
            .unwrap()
            .contains("national_id_last4"));
    }

    #[test]
    fn invalid_dob_rejected() {
        let csv = "id,national_id_last4,name_nfc,dob_yyyy,station_code
V001,1234,Jane Doe,abcd,US-DC-001";
        let report = import_roll_csv(ImportProfile::Washington, csv);
        assert_eq!(report.rejected, 1);
    }

    #[test]
    fn wrong_header_rejected() {
        let csv = "voter_id,id4,name,year,code
V001,1234,Jane,1980,US-DC-001";
        let report = import_roll_csv(ImportProfile::London, csv);
        assert_eq!(report.rejected, 1);
        assert!(report.rows[0]
            .error
            .as_ref()
            .unwrap()
            .contains("invalid header"));
    }

    #[test]
    fn profile_timezones_are_valid_iana() {
        for profile in ImportProfile::all() {
            let tz = profile.timezone();
            assert!(
                tz.parse::<chrono_tz::Tz>().is_ok(),
                "timezone {tz} for profile {:?} is not a valid IANA tz",
                profile
            );
        }
    }
}
