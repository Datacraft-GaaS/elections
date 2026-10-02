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

use cvr::ingest::IngestService;
use cvr::models::{BatchIngestRequest, EventSource};

#[tokio::test]
async fn ingest_returns_report() {
    let svc = IngestService::new();
    let req = BatchIngestRequest { source: EventSource::Crs, records: vec![] };
    let report = svc.ingest(req).await.unwrap();
    assert_eq!(report.record_count, 0);
}

#[tokio::test]
async fn ingest_counts_records() {
    let svc = IngestService::new();
    // Valid NRB records (no biometric_ref required for NRB).
    let records = vec![
        serde_json::json!({"national_id": "12345678", "name": "JANE DOE"}),
        serde_json::json!({"national_id": "87654321", "name": "JOHN SMITH"}),
    ];
    let req = BatchIngestRequest { source: EventSource::Nrb, records };
    let report = svc.ingest(req).await.unwrap();
    assert_eq!(report.record_count, 2);
    // No voter register is wired, so no record is falsely matched.
    assert_eq!(report.matched, 0);
    assert_eq!(report.unmatched, 2);
    assert_eq!(report.errors, 0);
}
