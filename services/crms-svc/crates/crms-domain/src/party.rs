//! Political party aggregate — M45 registration management.
//! PPA 2011, FR-M45-*, PPDT integration stub.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PartyStatus {
    Draft,
    Submitted,
    DocsReview,
    MembershipVerified,
    RegistrarReview,
    Approved,
    Conditional,
    Rejected,
    Gazetted,
    Suspended,
    Deregistered,
}

impl Default for PartyStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoliticalParty {
    pub id: Uuid,
    pub name_en: String,
    pub name_sw: String,
    pub abbreviation: String,
    pub ideology: Option<String>,
    pub status: PartyStatus,
    pub registration_number: Option<String>,
    pub gazette_notice_url: Option<String>,
    pub symbol_registry_id: Option<Uuid>,
    pub colour_hex: Option<String>,
    pub hq_address: String,
    pub verified_membership_count: u32,
    pub compliance_score: u8,
    pub coalition_id: Option<Uuid>,
    pub foreign_origin_flag: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl PoliticalParty {
    pub fn new(
        id: Uuid,
        name_en: String,
        name_sw: String,
        abbreviation: String,
        hq_address: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id,
            name_en,
            name_sw,
            abbreviation,
            ideology: None,
            status: PartyStatus::Draft,
            registration_number: None,
            gazette_notice_url: None,
            symbol_registry_id: None,
            colour_hex: None,
            hq_address,
            verified_membership_count: 0,
            compliance_score: 0,
            coalition_id: None,
            foreign_origin_flag: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn submit(&mut self) {
        self.status = PartyStatus::Submitted;
        self.updated_at = Utc::now();
    }

    pub fn approve(&mut self, registration_number: String, gazette_url: String) {
        self.status = PartyStatus::Approved;
        self.registration_number = Some(registration_number);
        self.gazette_notice_url = Some(gazette_url);
        self.updated_at = Utc::now();
    }

    pub fn suspend(&mut self) {
        self.status = PartyStatus::Suspended;
        self.updated_at = Utc::now();
    }

    pub fn deregister(&mut self) {
        self.status = PartyStatus::Deregistered;
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, PartyStatus::Approved | PartyStatus::Gazetted)
    }
}

/// Annual return filing (M45, FR-M45-annual-returns).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnualReturn {
    pub id: Uuid,
    pub party_id: Uuid,
    pub filing_year: i32,
    pub audited_accounts_url: String,
    pub updated_officials_roster: serde_json::Value,
    pub membership_count_update: u32,
    pub governance_self_declaration: String,
    pub foreign_origin_flag: bool,
    pub filed_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
}
