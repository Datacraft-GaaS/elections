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

//! Turnout rollup reads over `turnout_rollups` (V4).
//!
//! Scope resolution is pure logic; aggregation runs runtime sqlx queries so
//! nothing here depends on compile-time database state.
#![warn(missing_docs)]

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Row};

/// Requested geographic scope for a turnout query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TurnoutScope {
    National,
    County(String),
    Constituency(String),
    Station(String),
}

impl TurnoutScope {
    /// Wire value of the `scope` discriminator on TurnoutSummary.
    pub fn as_str(&self) -> &'static str {
        match self {
            TurnoutScope::National => "national",
            TurnoutScope::County(_) => "county",
            TurnoutScope::Constituency(_) => "constituency",
            TurnoutScope::Station(_) => "station",
        }
    }

    /// Scope code carried by the rollup rows (KE for national).
    pub fn scope_code(&self) -> &str {
        match self {
            TurnoutScope::National => "KE",
            TurnoutScope::County(c)
            | TurnoutScope::Constituency(c)
            | TurnoutScope::Station(c) => c,
        }
    }
}

/// Resolve query parameters into a scope. Most specific parameter wins.
pub fn resolve_scope(
    county_code: Option<&str>,
    constituency_code: Option<&str>,
    station_code: Option<&str>,
) -> TurnoutScope {
    if let Some(s) = station_code.filter(|s| !s.is_empty()) {
        return TurnoutScope::Station(s.to_owned());
    }
    if let Some(c) = constituency_code.filter(|s| !s.is_empty()) {
        return TurnoutScope::Constituency(c.to_owned());
    }
    if let Some(c) = county_code.filter(|s| !s.is_empty()) {
        return TurnoutScope::County(c.to_owned());
    }
    TurnoutScope::National
}

/// Turnout percentage at two decimal places; zero when nobody registered.
pub fn turnout_pct(identified: i64, registered: i64) -> f64 {
    if registered <= 0 {
        return 0.0;
    }
    // Scale first to keep two-decimal precision without f64 drift.
    ((identified * 10_000 / registered) as f64) / 100.0
}

/// Aggregated turnout summary (contract schema TurnoutSummary).
#[derive(Debug, Clone, Serialize)]
pub struct TurnoutSummary {
    pub election_id: String,
    pub scope: String,
    pub scope_code: String,
    pub total_identified: i64,
    pub total_registered: i64,
    pub turnout_pct: f64,
    pub as_of: DateTime<Utc>,
}

struct RollupTotals {
    identified: i64,
    registered: i64,
    as_of: Option<DateTime<Utc>>,
}

async fn sum_station_rows(pool: &PgPool, prefix: Option<&str>) -> sqlx::Result<RollupTotals> {
    let row = match prefix {
        Some(p) => {
            let pattern = format!("{p}%");
            sqlx::query(
                r#"
                SELECT COALESCE(SUM(authenticated_count), 0)::BIGINT AS ident,
                       COALESCE(SUM(registered_count), 0)::BIGINT AS reg,
                       MAX(updated_at) AS as_of
                FROM turnout_rollups
                WHERE scope = 'station' AND scope_code LIKE $1
                "#,
            )
            .bind(&pattern)
            .fetch_one(pool)
            .await?
        }
        None => {
            sqlx::query(
                r#"
                SELECT COALESCE(SUM(authenticated_count), 0)::BIGINT AS ident,
                       COALESCE(SUM(registered_count), 0)::BIGINT AS reg,
                       MAX(updated_at) AS as_of
                FROM turnout_rollups
                WHERE scope = 'station'
                "#,
            )
            .fetch_one(pool)
            .await?
        }
    };
    Ok(RollupTotals {
        identified: row.try_get("ident")?,
        registered: row.try_get("reg")?,
        as_of: row.try_get("as_of")?,
    })
}

/// Exact-match read of one rollup row at a given scope level.
async fn exact_row(
    pool: &PgPool,
    scope: &str,
    scope_code: &str,
) -> sqlx::Result<Option<RollupTotals>> {
    let row = sqlx::query(
        r#"
        SELECT authenticated_count AS ident, registered_count AS reg, updated_at AS as_of
        FROM turnout_rollups
        WHERE scope = $1 AND scope_code = $2
        "#,
    )
    .bind(scope)
    .bind(scope_code)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| RollupTotals {
        identified: r.try_get("ident").unwrap_or(0),
        registered: r.try_get("reg").unwrap_or(0),
        as_of: r.try_get("as_of").ok(),
    }))
}

fn finish(election_id: &str, scope: &TurnoutScope, t: RollupTotals) -> TurnoutSummary {
    let as_of = t.as_of.unwrap_or_else(Utc::now);
    TurnoutSummary {
        election_id: election_id.to_owned(),
        scope: scope.as_str().to_owned(),
        scope_code: scope.scope_code().to_owned(),
        total_identified: t.identified,
        total_registered: t.registered,
        turnout_pct: turnout_pct(t.identified, t.registered),
        as_of,
    }
}

/// Compute the turnout summary for a resolved scope.
///
/// Precedence per branch: exact pre-aggregated row first, then fall back to
/// summing child rows whose codes share the parent code as prefix.
pub async fn turnout_summary(
    pool: &PgPool,
    election_id: &str,
    scope: &TurnoutScope,
) -> sqlx::Result<TurnoutSummary> {
    let totals = match scope {
        TurnoutScope::National => {
            match exact_row(pool, "national", "KE").await? {
                Some(t) => t,
                None => sum_station_rows(pool, None).await?,
            }
        }
        TurnoutScope::County(code) => {
            if let Some(t) = exact_row(pool, "county", code).await? {
                t
            } else if let Some(t) = exact_row(pool, "constituency", code).await? {
                // County code reused at constituency granularity.
                t
            } else {
                sum_station_rows(pool, Some(code)).await?
            }
        }
        TurnoutScope::Constituency(code) => match exact_row(pool, "constituency", code).await? {
            Some(t) => t,
            None => sum_station_rows(pool, Some(code)).await?,
        },
        TurnoutScope::Station(code) => match exact_row(pool, "station", code).await? {
            Some(t) => t,
            None => RollupTotals { identified: 0, registered: 0, as_of: None },
        },
    };
    Ok(finish(election_id, scope, totals))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn most_specific_scope_wins() {
        assert_eq!(
            resolve_scope(Some("KE-030"), Some("KE-030-001"), Some("POLL-NRB-001")),
            TurnoutScope::Station("POLL-NRB-001".into())
        );
        assert_eq!(
            resolve_scope(Some("KE-030"), Some("KE-030-001"), None),
            TurnoutScope::Constituency("KE-030-001".into())
        );
        assert_eq!(
            resolve_scope(Some("KE-030"), None, None),
            TurnoutScope::County("KE-030".into())
        );
        assert_eq!(resolve_scope(None, None, None), TurnoutScope::National);
    }

    #[test]
    fn empty_strings_do_not_narrow_scope() {
        assert_eq!(
            resolve_scope(None, Some(""), Some("")),
            TurnoutScope::National
        );
    }

    #[test]
    fn pct_math() {
        assert!((turnout_pct(4_182_930, 22_026_000) - 18.99).abs() < 1e-9);
        assert_eq!(turnout_pct(1, 0), 0.0);
        assert_eq!(turnout_pct(0, 100), 0.0);
        assert_eq!(turnout_pct(50, 100), 50.0);
    }

    #[test]
    fn scope_wire_values() {
        assert_eq!(TurnoutScope::National.as_str(), "national");
        assert_eq!(TurnoutScope::National.scope_code(), "KE");
        assert_eq!(
            TurnoutScope::County("KE-030".into()).scope_code(),
            "KE-030"
        );
    }
}
