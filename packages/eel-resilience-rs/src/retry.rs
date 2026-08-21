use std::time::Duration;
use tokio::time::sleep;
use crate::error::ResilienceError;

/// Jitter strategy for backoff.
#[derive(Debug, Clone, Copy)]
pub enum JitterKind {
    /// Full jitter: random in [0, base * 2^attempt].
    Full,
    /// No jitter.
    None,
}

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter: JitterKind,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            jitter: JitterKind::Full,
        }
    }
}

/// Execute `f` with jittered exponential backoff.
///
/// `should_retry` determines if the error warrants a retry (idempotent ops only).
pub async fn with_retry<F, Fut, T, E, R>(
    config: &RetryConfig,
    should_retry: R,
    mut f: F,
) -> Result<T, ResilienceError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
    R: Fn(&E) -> bool,
{
    let mut attempt = 0u32;
    loop {
        match f().await {
            Ok(v) => return Ok(v),
            Err(e) if attempt + 1 < config.max_attempts && should_retry(&e) => {
                attempt += 1;
                let base = config.base_delay.as_millis() as u64 * (1u64 << attempt.min(10));
                let capped = base.min(config.max_delay.as_millis() as u64);
                let delay = match config.jitter {
                    JitterKind::Full => {
                        use std::time::SystemTime;
                        let nanos = SystemTime::now()
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap_or_default()
                            .subsec_nanos();
                        let jitter = (nanos as u64) % capped.max(1);
                        Duration::from_millis(jitter)
                    }
                    JitterKind::None => Duration::from_millis(capped),
                };
                sleep(delay).await;
            }
            Err(_) => return Err(ResilienceError::RetriesExhausted { attempts: attempt + 1 }),
        }
    }
}
