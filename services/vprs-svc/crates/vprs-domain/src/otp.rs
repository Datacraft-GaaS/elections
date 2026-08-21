use chrono::{DateTime, Utc};

pub const MAX_REQUESTS: u32 = 3;
pub const LOCKOUT_MINUTES: i64 = 15;

#[derive(Debug, Clone)]
pub struct OtpRequest {
    pub session_id: String,
    pub attempt_count: u32,
    pub locked_until: Option<DateTime<Utc>>,
}

impl OtpRequest {
    pub fn is_locked(&self) -> bool {
        self.locked_until.map(|t| t > Utc::now()).unwrap_or(false)
    }

    pub fn is_rate_limited(&self) -> bool {
        self.attempt_count >= MAX_REQUESTS
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OtpVerifyResult {
    Ok,
    RateLimited,
    Invalid,
    Locked,
}
