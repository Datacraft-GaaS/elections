use serde::{Deserialize, Serialize};

/// Maximum concurrent by-elections supported (FR-M50-008)
pub const MAX_CONCURRENT_BY_ELECTIONS: usize = 10;

/// Short-notice threshold in days (FR-M50-006, Elections Act s.24)
pub const SHORT_NOTICE_DAYS: i64 = 21;

/// Cost report T+14 advisory deadline in days (FR-M50-039)
pub const COST_REPORT_ADVISORY_DAYS: i64 = 14;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ByElectionStatus {
    Draft,
    Gazetted,
    NominationsOpen,
    Campaign,
    PollsOpen,
    Tallying,
    Declared,
    Certified,
    PetitionWindow,
    Closed,
}

impl Default for ByElectionStatus {
    fn default() -> Self {
        Self::Draft
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ByElectionTrigger {
    Recall,
    Vacancy,
    CourtOrder,
    Death,
}

impl Default for ByElectionTrigger {
    fn default() -> Self {
        Self::Vacancy
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SeatType {
    Mp,
    Senator,
    Governor,
    Cwr,
    Mca,
    Presidential,
}

impl Default for SeatType {
    fn default() -> Self {
        Self::Mp
    }
}

/// M50 aggregate: ByElection
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ByElection {
    pub id: String,
    pub constituency_id: String,
    pub seat_type: SeatType,
    pub trigger: ByElectionTrigger,
    pub status: ByElectionStatus,
    pub notice_date: String,
    pub election_date: String,
    pub cost_ceiling_kes: u64,
    pub gazette_notice_id: Option<String>,
    /// Set when triggered by M48 recall petition
    pub recall_petition_id: Option<String>,
    pub created_at: String,
}

impl ByElection {
    /// Returns true iff the election_date is fewer than SHORT_NOTICE_DAYS from notice_date.
    /// Computed in integer days; no floating point.
    pub fn is_short_notice(&self) -> bool {
        use chrono::NaiveDate;
        let notice = NaiveDate::parse_from_str(&self.notice_date, "%Y-%m-%d");
        let election = NaiveDate::parse_from_str(&self.election_date, "%Y-%m-%d");
        if let (Ok(n), Ok(e)) = (notice, election) {
            let diff = (e - n).num_days();
            diff < SHORT_NOTICE_DAYS
        } else {
            false
        }
    }
}

/// M50 aggregate: RegisterSubset -- voter register scoped to the affected constituency
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterSubset {
    pub id: String,
    pub by_election_id: String,
    /// FK to M02 BVR snapshot
    pub snapshot_id: String,
    pub constituency_id: String,
    pub station_count: u32,
    pub voter_count: u64,
    pub signed_blob_hash: Option<String>,
    pub generated_at: String,
    /// Set at nomination close; write path closed by OPA policy
    pub frozen_at: Option<String>,
}

/// Kit assignment within a KitPlan
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KitAssignment {
    pub kit_id: String,
    pub station_code: String,
    pub station_gps_lat: f64,
    pub station_gps_lng: f64,
    pub expected_delivery_date: String,
    pub acknowledged: bool,
}

/// M50 aggregate: KitPlan -- signed assignment of kits to polling stations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KitPlan {
    pub id: String,
    pub by_election_id: String,
    pub station_assignments: Vec<KitAssignment>,
    pub kit_ids: Vec<String>,
    pub signed_blob_hash: Option<String>,
    pub signed_at: Option<String>,
}

impl KitPlan {
    pub fn unacknowledged_count(&self) -> usize {
        self.station_assignments
            .iter()
            .filter(|a| !a.acknowledged)
            .count()
    }
}

/// M50 aggregate: CostReport -- submitted within T+14 of declaration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CostReport {
    pub id: String,
    pub by_election_id: String,
    pub kit_days_kes: u64,
    pub staff_days_kes: u64,
    pub transport_kes: u64,
    pub printing_kes: u64,
    pub venue_kes: u64,
    pub other_kes: u64,
    /// Must equal sum of all line items; validated on write
    pub total_kes: u64,
    pub submitted_at: Option<String>,
    pub submitted_to_cob: bool,
}

impl CostReport {
    /// Validates that total_kes equals the sum of all line items (FR-M50-038).
    pub fn validate_total(&self) -> Result<(), String> {
        let computed = self
            .kit_days_kes
            .saturating_add(self.staff_days_kes)
            .saturating_add(self.transport_kes)
            .saturating_add(self.printing_kes)
            .saturating_add(self.venue_kes)
            .saturating_add(self.other_kes);
        if computed != self.total_kes {
            Err(format!(
                "total_kes {total} does not equal sum of line items {computed}",
                total = self.total_kes
            ))
        } else {
            Ok(())
        }
    }
}
