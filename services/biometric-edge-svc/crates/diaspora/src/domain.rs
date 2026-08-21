//! Core domain types for diaspora voting (WP-5.8, M24).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// An embassy or consulate polling station abroad.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbassyStation {
    pub id: Uuid,
    /// ISO 3166-1 alpha-2 country code.
    pub country_iso: String,
    pub city: String,
    /// IANA timezone string, e.g. "America/New_York".
    pub timezone: String,
    pub address: String,
    /// Short station code, e.g. "US-DC-001".
    pub station_code: String,
    pub kits_deployed: u32,
    pub opened_at: Option<DateTime<Utc>>,
    pub closed_at: Option<DateTime<Utc>>,
}

impl EmbassyStation {
    pub fn new(
        country_iso: impl Into<String>,
        city: impl Into<String>,
        timezone: impl Into<String>,
        address: impl Into<String>,
        station_code: impl Into<String>,
        kits_deployed: u32,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            country_iso: country_iso.into(),
            city: city.into(),
            timezone: timezone.into(),
            address: address.into(),
            station_code: station_code.into(),
            kits_deployed,
            opened_at: None,
            closed_at: None,
        }
    }

    pub fn is_open(&self) -> bool {
        self.opened_at.is_some() && self.closed_at.is_none()
    }
}

/// A frozen snapshot of the diaspora voters eligible at a station.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DiasporaRoll {
    pub id: Uuid,
    pub embassy_station_id: Uuid,
    pub snapshot_taken_at: DateTime<Utc>,
    pub voter_count: u64,
    /// BLAKE3 hex digest over sorted voter IDs.
    pub hash: String,
    pub frozen: bool,
}

/// Official results form for diaspora polling stations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Form34ADiaspora {
    pub id: Uuid,
    pub embassy_station_id: Uuid,
    /// Human-readable form reference number.
    pub form_id: String,
    /// Map of candidate_id -> vote count.
    pub results: HashMap<String, u64>,
    pub rejected: u64,
    pub disputed: u64,
    pub transmitted_at: Option<DateTime<Utc>>,
    /// Base64-encoded HSM signature over the serialised results.
    pub hsm_signature: Option<String>,
    pub transmission_batch_id: Option<Uuid>,
}

impl Form34ADiaspora {
    pub fn new(
        embassy_station_id: Uuid,
        form_id: impl Into<String>,
        results: HashMap<String, u64>,
        rejected: u64,
        disputed: u64,
    ) -> Self {
        Self {
            id: Uuid::now_v7(),
            embassy_station_id,
            form_id: form_id.into(),
            results,
            rejected,
            disputed,
            transmitted_at: None,
            hsm_signature: None,
            transmission_batch_id: None,
        }
    }
}

/// Physical ballot pouch custody record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PouchCustody {
    pub id: Uuid,
    /// IDs of Form34ADiaspora records enclosed in this pouch.
    pub form_ids: Vec<Uuid>,
    pub sealed_at: Option<DateTime<Utc>>,
    /// Physical seal label or barcode.
    pub seal_id: Option<String>,
    pub custody_transfers: Vec<CustodyEvent>,
    pub received_at_nairobi: Option<DateTime<Utc>>,
}

impl PouchCustody {
    pub fn new(form_ids: Vec<Uuid>) -> Self {
        Self {
            id: Uuid::now_v7(),
            form_ids,
            sealed_at: None,
            seal_id: None,
            custody_transfers: Vec::new(),
            received_at_nairobi: None,
        }
    }

    pub fn seal(&mut self, seal_id: impl Into<String>, now: DateTime<Utc>) {
        self.seal_id = Some(seal_id.into());
        self.sealed_at = Some(now);
    }

    pub fn transfer(&mut self, event: CustodyEvent) {
        self.custody_transfers.push(event);
    }

    pub fn receive_at_nairobi(&mut self, now: DateTime<Utc>) {
        self.received_at_nairobi = Some(now);
    }
}

/// A single custody handover event in the pouch chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustodyEvent {
    pub transferred_at: DateTime<Utc>,
    pub from_person: String,
    pub to_person: String,
    pub location: String,
    pub gps_lat: f64,
    pub gps_lon: f64,
    /// Base64-encoded signature by the receiving officer.
    pub signature: String,
}
