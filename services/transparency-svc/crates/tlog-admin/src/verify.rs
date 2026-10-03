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

//! `tlog-admin verify-chain --sth-a <n> --sth-b <n>` — verify consistency between two STHs.

use anyhow::Result;

pub async fn run(sth_a: u64, sth_b: u64, api_url: &str) -> Result<()> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/api/v1/transparency/proof/consistency?from_size={}&to_size={}",
        api_url, sth_a, sth_b
    );
    let resp: serde_json::Value = client.get(&url).send().await?.json().await?;
    println!(
        "Consistency proof: {}",
        serde_json::to_string_pretty(&resp)?
    );
    Ok(())
}
