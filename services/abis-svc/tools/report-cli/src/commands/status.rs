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
