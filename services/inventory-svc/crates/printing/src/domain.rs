use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
/// PrintOrderStatus.
pub enum PrintOrderStatus {
    /// variant.
    Pending,
    /// variant.
    DualSigned,
    /// variant.
    Transmitted,
    /// variant.
    Printed,
    /// variant.
    InTransit,
    /// variant.
    AtWarehouse,
    /// variant.
    Allocated,
    /// variant.
    AtStation,
    /// variant.
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// PrintOrder.
pub struct PrintOrder {
    /// id.
    pub id: String,
    /// election_id.
    pub election_id: String,
    /// contest_id.
    pub contest_id: String,
    /// constituency_code.
    pub constituency_code: String,
    /// quantity_authorised.
    pub quantity_authorised: u64,
    /// serial_range_start.
    pub serial_range_start: u64,
    /// serial_range_end.
    pub serial_range_end: u64,
    /// paper_spec_hash.
    pub paper_spec_hash: String,
    /// issued_by_officer_id.
    pub issued_by_officer_id: String,
    /// countersigned_by_commissioner_id.
    pub countersigned_by_commissioner_id: String,
    /// issued_at.
    pub issued_at: DateTime<Utc>,
    /// gazette_notice_id.
    pub gazette_notice_id: Option<String>,
    /// status.
    pub status: PrintOrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// LotData.
pub struct LotData {
    /// id.
    pub id: String,
    /// print_order_id.
    pub print_order_id: String,
    /// printer_facility_id.
    pub printer_facility_id: String,
    /// serial_range_start.
    pub serial_range_start: u64,
    /// serial_range_end.
    pub serial_range_end: u64,
    /// quantity_printed.
    pub quantity_printed: u64,
    /// quantity_rejected.
    pub quantity_rejected: u64,
    /// rfid_chip_id_list_hash.
    pub rfid_chip_id_list_hash: String,
    /// print_date.
    pub print_date: DateTime<Utc>,
    /// received_at.
    pub received_at: Option<DateTime<Utc>>,
    /// variance_pct.
    pub variance_pct: f64,
    /// variance_alert_sent.
    pub variance_alert_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// PrintCustodyEventType.
pub enum PrintCustodyEventType {
    /// variant.
    PortClearance,
    /// variant.
    WarehouseReceipt,
    /// variant.
    ConstituencyAllocation,
    /// variant.
    StationReceipt,
    /// variant.
    StationCloseReturn,
    /// variant.
    RfidSpotCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// PrintCustodyEvent.
pub struct PrintCustodyEvent {
    /// id.
    pub id: String,
    /// lot_id.
    pub lot_id: String,
    /// event_type.
    pub event_type: PrintCustodyEventType,
    /// location_code.
    pub location_code: String,
    /// officer_id.
    pub officer_id: String,
    /// witness_id.
    pub witness_id: Option<String>,
    /// quantity_scanned.
    pub quantity_scanned: u64,
    /// rfid_match_count.
    pub rfid_match_count: u64,
    /// rfid_mismatch_count.
    pub rfid_mismatch_count: u64,
    /// seal_number.
    pub seal_number: Option<String>,
    /// scanned_at.
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// StationReconciliation.
pub struct StationReconciliation {
    /// id.
    pub id: String,
    /// station_code.
    pub station_code: String,
    /// election_id.
    pub election_id: String,
    /// ballots_received.
    pub ballots_received: u64,
    /// ballots_issued.
    pub ballots_issued: u64,
    /// ballots_unused.
    pub ballots_unused: u64,
    /// ballots_spoilt.
    pub ballots_spoilt: u64,
    /// discrepancy.
    pub discrepancy: i64,
    /// discrepancy_flag.
    pub discrepancy_flag: bool,
    /// reconciled_at.
    pub reconciled_at: DateTime<Utc>,
    /// reconciled_by.
    pub reconciled_by: String,
}

/// discrepancy.
pub fn discrepancy(r: &StationReconciliation) -> i64 {
    r.ballots_received as i64
        - r.ballots_issued as i64
        - r.ballots_unused as i64
        - r.ballots_spoilt as i64
}
