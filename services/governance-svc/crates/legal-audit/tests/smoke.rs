#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_legal_audit::router::build_router();
    drop(app);
}
