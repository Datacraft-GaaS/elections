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

use serde::{Deserialize, Serialize};

/// CUSTODY_EVENT_RECORDED.
pub const CUSTODY_EVENT_RECORDED: &str = "eel.printing.custody_event_recorded";
/// QUANTITY_VARIANCE_ALERT.
pub const QUANTITY_VARIANCE_ALERT: &str = "eel.printing.quantity_variance_alert";
/// RFID_MISMATCH_ALERT.
pub const RFID_MISMATCH_ALERT: &str = "eel.printing.rfid_mismatch_alert";
/// STATION_RECONCILIATION_DISCREPANCY.
pub const STATION_RECONCILIATION_DISCREPANCY: &str =
    "eel.printing.station_reconciliation_discrepancy";

#[derive(Debug, Clone, Serialize, Deserialize)]
/// QuantityVarianceAlert.
pub struct QuantityVarianceAlert {
    /// lot_id.
    pub lot_id: String,
    /// authorised.
    pub authorised: u64,
    /// printed.
    pub printed: u64,
    /// variance_pct.
    pub variance_pct: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// RfidMismatchAlert.
pub struct RfidMismatchAlert {
    /// lot_id.
    pub lot_id: String,
    /// chip_count_expected.
    pub chip_count_expected: u64,
    /// chip_count_actual.
    pub chip_count_actual: u64,
}
