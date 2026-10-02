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

//! Fee schedules and nomination fee payment — WP-4.11, FR-M47.
//! Nomination fees are charged per position per election cycle; a candidate's
//! payment must be confirmed before submission (FR-M47-006).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::candidate::Position;

/// Nomination fee schedule for an elective position in a given cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeSchedule {
    pub id: Uuid,
    pub election_cycle: String,
    pub position: Position,
    /// Amount in the smallest currency unit (KSh cents).
    pub amount_cents: u64,
    pub due_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl FeeSchedule {
    pub fn new(
        election_cycle: String,
        position: Position,
        amount_cents: u64,
        due_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            election_cycle,
            position,
            amount_cents,
            due_at,
            created_at: Utc::now(),
        }
    }
}

/// Lifecycle of a nomination fee payment (FR-M47-006).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaymentStatus {
    Pending,
    Confirmed,
    Refunded,
    Overdue,
}

impl Default for PaymentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// A fee payment recorded against a candidate's nomination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeePayment {
    pub id: Uuid,
    pub schedule_id: Uuid,
    pub candidate_id: Uuid,
    pub amount_cents: u64,
    pub status: PaymentStatus,
    pub mpesa_ref: Option<String>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl FeePayment {
    pub fn new(schedule_id: Uuid, candidate_id: Uuid, amount_cents: u64) -> Self {
        Self {
            id: Uuid::new_v4(),
            schedule_id,
            candidate_id,
            amount_cents,
            status: PaymentStatus::Pending,
            mpesa_ref: None,
            confirmed_at: None,
            created_at: Utc::now(),
        }
    }

    /// Confirm via M-Pesa callback or officer reconciliation.
    pub fn confirm(&mut self, mpesa_ref: String) {
        self.status = PaymentStatus::Confirmed;
        self.mpesa_ref = Some(mpesa_ref);
        self.confirmed_at = Some(Utc::now());
    }

    pub fn is_confirmed(&self) -> bool {
        self.status == PaymentStatus::Confirmed
    }
}

/// Locate the payable schedule for a candidate's position.
pub fn schedule_for_position<'a>(
    schedules: &'a [FeeSchedule],
    position: &crate::candidate::Position,
) -> Option<&'a FeeSchedule> {
    schedules.iter().find(|s| &s.position == position)
}
