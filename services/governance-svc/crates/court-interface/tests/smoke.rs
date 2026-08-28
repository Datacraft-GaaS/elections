#[tokio::test]
async fn router_smoke() {
    let state = eel_governance_court::AppState::new();
    let app = eel_governance_court::router::build_router(state);
    drop(app);
}
