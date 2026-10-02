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

use crate::api_client::ReportsClient;
use crate::commands::download::sha256_hex;
use anyhow::{Context, Result};

pub async fn run(client: &ReportsClient, file: &str, artifact_id: &str) -> Result<()> {
    let bytes = std::fs::read(file).context("read local file")?;
    let actual_hex = sha256_hex(&bytes);

    let meta = client.get_artifact_meta(artifact_id).await?;
    let expected_hex = &meta.sha256_hex;

    println!("artifact_id:  {artifact_id}");
    println!("local_file:   {file}");
    println!("expected_sha: {expected_hex}");
    println!("actual_sha:   {actual_hex}");

    if actual_hex == *expected_hex {
        println!("Result: PASS - file matches catalog hash");
        Ok(())
    } else {
        eprintln!("Result: FAIL - SHA-256 mismatch (file may be tampered or corrupt)");
        std::process::exit(2);
    }
}
