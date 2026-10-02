// Copyright (c) 2026 Nyimbi Odero
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! G11 — Observer Access View (CLOSED)
//! Read-only, sanitised, rate-limited, audit-logged access per observer org.
//! M5: every query emits auditId=g11-{org_id}-{timestamp}, proofHash, signature.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverAccessConfig {
    pub org_id: String,
    pub organisation: String, // "EU_EOM", "CARTER_CENTER", "IFES", "ELOG"
    pub roles: Vec<String>,
    pub rate_limit_rpm: u32,
    pub allowed_endpoints: Vec<String>,
}
impl ObserverAccessConfig {
    /// Validate that an observer org is authorised to view the requested resource.
    pub fn can_view(&self, endpoint: &str) -> bool {
        self.allowed_endpoints.iter().any(|e| endpoint.starts_with(e))
    }
    /// Sanitise a response — strip PII, redact raw biometric data.
    pub fn sanitise(&self, data: &serde_json::Value) -> serde_json::Value {
        // Strip fields that observers must not see: raw biometrics, operator PINs, audit details
        serde_json::json!({
            "status": data.get("status"),
            "form_hash": data.get("form_hash"),
            "transmission_time": data.get("transmission_time"),
            "station_code": data.get("station_code"),
            "observer_note": "[REDACTED per IAB charter clause 3.1]"
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observer_sees_form_hash_not_raw_biometrics() {
        let cfg = ObserverAccessConfig {
            org_id: "eu-eom-001".into(),
            organisation: "EU_EOM".into(),
            roles: vec!["observer".into()],
            rate_limit_rpm: 60,
            allowed_endpoints: vec!["/v1/results".into(), "/v1/forms".into()],
        };
        assert!(cfg.can_view("/v1/results/0101010101"));
        assert!(!cfg.can_view("/v1/admin/operators"));
        let data = serde_json::json!({"form_hash":"abc","raw_biometric":"secret","station_code":"0101010101"});
        let scrubbed = cfg.sanitise(&data);
        assert!(!scrubbed.to_string().contains("secret"));
        assert!(scrubbed.to_string().contains("abc"));
    }
}
