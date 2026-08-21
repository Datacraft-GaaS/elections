use serde::{Deserialize, Serialize};

/// Request priority class. Higher ordinal = higher priority.
/// Admin requests are NEVER shed before lower priority classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[repr(u8)]
pub enum Priority {
    /// Anonymous (unauthenticated) requests - shed first.
    Anonymous = 0,
    /// Accredited machine-to-machine callers.
    AccreditedMachine = 1,
    /// Authenticated human users.
    AuthenticatedUser = 2,
    /// Internal admin requests - shed last.
    Admin = 3,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Anonymous
    }
}

impl Priority {
    /// Compute priority from request headers.
    /// Looks for `X-Eel-Priority` header or falls back to auth presence.
    pub fn from_headers(headers: &http::HeaderMap) -> Self {
        if let Some(val) = headers.get("x-eel-priority") {
            if let Ok(s) = val.to_str() {
                match s {
                    "admin" => return Self::Admin,
                    "authenticated_user" => return Self::AuthenticatedUser,
                    "accredited_machine" => return Self::AccreditedMachine,
                    _ => {}
                }
            }
        }
        // If Authorization header present, at least AuthenticatedUser
        if headers.contains_key("authorization") {
            Self::AuthenticatedUser
        } else {
            Self::Anonymous
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Admin => "admin",
            Self::AuthenticatedUser => "authenticated_user",
            Self::AccreditedMachine => "accredited_machine",
            Self::Anonymous => "anonymous",
        }
    }
}
