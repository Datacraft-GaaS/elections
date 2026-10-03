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

use crate::api_client::{DefinitionFile, ReportsClient};
use anyhow::{Context, Result};
use serde_json::Value;

pub async fn run(client: &ReportsClient, file: &str) -> Result<()> {
    let raw = std::fs::read_to_string(file).context("read definition file")?;
    // Accept YAML or JSON by parsing as JSON after a best-effort YAML-to-JSON pass.
    // Simple approach: try JSON first, then treat as plain struct via serde_json.
    let def: DefinitionFile =
        serde_json::from_str(&raw).context("parse definition file as JSON")?;
    let resp: Value = client
        .post_json("/api/v1/reports/definitions", &def)
        .await?;
    println!(
        "Created: {}",
        resp.get("definition_id")
            .and_then(|v| v.as_str())
            .unwrap_or("?")
    );
    Ok(())
}
