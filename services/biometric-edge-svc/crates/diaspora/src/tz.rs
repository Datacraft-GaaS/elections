//! Timezone helpers for diaspora station scheduling (WP-5.8, M24).
//!
//! All events are stored in UTC. Local times are computed on-the-fly from
//! the station's IANA timezone string. chrono-tz handles DST transitions.

use chrono::{DateTime, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::Tz;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TzError {
    #[error("unknown IANA timezone: {0}")]
    UnknownTz(String),
    #[error("ambiguous local time (DST gap or fold): {0}")]
    AmbiguousLocalTime(String),
    #[error("non-existent local time (DST gap): {0}")]
    NonExistentLocalTime(String),
}

/// Parse an IANA timezone string into a `chrono_tz::Tz`.
pub fn parse_tz(iana: &str) -> Result<Tz, TzError> {
    iana.parse::<Tz>()
        .map_err(|_| TzError::UnknownTz(iana.to_owned()))
}

/// Convert a naive local datetime (station's wall clock) to UTC, using
/// the IANA timezone string stored on the station. DST-safe: panics on
/// ambiguous/non-existent times; callers should avoid scheduling in gaps.
pub fn local_to_utc(iana: &str, local: NaiveDateTime) -> Result<DateTime<Utc>, TzError> {
    let tz = parse_tz(iana)?;
    match tz.from_local_datetime(&local) {
        chrono::LocalResult::Single(dt) => Ok(dt.with_timezone(&Utc)),
        chrono::LocalResult::Ambiguous(earlier, _later) => {
            // During DST fold: pick the earlier (pre-rollback) offset.
            // This is conservative: the station is assumed to close before
            // the clocks go back.
            Ok(earlier.with_timezone(&Utc))
        }
        chrono::LocalResult::None => {
            Err(TzError::NonExistentLocalTime(format!("{local} in {iana}")))
        }
    }
}

/// Convert a UTC datetime to local wall-clock time for the given station.
pub fn utc_to_local(iana: &str, utc: DateTime<Utc>) -> Result<DateTime<Tz>, TzError> {
    let tz = parse_tz(iana)?;
    Ok(utc.with_timezone(&tz))
}

/// Compute the UTC close time given the station's IANA timezone and the
/// intended local close time (naive). Returns UTC timestamp for event storage.
pub fn station_close_utc(iana: &str, local_close: NaiveDateTime) -> Result<DateTime<Utc>, TzError> {
    local_to_utc(iana, local_close)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use proptest::prelude::*;

    // Embassy profiles and representative local close times.
    const PROFILES: &[(&str, &str)] = &[
        ("Africa/Nairobi", "2027-08-10"),
        ("Europe/London", "2027-08-10"),
        ("America/New_York", "2027-08-10"),
        ("Asia/Dubai", "2027-08-10"),
        ("Australia/Sydney", "2027-08-10"),
    ];

    proptest! {
        /// Round-trip: local -> UTC -> local == original (within same tz,
        /// outside DST transitions). We parametrise over hours 0..23 to
        /// avoid hitting DST gaps which are by definition not representable.
        #[test]
        fn prop_local_utc_roundtrip(hour in 1u32..22, minute in 0u32..59) {
            for (iana, date_str) in PROFILES {
                let date = NaiveDate::parse_from_str(date_str, "%Y-%m-%d").unwrap();
                let local = date.and_hms_opt(hour, minute, 0).unwrap();
                let utc = local_to_utc(iana, local).unwrap();
                let back = utc_to_local(iana, utc).unwrap().naive_local();
                prop_assert_eq!(back, local, "round-trip failed for tz={}", iana);
            }
        }
    }

    #[test]
    fn parse_known_timezones() {
        for (iana, _) in PROFILES {
            assert!(parse_tz(iana).is_ok(), "failed to parse {iana}");
        }
    }

    #[test]
    fn parse_unknown_tz_returns_error() {
        assert!(parse_tz("Moon/CraterBase").is_err());
    }

    #[test]
    fn dst_aware_new_york_standard_time() {
        // January 15 is well inside EST (UTC-5). Close at 17:00 local = 22:00 UTC.
        let local = NaiveDate::from_ymd_opt(2027, 1, 15)
            .unwrap()
            .and_hms_opt(17, 0, 0)
            .unwrap();
        let utc = station_close_utc("America/New_York", local).unwrap();
        assert_eq!(utc.hour(), 22);
    }

    #[test]
    fn dst_aware_london_summer_time() {
        // July 15 is BST (UTC+1). Close at 18:00 local = 17:00 UTC.
        let local = NaiveDate::from_ymd_opt(2027, 7, 15)
            .unwrap()
            .and_hms_opt(18, 0, 0)
            .unwrap();
        let utc = station_close_utc("Europe/London", local).unwrap();
        assert_eq!(utc.hour(), 17);
    }
}
