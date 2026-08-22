//! Coalition registration — M45.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CoalitionStatus {
    Forming,
    Active,
    Dissolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoalitionRegistration {
    pub id: Uuid,
    pub name: String,
    pub member_party_ids: Vec<Uuid>,
    pub confirmed_party_ids: Vec<Uuid>,
    pub agreement_doc_url: Option<String>,
    pub seat_allocation_formula: Option<String>,
    pub status: CoalitionStatus,
    pub dissolved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl CoalitionRegistration {
    pub fn new(id: Uuid, name: String, member_party_ids: Vec<Uuid>) -> Self {
        Self {
            id,
            name,
            member_party_ids,
            confirmed_party_ids: Vec::new(),
            agreement_doc_url: None,
            seat_allocation_formula: None,
            status: CoalitionStatus::Forming,
            dissolved_at: None,
            created_at: Utc::now(),
        }
    }

    pub fn confirm_member(&mut self, party_id: Uuid) {
        if !self.confirmed_party_ids.contains(&party_id) {
            self.confirmed_party_ids.push(party_id);
        }
        if self.confirmed_party_ids.len() == self.member_party_ids.len() {
            self.status = CoalitionStatus::Active;
        }
    }

    pub fn dissolve(&mut self) {
        self.status = CoalitionStatus::Dissolved;
        self.dissolved_at = Some(Utc::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_trace_rs::trace;

    #[trace("FR-M45-015")]
    #[test]
    fn coalition_activates_only_after_all_members_confirm() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut c = CoalitionRegistration::new(Uuid::new_v4(), "Grand Alliance".into(), vec![a, b]);
        assert_eq!(c.status, CoalitionStatus::Forming);
        c.confirm_member(a);
        assert_eq!(c.status, CoalitionStatus::Forming);
        c.confirm_member(b);
        assert_eq!(c.status, CoalitionStatus::Active);
    }

    #[trace("FR-M45-015")]
    #[test]
    fn duplicate_confirmation_does_not_double_count() {
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let mut c = CoalitionRegistration::new(Uuid::new_v4(), "Alliance".into(), vec![a, b]);
        c.confirm_member(a);
        c.confirm_member(a);
        assert_eq!(c.confirmed_party_ids.len(), 1);
        assert_eq!(c.status, CoalitionStatus::Forming);
    }
}
