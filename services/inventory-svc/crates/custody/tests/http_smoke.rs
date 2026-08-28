use custody::{router, state::AppState};

#[tokio::test]
async fn router_smoke() {
    let state = AppState::new();
    let app = router::router(state);
    drop(app);
}
