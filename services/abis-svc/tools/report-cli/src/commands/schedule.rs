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

use crate::api_client::ReportsClient;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct ScheduleEntry {
    run_at: String,
    window_from: String,
    window_to: String,
}

pub async fn run(client: &ReportsClient, slug: &str, next: u32) -> Result<()> {
    let entries: Vec<ScheduleEntry> = client
        .get_json(&format!("/api/v1/reports/definitions/{slug}/schedule?next={next}"))
        .await?;

    if entries.is_empty() {
        println!("No scheduled runs found (report may be on-demand only).");
        return Ok(());
    }

    println!("{:<30} {:<27} {}", "RUN_AT", "WINDOW_FROM", "WINDOW_TO");
    println!("{}", "-".repeat(90));
    for e in &entries {
        println!("{:<30} {:<27} {}", e.run_at, e.window_from, e.window_to);
    }
    Ok(())
}
