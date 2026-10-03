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
    let resp: Value = client
        .post_json("/api/v1/reports/subscriptions", &req)
        .await?;
    println!(
        "Subscribed: subscription_id={}",
        resp.get("subscription_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
    );
    Ok(())
}
