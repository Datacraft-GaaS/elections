use credential::http::{router, AppState};

#[tokio::test]
async fn router_smoke() {
    let state = AppState::new_shared();
    let app = router(state);
    drop(app);
}
