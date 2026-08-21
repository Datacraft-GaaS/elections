use eel_resilience_rs::{
    circuit_breaker::{CircuitBreaker, CircuitState},
    bulkhead::Bulkhead,
    degradation::{DegradationFlag, DegradationMode},
    idempotency::IdempotencyStore,
    retry::{RetryConfig, with_retry},
    timeout::with_timeout,
    error::ResilienceError,
};
use std::time::Duration;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[tokio::test]
async fn circuit_breaker_opens_after_threshold() {
    let cb = CircuitBreaker::new("test-dep", 3, Duration::from_secs(60), Duration::from_secs(5));
    // Record failures up to threshold
    cb.record_failure().await;
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Closed);
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    // Next allow should fail
    assert!(cb.allow().await.is_err());
}

#[tokio::test]
async fn circuit_breaker_half_open_on_cooldown() {
    let cb = CircuitBreaker::new("dep2", 1, Duration::from_secs(60), Duration::from_millis(10));
    cb.record_failure().await;
    assert_eq!(cb.state().await, CircuitState::Open);
    tokio::time::sleep(Duration::from_millis(20)).await;
    // After cooldown allow() should succeed and state transitions to HalfOpen
    assert!(cb.allow().await.is_ok());
    assert_eq!(cb.state().await, CircuitState::HalfOpen);
}

#[tokio::test]
async fn bulkhead_rejects_over_capacity() {
    let bh = Bulkhead::new("upstream", 2);
    let _p1 = bh.acquire().await.expect("first permit");
    let _p2 = bh.acquire().await.expect("second permit");
    let p3 = bh.acquire().await;
    assert!(p3.is_err());
}

#[tokio::test]
async fn bulkhead_releases_on_drop() {
    let bh = Bulkhead::new("upstream", 1);
    {
        let _p = bh.acquire().await.unwrap();
        assert!(bh.acquire().await.is_err());
    } // _p dropped here
    // Should be available again
    assert!(bh.acquire().await.is_ok());
}

#[tokio::test]
async fn degradation_flag_transitions() {
    let flag = DegradationFlag::new();
    assert_eq!(flag.get().await, DegradationMode::Normal);
    assert!(flag.is_write_allowed().await);

    flag.set(DegradationMode::ReadOnlyLockdown).await;
    assert_eq!(flag.get().await, DegradationMode::ReadOnlyLockdown);
    assert!(!flag.is_write_allowed().await);

    flag.set(DegradationMode::Degraded).await;
    assert!(flag.is_write_allowed().await);
}

#[tokio::test]
async fn idempotency_deduplicates() {
    let store = IdempotencyStore::new(86400);
    let body = b"request body";

    // First call: new key
    let result = store.check("user1", "key-abc", body).await.unwrap();
    assert!(result.is_none());

    // Store a response
    store.store_response("user1", "key-abc", b"response".to_vec()).await;

    // Second call with same body: returns cached response
    let result = store.check("user1", "key-abc", body).await.unwrap();
    assert_eq!(result, Some(b"response".to_vec()));
}

#[tokio::test]
async fn idempotency_conflict_on_different_body() {
    let store = IdempotencyStore::new(86400);
    store.check("user1", "key-xyz", b"body-1").await.unwrap();
    let result = store.check("user1", "key-xyz", b"body-2").await;
    assert!(matches!(result, Err(ResilienceError::IdempotencyConflict)));
}

#[tokio::test]
async fn retry_exhausts_and_returns_error() {
    let config = RetryConfig {
        max_attempts: 3,
        base_delay: Duration::from_millis(1),
        max_delay: Duration::from_millis(10),
        ..Default::default()
    };
    let attempts = Arc::new(AtomicU32::new(0));
    let a = attempts.clone();
    let result = with_retry(&config, |_: &String| true, move || {
        let a = a.clone();
        async move {
            a.fetch_add(1, Ordering::Relaxed);
            Err::<(), String>("always fails".to_string())
        }
    }).await;
    assert!(matches!(result, Err(ResilienceError::RetriesExhausted { attempts: 3 })));
    assert_eq!(attempts.load(Ordering::Relaxed), 3);
}

#[tokio::test]
async fn timeout_triggers() {
    let result = with_timeout(Duration::from_millis(10), async {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }).await;
    assert!(matches!(result, Err(ResilienceError::Timeout)));
}

#[tokio::test]
async fn timeout_passes_fast_future() {
    let result = with_timeout(Duration::from_secs(5), async { 42u32 }).await;
    assert_eq!(result.unwrap(), 42);
}
