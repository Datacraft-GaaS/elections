#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_rla::router::build_router();
    drop(app);
}
