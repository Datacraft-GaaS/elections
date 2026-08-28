use verifier_api::http::router;

#[tokio::test]
async fn router_smoke() {
    let app = router();
    drop(app);
}
