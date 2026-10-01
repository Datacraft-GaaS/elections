use channel_gateway::http::router;
use channel_gateway::store::AppState;
use std::sync::Arc;

#[tokio::test]
async fn router_smoke() {
    let app = router(Arc::new(AppState::new()));
    drop(app);
}
