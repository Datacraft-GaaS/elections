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

//! `tlog-admin bulk-ingest --topic <t> --from-offset <n>` — replay a Kafka topic into the log.

use anyhow::Result;

pub async fn run(topic: &str, from_offset: i64, _api_url: &str) -> Result<()> {
    tracing::info!(topic = %topic, from_offset = from_offset, "bulk-ingest started (stub)");
    println!(
        "bulk-ingest from offset {} on topic {} (stub)",
        from_offset, topic
    );
    Ok(())
}
