use std::sync::Arc;

#[tokio::test]
async fn router_smoke() {
    let app = eel_governance_oag::router::build_router(Arc::new(
        eel_governance_oag::AppState::default(),
    ));
    drop(app);
}
