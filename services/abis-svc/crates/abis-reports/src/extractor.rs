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

use std::time::Instant;

use sqlx::{Column, PgPool, Row};
use tera::{Context, Tera};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info};
use uuid::Uuid;

use crate::error::{ReportError, Result};
use crate::render::Rows;

const MAX_ROWS_DEFAULT: u64 = 5_000_000;

/// Extract report data from a PG pool using a SQL template file.
pub async fn extract(
    pool: &PgPool,
    run_id: Uuid,
    sql_template_ref: &str,
    window_from: chrono::DateTime<chrono::Utc>,
    window_to: chrono::DateTime<chrono::Utc>,
    max_rows: Option<u64>,
    cancel: CancellationToken,
) -> Result<(Rows, i64, i32)> {
    let max_rows = max_rows.unwrap_or(MAX_ROWS_DEFAULT);
    let started = Instant::now();

    // Render the SQL template
    let sql_source = tokio::fs::read_to_string(sql_template_ref)
        .await
        .map_err(|e| {
            ReportError::Renderer(format!(
                "cannot read sql template '{sql_template_ref}': {e}"
            ))
        })?;

    let mut tera = Tera::default();
    tera.add_raw_template("query", &sql_source)?;
    let mut ctx = Context::new();
    ctx.insert("window_from", &window_from.to_rfc3339());
    ctx.insert("window_to", &window_to.to_rfc3339());
    ctx.insert("run_id", &run_id.to_string());
    let sql = tera.render("query", &ctx)?;

    debug!(run_id = %run_id, "executing sql template");

    let rows_pg = sqlx::query(&sql).fetch_all(pool).await?;

    if cancel.is_cancelled() {
        return Err(ReportError::Cancelled);
    }

    let row_count = rows_pg.len() as u64;
    if row_count > max_rows {
        return Err(ReportError::TooManyRows(row_count, max_rows));
    }

    // Column names from the first row
    let columns: Vec<String> = if let Some(first) = rows_pg.first() {
        first
            .columns()
            .iter()
            .map(|c| c.name().to_owned())
            .collect()
    } else {
        vec![]
    };

    // Convert rows
    let data: Vec<Vec<serde_json::Value>> = rows_pg
        .iter()
        .map(|row| pg_row_to_json(row, columns.len()))
        .collect();

    let elapsed_ms = started.elapsed().as_millis() as i32;
    let rows_extracted = data.len() as i64;

    info!(run_id = %run_id, rows = rows_extracted, elapsed_ms, "extraction complete");

    Ok((Rows::new(columns, data), rows_extracted, elapsed_ms))
}

fn pg_row_to_json(row: &sqlx::postgres::PgRow, col_count: usize) -> Vec<serde_json::Value> {
    (0..col_count)
        .map(|i| {
            // Try types in preference order; fall back to Null
            if let Ok(v) = row.try_get::<Option<i64>, _>(i) {
                return v
                    .map(|n| serde_json::Value::Number(n.into()))
                    .unwrap_or(serde_json::Value::Null);
            }
            if let Ok(v) = row.try_get::<Option<String>, _>(i) {
                return v
                    .map(serde_json::Value::String)
                    .unwrap_or(serde_json::Value::Null);
            }
            if let Ok(v) = row.try_get::<Option<f64>, _>(i) {
                return v
                    .and_then(|f| serde_json::Number::from_f64(f))
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null);
            }
            if let Ok(v) = row.try_get::<Option<bool>, _>(i) {
                return v
                    .map(serde_json::Value::Bool)
                    .unwrap_or(serde_json::Value::Null);
            }
            serde_json::Value::Null
        })
        .collect()
}
