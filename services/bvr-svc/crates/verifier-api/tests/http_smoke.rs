use verifier_api::{http::router, store::VerifierStore};

#[tokio::test]
async fn router_smoke() {
    let app = router(VerifierStore::new().shared());
    drop(app);
}
