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

use crate::api_client::{Delivery, ReportsClient};
use anyhow::Result;
use serde_json::json;

pub async fn run(client: &ReportsClient, run_id: &str, retry_failed: bool) -> Result<()> {
    let deliveries: Vec<Delivery> = client
        .get_json(&format!("/api/v1/reports/runs/{run_id}/deliveries"))
        .await?;

    println!(
        "{:<40} {:<12} {:>5}  {}",
        "DELIVERY_ID", "STATUS", "ATTEMPTS", "LAST_ERROR"
    );
    println!("{}", "-".repeat(100));
    for d in &deliveries {
        let err = d.last_error.as_deref().unwrap_or("-");
        println!(
            "{:<40} {:<12} {:>5}  {}",
            d.delivery_id, d.status, d.attempt_count, err
        );
    }
    println!("\n{} delivery record(s)", deliveries.len());

    if retry_failed {
        let failed: Vec<&str> = deliveries
            .iter()
            .filter(|d| d.status == "failed")
            .map(|d| d.delivery_id.as_str())
            .collect();

        if failed.is_empty() {
            println!("No failed deliveries to retry.");
        } else {
            let body = json!({ "delivery_ids": failed });
            let resp: serde_json::Value = client
                .post_json("/api/v1/reports/deliveries/retry", &body)
                .await?;
            println!("Retried {} delivery(s): {:?}", failed.len(), resp);
        }
    }
    Ok(())
}
