use eel_loadshedding_rs::{
    priority::Priority,
    shedder::{CoDelConfig, CoDelShedder},
    layer::LoadSheddingLayer,
};
use std::time::{Duration, Instant};
use http::{Request, Response, StatusCode};
use http_body_util::Full;
use bytes::Bytes;
use tower::{Service, ServiceExt};

// Priority ordering
#[tokio::test]
async fn priority_ordering() {
    assert!(Priority::Admin > Priority::AuthenticatedUser);
    assert!(Priority::AuthenticatedUser > Priority::AccreditedMachine);
    assert!(Priority::AccreditedMachine > Priority::Anonymous);
}

#[tokio::test]
async fn priority_from_headers_anonymous_default() {
    let headers = http::HeaderMap::new();
    assert_eq!(Priority::from_headers(&headers), Priority::Anonymous);
}

#[tokio::test]
async fn priority_from_headers_admin() {
    let mut headers = http::HeaderMap::new();
    headers.insert("x-eel-priority", "admin".parse().unwrap());
    assert_eq!(Priority::from_headers(&headers), Priority::Admin);
}

#[tokio::test]
async fn priority_from_authorization_header() {
    let mut headers = http::HeaderMap::new();
    headers.insert("authorization", "Bearer token123".parse().unwrap());
    assert_eq!(Priority::from_headers(&headers), Priority::AuthenticatedUser);
}

#[tokio::test]
async fn shedder_admits_admin_under_capacity() {
    let config = CoDelConfig {
        max_inflight: 10,
        shed_below: Priority::Anonymous,
        ..Default::default()
    };
    let shedder = CoDelShedder::new(config);
    let result = shedder.admit(Priority::Admin, Instant::now()).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn shedder_sheds_anonymous_at_capacity() {
    let config = CoDelConfig {
        max_inflight: 1,
        shed_below: Priority::Anonymous,
        target_latency: Duration::from_millis(1),
        interval: Duration::from_millis(1),
        ..Default::default()
    };
    let shedder = CoDelShedder::new(config);

    // Acquire the only permit
    let _permit = shedder.admit(Priority::Admin, Instant::now()).await.ok();

    // Now an anonymous request should be shed
    let start = Instant::now() - Duration::from_secs(1); // simulate old request
    let result = shedder.admit(Priority::Anonymous, start).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn metrics_shed_count_increments() {
    let config = CoDelConfig {
        max_inflight: 1,
        shed_below: Priority::Anonymous,
        target_latency: Duration::from_millis(1),
        interval: Duration::from_millis(1),
        ..Default::default()
    };
    let shedder = CoDelShedder::new(config);
    let _permit = shedder.admit(Priority::Admin, Instant::now()).await.ok();

    let old_start = Instant::now() - Duration::from_secs(1);
    let _ = shedder.admit(Priority::Anonymous, old_start).await;

    let snap = shedder.metrics.snapshot();
    assert!(snap.shed_anonymous > 0 || snap.shed_admin > 0, "expected at least one shed");
}

// Test tower layer integration
#[tokio::test]
async fn layer_passes_normal_request() {
    use tower::service_fn;

    let config = CoDelConfig {
        max_inflight: 100,
        ..Default::default()
    };
    let layer = LoadSheddingLayer::new(CoDelShedder::new(config));

    let inner = service_fn(|_req: Request<Full<Bytes>>| async {
        Ok::<Response<Full<Bytes>>, Box<dyn std::error::Error + Send + Sync>>(
            Response::builder()
                .status(200)
                .body(Full::new(Bytes::from("ok")))
                .unwrap()
        )
    });

    let mut svc = tower::ServiceBuilder::new()
        .layer(layer)
        .service(inner);

    let req = Request::builder()
        .body(Full::new(Bytes::new()))
        .unwrap();

    let resp = svc.ready().await.unwrap().call(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}
