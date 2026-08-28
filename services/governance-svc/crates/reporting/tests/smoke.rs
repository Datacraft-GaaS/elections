#[tokio::test]
async fn router_smoke() {
    let state = eel_governance_reporting::AppState::new();
    let app = eel_governance_reporting::router::build_router(state);
    drop(app);
}
