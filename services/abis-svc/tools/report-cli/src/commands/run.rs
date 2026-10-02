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

use crate::api_client::{ReportRun, ReportsClient, RunRequest};
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

const TERMINAL_STATUSES: &[&str] = &["complete", "failed", "cancelled"];

pub async fn run(
    client: &ReportsClient,
    slug: &str,
    from: &str,
    to: &str,
    formats: &[String],
    wait: bool,
) -> Result<()> {
    let req = RunRequest {
        slug: slug.to_string(),
        window_from: from.to_string(),
        window_to: to.to_string(),
        formats: formats.to_vec(),
        requested_by: std::env::var("USER").unwrap_or_else(|_| "operator".to_string()),
    };

    let run: ReportRun = client.post_json("/api/v1/reports/runs", &req).await?;
    println!("Run queued: {} (status: {})", run.run_id, run.status);

    if !wait {
        return Ok(());
    }

    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("Waiting for run {} ...", run.run_id));
    pb.enable_steady_tick(Duration::from_millis(120));

    let run_id = run.run_id.clone();
    loop {
        tokio::time::sleep(Duration::from_secs(3)).await;
        let current: ReportRun = client
            .get_json(&format!("/api/v1/reports/runs/{run_id}"))
            .await?;
        pb.set_message(format!("run={run_id} status={}", current.status));
        if TERMINAL_STATUSES.contains(&current.status.as_str()) {
            pb.finish_and_clear();
            match current.status.as_str() {
                "complete" => {
                    println!(
                        "Complete: {} rows in {}ms",
                        current.rows_extracted.unwrap_or(0),
                        current.total_ms.unwrap_or(0)
                    );
                }
                "failed" => {
                    eprintln!(
                        "Run failed: {}",
                        current.failure_reason.as_deref().unwrap_or("unknown")
                    );
                    std::process::exit(1);
                }
                _ => println!("Run ended with status: {}", current.status),
            }
            break;
        }
    }
    Ok(())
}
