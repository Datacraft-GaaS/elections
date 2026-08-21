use crate::api_client::{ReportsClient, SubscribeRequest};
use anyhow::Result;
use serde_json::Value;

pub async fn run(
    client: &ReportsClient,
    subscriber: &str,
    slug: &str,
    format: &str,
    channel: &str,
    endpoint: &str,
) -> Result<()> {
    let req = SubscribeRequest {
        subscriber_name: subscriber.to_string(),
        slug: slug.to_string(),
        format: format.to_string(),
        channel: channel.to_string(),
        endpoint: endpoint.to_string(),
    };
    let resp: Value = client.post_json("/api/v1/reports/subscriptions", &req).await?;
    println!(
        "Subscribed: subscription_id={}",
        resp.get("subscription_id").and_then(|v| v.as_str()).unwrap_or("?")
    );
    Ok(())
}
