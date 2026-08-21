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
