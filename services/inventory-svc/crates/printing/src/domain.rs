use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintOrderStatus {
    Pending,
    DualSigned,
    Transmitted,
    Printed,
    InTransit,
    AtWarehouse,
    Allocated,
    AtStation,
    Reconciled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintOrder {
    pub id: String,
    pub election_id: String,
    pub contest_id: String,
    pub constituency_code: String,
    pub quantity_authorised: u64,
    pub serial_range_start: u64,
    pub serial_range_end: u64,
    pub paper_spec_hash: String,
    pub issued_by_officer_id: String,
    pub countersigned_by_commissioner_id: String,
    pub issued_at: DateTime<Utc>,
    pub gazette_notice_id: Option<String>,
    pub status: PrintOrderStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LotData {
    pub id: String,
    pub print_order_id: String,
    pub printer_facility_id: String,
    pub serial_range_start: u64,
    pub serial_range_end: u64,
    pub quantity_printed: u64,
    pub quantity_rejected: u64,
    pub rfid_chip_id_list_hash: String,
    pub print_date: DateTime<Utc>,
    pub received_at: Option<DateTime<Utc>>,
    pub variance_pct: f64,
    pub variance_alert_sent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrintCustodyEventType {
    PortClearance,
    WarehouseReceipt,
    ConstituencyAllocation,
    StationReceipt,
    StationCloseReturn,
    RfidSpotCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrintCustodyEvent {
    pub id: String,
    pub lot_id: String,
    pub event_type: PrintCustodyEventType,
    pub location_code: String,
    pub officer_id: String,
    pub witness_id: Option<String>,
    pub quantity_scanned: u64,
    pub rfid_match_count: u64,
    pub rfid_mismatch_count: u64,
    pub seal_number: Option<String>,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StationReconciliation {
    pub id: String,
    pub station_code: String,
    pub election_id: String,
    pub ballots_received: u64,
    pub ballots_issued: u64,
    pub ballots_unused: u64,
    pub ballots_spoilt: u64,
    pub discrepancy: i64,
    pub discrepancy_flag: bool,
    pub reconciled_at: DateTime<Utc>,
    pub reconciled_by: String,
}

pub fn discrepancy(r: &StationReconciliation) -> i64 {
    r.ballots_received as i64
        - r.ballots_issued as i64
        - r.ballots_unused as i64
        - r.ballots_spoilt as i64
}
