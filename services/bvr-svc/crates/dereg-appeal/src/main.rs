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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/dereg_appeal".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&db_url)
        .await
        .expect("connect to dereg-appeal database");
    let app = dereg_appeal::app(pool);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
    tracing::info!("dereg-appeal listening on :8083");
    axum::serve(listener, app).await.unwrap();
}
