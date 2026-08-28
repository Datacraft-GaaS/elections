#[tokio::test]
async fn router_smoke() {
    let state = eel_governance_legal_audit::AppState::new();
    let app = eel_governance_legal_audit::router::build_router(state);
    drop(app);
}
