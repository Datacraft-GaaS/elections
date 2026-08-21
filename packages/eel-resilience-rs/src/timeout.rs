use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;
use crate::error::ResilienceError;

/// Wrap a future with a timeout, mapping to `ResilienceError::Timeout`.
pub async fn with_timeout<F, T>(duration: Duration, f: F) -> Result<T, ResilienceError>
where
    F: Future<Output = T>,
{
    timeout(duration, f).await.map_err(|_| ResilienceError::Timeout)
}
