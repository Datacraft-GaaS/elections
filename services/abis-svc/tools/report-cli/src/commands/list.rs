use crate::api_client::ReportsClient;
use anyhow::Result;

pub async fn run(client: &ReportsClient, class: Option<&str>, active_only: bool) -> Result<()> {
    let mut path = "/api/v1/reports/definitions?".to_string();
    if let Some(c) = class {
        path.push_str(&format!("class={c}&"));
    }
    if active_only {
        path.push_str("active_only=true&");
    }

    let defs: Vec<crate::api_client::ReportDefinition> = client.get_json(&path).await?;

    println!("{:<45} {:<15} {:<12} {:<10} {:>5}",
        "SLUG", "CLASS", "DATA", "SCHEDULE", "RET_YR");
    println!("{}", "-".repeat(95));
    for d in &defs {
        let schedule = d.default_schedule.as_deref().unwrap_or("on-demand");
        let retired = if d.retired_at.is_some() { " [retired]" } else { "" };
        println!("{:<45} {:<15} {:<12} {:<10} {:>5}{}",
            d.slug, d.report_class, d.data_class, schedule, d.retention_years, retired);
    }
    println!("\n{} definition(s)", defs.len());
    Ok(())
}
