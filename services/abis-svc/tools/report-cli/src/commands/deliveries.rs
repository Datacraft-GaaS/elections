use crate::api_client::{Delivery, ReportsClient};
use anyhow::Result;
use serde_json::json;

pub async fn run(client: &ReportsClient, run_id: &str, retry_failed: bool) -> Result<()> {
    let deliveries: Vec<Delivery> =
        client.get_json(&format!("/api/v1/reports/runs/{run_id}/deliveries")).await?;

    println!("{:<40} {:<12} {:>5}  {}", "DELIVERY_ID", "STATUS", "ATTEMPTS", "LAST_ERROR");
    println!("{}", "-".repeat(100));
    for d in &deliveries {
        let err = d.last_error.as_deref().unwrap_or("-");
        println!("{:<40} {:<12} {:>5}  {}", d.delivery_id, d.status, d.attempt_count, err);
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
