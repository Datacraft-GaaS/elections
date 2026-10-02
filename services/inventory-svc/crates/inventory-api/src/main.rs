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

use std::env;
use tracing::info;

/// Migration files applied in order. The repo names them `V###__name.sql`,
/// which the sqlx `migrate!` macro cannot parse, so we embed and run them
/// manually (each file is idempotent via `CREATE TABLE IF NOT EXISTS`).
const MIGRATIONS: &[(&str, &str)] = &[
    (
        "V001__custody_core",
        include_str!("../../../migrations/V001__custody_core.sql"),
    ),
    (
        "V002__printing_chain",
        include_str!("../../../migrations/V002__printing_chain.sql"),
    ),
    (
        "V003__diaspora_logistics",
        include_str!("../../../migrations/V003__diaspora_logistics.sql"),
    ),
    (
        "V004__transport_security",
        include_str!("../../../migrations/V004__transport_security.sql"),
    ),
    (
        "V005__warehouse",
        include_str!("../../../migrations/V005__warehouse.sql"),
    ),
    (
        "V051__rfid_ballot_covers",
        include_str!("../../../migrations/V051__rfid_ballot_covers.sql"),
    ),
];

const SERVICE_NAME: &str = "inventory-svc";

async fn run_migrations(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    // Track applied migrations by (service, name) so restarts are idempotent
    // and the repo-wide `eel-migrate` tool shares the same ledger.
    sqlx::raw_sql(
        "CREATE TABLE IF NOT EXISTS _eel_migrations (\
           service    TEXT        NOT NULL, \
           name       TEXT        NOT NULL, \
           applied_at TIMESTAMPTZ NOT NULL DEFAULT now(), \
           PRIMARY KEY (service, name))",
    )
    .execute(pool)
    .await?;

    for (name, sql) in MIGRATIONS {
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _eel_migrations WHERE service = $1 AND name = $2)",
        )
        .bind(SERVICE_NAME)
        .bind(name)
        .fetch_one(pool)
        .await?;
        if applied {
            info!(migration = name, "already applied");
            continue;
        }

        let mut tx = pool.begin().await?;
        sqlx::raw_sql(sql).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO _eel_migrations (service, name) VALUES ($1, $2)")
            .bind(SERVICE_NAME)
            .bind(name)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        info!(migration = name, "applied");
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("inventory_api=info".parse().unwrap()),
        )
        .json()
        .init();

    let addr = env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:8081".to_string());
    info!(addr, "inventory-api starting");

    // Postgres when DATABASE_URL is set (migrations applied at startup);
    // otherwise an in-memory store so the service runs out of the box.
    let store = match env::var("DATABASE_URL") {
        Ok(url) => {
            let pool = sqlx::postgres::PgPoolOptions::new()
                .max_connections(10)
                .connect(&url)
                .await
                .expect("database connect failed");
            run_migrations(&pool).await.expect("migration failed");
            info!("postgres store connected, migrations applied");
            inventory_api::store::Store::Pg(pool)
        }
        Err(_) => {
            info!("DATABASE_URL unset; using in-memory store");
            inventory_api::store::Store::default()
        }
    };

    // JWT + SpiceDB authorization when both are configured.
    let app = match inventory_api::auth::Auth::from_env() {
        Some(auth) => {
            info!("auth enabled (Keycloak JWT + SpiceDB)");
            inventory_api::router::build_authed_router(store, auth)
        }
        None => {
            info!("auth disabled (SPICEDB_ENDPOINT / JWKS_URL unset)");
            inventory_api::router::build_router(store)
        }
    };

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("bind failed");
    info!(addr, "inventory-api HTTP listening");
    axum::serve(listener, app)
        .await
        .expect("HTTP server failed");
}
