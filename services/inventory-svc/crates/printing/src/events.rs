use serde::{Deserialize, Serialize};

pub const CUSTODY_EVENT_RECORDED: &str = "eel.printing.custody_event_recorded";
pub const QUANTITY_VARIANCE_ALERT: &str = "eel.printing.quantity_variance_alert";
pub const RFID_MISMATCH_ALERT: &str = "eel.printing.rfid_mismatch_alert";
pub const STATION_RECONCILIATION_DISCREPANCY: &str =
    "eel.printing.station_reconciliation_discrepancy";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantityVarianceAlert {
    pub lot_id: String,
    pub authorised: u64,
    pub printed: u64,
    pub variance_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RfidMismatchAlert {
    pub lot_id: String,
    pub chip_count_expected: u64,
    pub chip_count_actual: u64,
}
