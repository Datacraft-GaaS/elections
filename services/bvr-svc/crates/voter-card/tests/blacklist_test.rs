use chrono::Utc;
use uuid::Uuid;
use voter_card::blacklist::BlacklistService;
use voter_card::models::{BlacklistEntry, BlacklistSource};

#[tokio::test]
async fn add_entry_succeeds() {
    let svc = BlacklistService::new();
    let entry = BlacklistEntry {
        id: Uuid::now_v7(),
        card_number: "KE-2024-000001".to_string(),
        reason: "reported lost".to_string(),
        blacklisted_at: Utc::now(),
        blacklist_source: BlacklistSource::LostReport,
    };
    assert!(svc.add_entry(entry).await.is_ok());
}
