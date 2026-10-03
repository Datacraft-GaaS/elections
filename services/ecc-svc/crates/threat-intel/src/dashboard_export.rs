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

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::correlator::CorrelationMatch;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrafanaPanelData {
    pub title: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub targets: Vec<Value>,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Value>>,
}

pub fn build_grafana_panel(matches: &[CorrelationMatch]) -> GrafanaPanelData {
    let columns = vec![
        "flag_id".to_string(),
        "indicator_id".to_string(),
        "indicator_name".to_string(),
        "pattern".to_string(),
        "confidence".to_string(),
        "matched_at".to_string(),
    ];

    let rows = matches
        .iter()
        .map(|m| {
            vec![
                Value::String(m.flag_id.clone()),
                Value::String(m.indicator_id.clone()),
                m.indicator_name
                    .as_ref()
                    .map(|n| Value::String(n.clone()))
                    .unwrap_or(Value::Null),
                m.pattern
                    .as_ref()
                    .map(|p| Value::String(p.clone()))
                    .unwrap_or(Value::Null),
                serde_json::Number::from_f64(m.confidence)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                Value::String(m.matched_at.clone()),
            ]
        })
        .collect();

    GrafanaPanelData {
        title: "ECC Threat Correlation Matches".to_string(),
        type_: "table".to_string(),
        targets: vec![],
        columns,
        rows,
    }
}

/// Formats a PostgreSQL NOTIFY payload for the ECC LiveView channel.
/// Does not execute the query; callers integrate with their own PgPool.
pub fn format_notify_payload(
    matches: &[CorrelationMatch],
    channel: &str,
) -> Result<String, serde_json::Error> {
    let payload = serde_json::to_string(matches)?;
    Ok(format!(
        "NOTIFY \"{}\", '{}'",
        channel,
        payload.replace('\'', "''")
    ))
}

pub async fn publish_to_ecc_channel(
    matches: &[CorrelationMatch],
    channel: &str,
) -> Result<(), anyhow::Error> {
    let _cmd = format_notify_payload(matches, channel)?;
    // Actual DB dispatch is left to the caller via their PgPool.
    Ok(())
}
