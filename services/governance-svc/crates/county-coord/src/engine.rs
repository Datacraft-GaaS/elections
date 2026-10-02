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

use crate::models::*;
use chrono::Utc;

pub struct CoordEngine;

impl CoordEngine {
    /// Check if MoU is expiring within `days` days.
    pub fn is_expiring_soon(mou: &Mou, days: i64) -> bool {
        if let Ok(expiry) = chrono::DateTime::parse_from_rfc3339(&mou.expiry_date) {
            let remaining = expiry.signed_duration_since(Utc::now()).num_days();
            remaining > 0 && remaining <= days
        } else {
            false
        }
    }

    /// Determine if incident should be auto-escalated to ECC (Critical or High unresolved).
    pub fn should_escalate_to_ecc(incident: &CoordinationIncident) -> bool {
        matches!(
            incident.severity,
            IncidentSeverity::Critical | IncidentSeverity::High
        ) && incident.resolved_at.is_none()
    }

    /// Validate SAML ACS assertion (stub; real impl verifies XML signature chain).
    pub fn validate_saml_assertion(assertion_xml: &str) -> bool {
        !assertion_xml.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escalation_logic_critical() {
        let incident = CoordinationIncident {
            severity: IncidentSeverity::Critical,
            resolved_at: None,
            ..Default::default()
        };
        assert!(CoordEngine::should_escalate_to_ecc(&incident));
    }

    #[test]
    fn no_escalation_when_resolved() {
        let incident = CoordinationIncident {
            severity: IncidentSeverity::Critical,
            resolved_at: Some("2024-01-01T00:00:00Z".to_string()),
            ..Default::default()
        };
        assert!(!CoordEngine::should_escalate_to_ecc(&incident));
    }
}
