#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_oag::router::build_router();
    drop(app);
}
