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

pub struct ObserverEngine;

impl ObserverEngine {
    /// Each mission gets an isolated workspace_id derived from its id.
    /// In production this provisions a SpiceDB namespace.
    pub fn provision_workspace(mission_id: &str) -> String {
        format!("ws-{mission_id}")
    }

    /// Verify mission is active before accepting a report.
    pub fn can_submit_report(mission: &ObserverMission) -> bool {
        mission.status == MissionStatus::Active
    }

    /// Validate that all participating missions exist (stub; real impl queries DB).
    pub fn validate_joint_statement_participants(ids: &[String]) -> bool {
        !ids.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workspace_provisioning() {
        let ws = ObserverEngine::provision_workspace("mission-123");
        assert!(ws.starts_with("ws-"));
    }

    #[test]
    fn inactive_mission_cannot_report() {
        let m = ObserverMission {
            status: MissionStatus::Registered,
            ..Default::default()
        };
        assert!(!ObserverEngine::can_submit_report(&m));
    }

    #[test]
    fn active_mission_can_report() {
        let m = ObserverMission {
            status: MissionStatus::Active,
            ..Default::default()
        };
        assert!(ObserverEngine::can_submit_report(&m));
    }
}
