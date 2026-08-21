use chrono::{DateTime, Utc};
use eel_pii_rs::{PIIRestricted, PIISensitive};
use serde::{Deserialize, Serialize};

use crate::ids::uuid7str;

/// Biometric template reference stored alongside a voter record.
/// The actual ciphertext lives in MinIO; this is the locator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricTemplateRef {
    pub template_id: String,
    /// "fingerprint" | "iris" | "face"
    pub modality: String,
    /// Encryption key id used for the stored ciphertext.
    pub key_id: String,
}

/// Registration lifecycle for a voter record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoterStatus {
    Pending,
    Active,
    Suspended,
    Deregistered,
}

/// BVR voter aggregate. County-partitioned per ADR-0013.
///
/// PII tiers per ADR-0016:
/// - Tier A (PIIRestricted): national_id, full_name, dob, phone
/// - Tier B (PIISensitive): polling_station_id
pub struct Voter {
    pub voter_id: String,
    pub version: i64,
    /// County partition key (Tier B operational).
    pub county_code: String,
    /// Tier A: national ID number.
    pub national_id: PIIRestricted<String>,
    /// Tier A: voter full name.
    pub full_name: PIIRestricted<String>,
    /// Tier A: date of birth (ISO date string).
    pub dob: PIIRestricted<String>,
    /// Tier A: contact phone (optional).
    pub phone: PIIRestricted<Option<String>>,
    /// Tier B: assigned polling station.
    pub polling_station_id: PIISensitive<String>,
    pub status: VoterStatus,
    pub biometric_template_refs: Vec<BiometricTemplateRef>,
    pub election_scope: String,
    /// Canary row marker per EC-SEC-16.
    pub is_canary: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl std::fmt::Debug for Voter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Voter")
            .field("voter_id", &self.voter_id)
            .field("version", &self.version)
            .field("county_code", &self.county_code)
            .field("national_id", &self.national_id)
            .field("full_name", &self.full_name)
            .field("dob", &self.dob)
            .field("phone", &self.phone)
            .field("polling_station_id", &self.polling_station_id)
            .field("status", &self.status)
            .field("election_scope", &self.election_scope)
            .field("is_canary", &self.is_canary)
            .finish()
    }
}

impl Voter {
    pub fn new(
        national_id: String,
        full_name: String,
        dob: String,
        county_code: String,
        polling_station_id: String,
        election_scope: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            voter_id: uuid7str(),
            version: 0,
            county_code,
            national_id: PIIRestricted::new(national_id),
            full_name: PIIRestricted::new(full_name),
            dob: PIIRestricted::new(dob),
            phone: PIIRestricted::new(None),
            polling_station_id: PIISensitive::new(polling_station_id),
            status: VoterStatus::Pending,
            biometric_template_refs: vec![],
            election_scope,
            is_canary: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_status_change(&mut self, new_status: VoterStatus) {
        self.status = new_status;
        self.version += 1;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eel_pii_rs::{NoopAuditSink, Purpose};

    #[test]
    fn new_voter_has_pending_status() {
        let v = Voter::new(
            "KE12345678".into(),
            "Jane Voter".into(),
            "1990-06-15".into(),
            "047".into(),
            "047-001-001".into(),
            "2027-general".into(),
        );
        assert_eq!(v.status, VoterStatus::Pending);
        assert!(!v.is_canary);
        assert_eq!(v.version, 0);
    }

    #[test]
    fn status_change_increments_version() {
        let mut v = Voter::new(
            "KE12345678".into(),
            "Jane Voter".into(),
            "1990-06-15".into(),
            "047".into(),
            "047-001-001".into(),
            "2027-general".into(),
        );
        v.apply_status_change(VoterStatus::Active);
        assert_eq!(v.status, VoterStatus::Active);
        assert_eq!(v.version, 1);
    }

    #[test]
    fn national_id_is_restricted() {
        let v = Voter::new(
            "KE99887766".into(),
            "Test Name".into(),
            "2000-01-01".into(),
            "001".into(),
            "001-001-001".into(),
            "2027-general".into(),
        );
        let dbg = format!("{v:?}");
        assert!(!dbg.contains("KE99887766"));
        let revealed = v
            .national_id
            .reveal(Purpose::Identification, &NoopAuditSink);
        assert_eq!(revealed, "KE99887766");
    }
}
