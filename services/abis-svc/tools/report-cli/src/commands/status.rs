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

use crate::api_client::{ReportRun, ReportsClient};
use anyhow::Result;

pub async fn run(client: &ReportsClient, run_id: &str) -> Result<()> {
    let run: ReportRun = client
        .get_json(&format!("/api/v1/reports/runs/{run_id}"))
        .await?;
    println!("run_id:        {}", run.run_id);
    println!("slug:          {}", run.slug);
    println!("status:        {}", run.status);
    println!("window_from:   {}", run.window_from);
    println!("window_to:     {}", run.window_to);
    if let Some(rows) = run.rows_extracted {
        println!("rows:          {rows}");
    }
    if let Some(ms) = run.total_ms {
        println!("total_ms:      {ms}");
    }
    if let Some(reason) = &run.failure_reason {
        println!("failure:       {reason}");
    }
    Ok(())
}
