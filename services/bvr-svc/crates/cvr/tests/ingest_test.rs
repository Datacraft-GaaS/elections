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
    let records = vec![serde_json::json!({"id": "a"}), serde_json::json!({"id": "b"})];
    let req = BatchIngestRequest { source: EventSource::Nrb, records };
    let report = svc.ingest(req).await.unwrap();
    assert_eq!(report.record_count, 2);
    assert_eq!(report.matched, 2);
}
