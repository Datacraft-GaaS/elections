use serde::{Deserialize, Serialize};

/// Extracted from a validated JWT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject {
    /// Keycloak `sub` claim (UUID string).
    pub sub: String,
    /// Keycloak realm name.
    pub realm: String,
    /// Keycloak session ID (`sid` claim).
    pub session_id: Option<String>,
    /// Roles from `realm_access.roles`.
    pub roles: Vec<String>,
    /// Purpose scopes parsed from `scope` claim.
    pub purpose_scopes: Vec<String>,
}
