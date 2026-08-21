use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::{audit::{AuditEvent, AuditSink}, error::AuthError, subject::Subject};
use serde_json::json;

/// Consistency mode for SpiceDB checks.
#[derive(Debug, Clone)]
pub enum Consistency {
    /// Minimize latency - may serve stale data.
    MinimizeLatency,
    /// Fully consistent read.
    FullConsistency,
    /// At least as fresh as the given zookie.
    AtLeastAsFresh(String),
}

/// Authorization decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Allow,
    Deny,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct CacheKey {
    subject: String,
    permission: String,
    resource: String,
    zookie: Option<String>,
}

/// Thin SpiceDB client using HTTP JSON API with in-memory cache and fail-closed semantics.
pub struct SpiceDbClient<S: AuditSink> {
    endpoint: String,
    cache: Arc<Mutex<HashMap<CacheKey, Decision>>>,
    audit_sink: Arc<S>,
}

impl<S: AuditSink> SpiceDbClient<S> {
    pub fn new(endpoint: impl Into<String>, audit_sink: S) -> Self {
        Self {
            endpoint: endpoint.into(),
            cache: Arc::new(Mutex::new(HashMap::new())),
            audit_sink: Arc::new(audit_sink),
        }
    }

    /// Check whether `subject` has `permission` on `resource`.
    ///
    /// Caches the result keyed by (subject, permission, resource, zookie).
    /// Fails closed: if SpiceDB is unavailable, returns `Deny`.
    pub async fn check(
        &self,
        subject: &Subject,
        permission: &str,
        resource: &str,
        consistency: Consistency,
        context: serde_json::Value,
    ) -> Result<Decision, AuthError> {
        let zookie = match &consistency {
            Consistency::AtLeastAsFresh(z) => Some(z.clone()),
            _ => None,
        };

        let key = CacheKey {
            subject: subject.sub.clone(),
            permission: permission.to_owned(),
            resource: resource.to_owned(),
            zookie: zookie.clone(),
        };

        {
            let cache = self.cache.lock().await;
            if let Some(&decision) = cache.get(&key) {
                let ev = AuditEvent::new(
                    &subject.sub,
                    permission,
                    resource,
                    if decision == Decision::Allow { "allow" } else { "deny" },
                    json!({"cache": "hit", "context": context}),
                );
                let _ = self.audit_sink.emit(ev).await;
                return Ok(decision);
            }
        }

        let decision = self.do_check(subject, permission, resource, &consistency, &context).await
            .unwrap_or(Decision::Deny);

        {
            let mut cache = self.cache.lock().await;
            cache.insert(key, decision);
        }

        let ev = AuditEvent::new(
            &subject.sub,
            permission,
            resource,
            if decision == Decision::Allow { "allow" } else { "deny" },
            json!({"cache": "miss", "context": context}),
        );
        let _ = self.audit_sink.emit(ev).await;
        Ok(decision)
    }

    async fn do_check(
        &self,
        subject: &Subject,
        permission: &str,
        resource: &str,
        consistency: &Consistency,
        _context: &serde_json::Value,
    ) -> Result<Decision, AuthError> {
        let consistency_val = match consistency {
            Consistency::MinimizeLatency => json!({"minimizeLatency": {}}),
            Consistency::FullConsistency => json!({"fullyConsistent": {}}),
            Consistency::AtLeastAsFresh(z) => json!({"atLeastAsFresh": {"token": z}}),
        };

        let (res_type, res_id) = resource.split_once(':')
            .unwrap_or(("resource", resource));
        let (sub_type, sub_id) = subject.sub.split_once(':')
            .unwrap_or(("user", subject.sub.as_str()));

        let body = json!({
            "consistency": consistency_val,
            "resource": {"objectType": res_type, "objectId": res_id},
            "permission": permission,
            "subject": {"object": {"objectType": sub_type, "objectId": sub_id}},
        });

        let url = format!("{}/v1/permissions/check", self.endpoint.trim_end_matches('/'));
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| AuthError::Transport(e.to_string()))?;

        let resp = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AuthError::SpiceDb(e.to_string()))?;

        if !resp.status().is_success() {
            return Err(AuthError::SpiceDb(format!("HTTP {}", resp.status())));
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| AuthError::SpiceDb(e.to_string()))?;

        let permissionship = json["permissionship"].as_str().unwrap_or("PERMISSIONSHIP_NO_PERMISSION");
        Ok(if permissionship.contains("HAS_PERMISSION") {
            Decision::Allow
        } else {
            Decision::Deny
        })
    }

    /// Invalidate all cached decisions.
    pub async fn invalidate_cache(&self) {
        self.cache.lock().await.clear();
    }
}
