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

//! Repo-wide migration runner.
//!
//! Applies every `services/*/migrations/*.sql` and crate-level
//! `services/*/crates/*/migrations/*.sql` to the database named by
//! `DATABASE_URL`, in (service, version) order, tracking applied migrations in
//! `_eel_migrations`. Handles the `V###` filename convention that the sqlx
//! `migrate` CLI cannot parse, and is idempotent across runs.

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use tracing::info;

#[derive(Debug)]
struct Migration {
    service: String,
    version: u64,
    name: String,
    path: PathBuf,
}

/// Extract the numeric version from a migration filename of the form
/// `V001__foo.sql`, `V1__foo.sql`, `V0001__foo.sql`, or `001__foo.sql`.
fn version_of(filename: &str) -> u64 {
    let stem = filename.strip_suffix(".sql").unwrap_or(filename);
    let digits = stem.trim_start_matches(|c: char| !c.is_ascii_digit());
    let n: String = digits.chars().take_while(|c| c.is_ascii_digit()).collect();
    n.parse().unwrap_or(0)
}

/// Scan `services/*/migrations/*.sql` and crate-level
/// `services/*/crates/*/migrations/*.sql`, returning them sorted by
/// (service, version, name).
fn discover(services_dir: &str) -> Result<Vec<Migration>> {
    let mut out = Vec::new();
    for svc_entry in std::fs::read_dir(services_dir)
        .with_context(|| format!("read services dir {services_dir:?}"))?
    {
        let svc_entry = svc_entry?;
        if !svc_entry.file_type()?.is_dir() {
            continue;
        }
        let svc_name = svc_entry.file_name().to_string_lossy().to_string();
        let svc_path = svc_entry.path();

        let mig_dir = svc_path.join("migrations");
        if mig_dir.is_dir() {
            collect_dir(&mut out, &svc_name, &mig_dir)?;
        }

        let crates_dir = svc_path.join("crates");
        if crates_dir.is_dir() {
            for crate_entry in std::fs::read_dir(&crates_dir)
                .with_context(|| format!("read crates dir {crates_dir:?}"))?
            {
                let crate_entry = crate_entry?;
                if !crate_entry.file_type()?.is_dir() {
                    continue;
                }
                let crate_name = crate_entry.file_name().to_string_lossy().to_string();
                let crate_mig = crate_entry.path().join("migrations");
                if crate_mig.is_dir() {
                    collect_dir(&mut out, &format!("{svc_name}/{crate_name}"), &crate_mig)?;
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.service, a.version, &a.name).cmp(&(&b.service, b.version, &b.name)));
    Ok(out)
}

fn collect_dir(out: &mut Vec<Migration>, service: &str, dir: &std::path::Path) -> Result<()> {
    for file in std::fs::read_dir(dir).with_context(|| format!("read migrations dir {dir:?}"))? {
        let file = file?;
        let name = file.file_name().to_string_lossy().to_string();
        if !name.ends_with(".sql") {
            continue;
        }
        out.push(Migration {
            service: service.to_string(),
            version: version_of(&name),
            name,
            path: file.path(),
        });
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("eel_migrate=info".parse().unwrap()),
        )
        .init();

    let url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let services_dir = std::env::var("EEL_SERVICES_DIR").unwrap_or_else(|_| "services".to_string());

    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .context("connect to Postgres")?;

    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _eel_migrations (\
           service    TEXT        NOT NULL, \
           name       TEXT        NOT NULL, \
           applied_at TIMESTAMPTZ NOT NULL DEFAULT now(), \
           PRIMARY KEY (service, name))",
    )
    .execute(&pool)
    .await
    .context("create _eel_migrations")?;

    let migrations = discover(&services_dir)?;
    if migrations.is_empty() {
        bail!("no *.sql migrations found under {services_dir:?}");
    }
    info!(count = migrations.len(), "discovered migrations");

    let mut applied = 0usize;
    let mut skipped = 0usize;
    for m in &migrations {
        let already: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _eel_migrations WHERE service = $1 AND name = $2)",
        )
        .bind(&m.service)
        .bind(&m.name)
        .fetch_one(&pool)
        .await?;

        if already {
            skipped += 1;
            continue;
        }

        let sql = std::fs::read_to_string(&m.path)
            .with_context(|| format!("read {}", m.path.display()))?;

        let mut tx = pool.begin().await?;
        sqlx::raw_sql(&sql)
            .execute(&mut *tx)
            .await
            .with_context(|| format!("apply {}/{}", m.service, m.name))?;
        sqlx::query("INSERT INTO _eel_migrations (service, name) VALUES ($1, $2)")
            .bind(&m.service)
            .bind(&m.name)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;

        info!(service = %m.service, migration = %m.name, "applied");
        applied += 1;
    }

    info!(applied, skipped, "migration run complete");
    Ok(())
}
