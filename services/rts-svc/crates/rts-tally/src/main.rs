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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("rts_tally=info".parse().unwrap()),
        )
        .json()
        .init();
    let _kafka_brokers = env::var("KAFKA_BROKERS").unwrap_or_else(|_| "localhost:9092".to_string());
    let _tlog_endpoint =
        env::var("TLOG_ENDPOINT").unwrap_or_else(|_| "http://localhost:50052".to_string());
    let _db_url = env::var("DB_URL").expect("DB_URL must be set");
    info!("rts-tally starting (stub mode)");
    tokio::signal::ctrl_c().await.ok();
    info!("rts-tally shutting down");
}
