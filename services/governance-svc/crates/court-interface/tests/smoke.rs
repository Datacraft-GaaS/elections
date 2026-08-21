#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_court::router::build_router();
    drop(app);
}
