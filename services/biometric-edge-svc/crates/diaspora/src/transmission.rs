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

//! Delayed batch transmission for Form 34A diaspora results (WP-5.8, M24).
//!
//! Forms are batched by station and transmitted on a schedule. If the
//! HTTP delivery fails, the batch is retried with exponential backoff.
//! Successful delivery emits `diaspora_form_transmitted.v1` via the outbox.
//! Permanent failure emits `diaspora_transmission_deferred.v1`.

use crate::domain::Form34ADiaspora;
use crate::events;
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{info, warn};
use uuid::Uuid;

/// Maximum delivery attempts before a batch is marked deferred.
const MAX_ATTEMPTS: u32 = 5;

#[derive(Debug, Error)]
pub enum TransmissionError {
    #[error("all {MAX_ATTEMPTS} delivery attempts failed for batch {0}")]
    AllAttemptsFailed(Uuid),
    #[error("serialisation error: {0}")]
    Serialise(#[from] serde_json::Error),
}

/// A batch of forms scheduled for delayed transmission.
#[derive(Debug, Clone)]
pub struct TransmissionBatch {
    pub id: Uuid,
    /// Deduplicated form IDs in this batch (order preserved from insertion).
    pub form_ids: Vec<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub attempts: u32,
    pub last_error: Option<String>,
    pub status: BatchStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BatchStatus {
    Pending,
    Sent,
    Deferred,
}

/// Build a batch from a list of forms, deduplicating by form_id.
/// Returns the batch and the deduplicated slice of forms to transmit.
pub fn batch_transmit(
    forms: Vec<Form34ADiaspora>,
    scheduled_at: DateTime<Utc>,
) -> (TransmissionBatch, Vec<Form34ADiaspora>) {
    let mut seen: HashMap<Uuid, bool> = HashMap::new();
    let mut deduped: Vec<Form34ADiaspora> = Vec::new();
    for form in forms {
        if seen.insert(form.id, true).is_none() {
            deduped.push(form);
        }
    }
    let batch = TransmissionBatch {
        id: Uuid::now_v7(),
        form_ids: deduped.iter().map(|f| f.id).collect(),
        scheduled_at,
        attempts: 0,
        last_error: None,
        status: BatchStatus::Pending,
    };
    (batch, deduped)
}

/// Compute the next retry time using exponential backoff (base 30 s).
pub fn next_retry_at(batch: &TransmissionBatch, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if batch.attempts >= MAX_ATTEMPTS {
        return None;
    }
    let delay_secs = 30i64 * (1i64 << batch.attempts.min(10));
    Some(now + Duration::seconds(delay_secs))
}

/// Attempt delivery of a batch. Returns the updated batch and a list of
/// outbox envelopes to persist (success = transmitted, failure = deferred).
///
/// The `deliver_fn` parameter allows injecting a mock transport for tests.
pub async fn attempt_delivery<F, Fut>(
    mut batch: TransmissionBatch,
    forms: &[Form34ADiaspora],
    deliver_fn: F,
    now: DateTime<Utc>,
) -> (
    TransmissionBatch,
    Vec<eel_outbox_rs::OutboxEnvelope>,
    Result<(), TransmissionError>,
)
where
    F: FnOnce(Vec<Form34ADiaspora>) -> Fut,
    Fut: std::future::Future<Output = Result<(), String>>,
{
    batch.attempts += 1;
    let forms_owned: Vec<Form34ADiaspora> = forms.to_vec();

    match deliver_fn(forms_owned.clone()).await {
        Ok(()) => {
            info!(batch_id = %batch.id, forms = batch.form_ids.len(), "diaspora batch delivered");
            batch.status = BatchStatus::Sent;
            let envelopes: Vec<eel_outbox_rs::OutboxEnvelope> = forms_owned
                .iter()
                .map(|f| {
                    events::make_envelope(
                        events::FORM_TRANSMITTED,
                        f.id,
                        Some(&format!("form-34a/{}", f.form_id)),
                        serde_json::json!({
                            "form_id": f.id,
                            "batch_id": batch.id,
                            "transmitted_at": now,
                        }),
                    )
                })
                .collect();
            (batch, envelopes, Ok(()))
        }
        Err(err) => {
            warn!(batch_id = %batch.id, attempt = batch.attempts, error = %err, "diaspora delivery failed");
            batch.last_error = Some(err);
            let final_envelope = if batch.attempts >= MAX_ATTEMPTS {
                batch.status = BatchStatus::Deferred;
                vec![events::make_envelope(
                    events::TRANSMISSION_DEFERRED,
                    batch.id,
                    None,
                    serde_json::json!({
                        "batch_id": batch.id,
                        "form_ids": batch.form_ids,
                        "attempts": batch.attempts,
                        "deferred_at": now,
                    }),
                )]
            } else {
                vec![]
            };
            (
                batch,
                final_envelope,
                Err(TransmissionError::AllAttemptsFailed(Uuid::now_v7())),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_form(station_id: Uuid) -> Form34ADiaspora {
        Form34ADiaspora::new(
            station_id,
            format!("FORM-{}", Uuid::now_v7()),
            HashMap::from([("candidate-A".to_owned(), 42u64)]),
            0,
            0,
        )
    }

    #[test]
    fn batch_deduplicates_by_form_id() {
        let station = Uuid::now_v7();
        let form = make_form(station);
        let (batch, deduped) =
            batch_transmit(vec![form.clone(), form.clone()], Utc::now());
        assert_eq!(deduped.len(), 1);
        assert_eq!(batch.form_ids.len(), 1);
    }

    #[test]
    fn batch_preserves_distinct_forms() {
        let station = Uuid::now_v7();
        let (batch, deduped) =
            batch_transmit(vec![make_form(station), make_form(station)], Utc::now());
        assert_eq!(deduped.len(), 2);
        assert_eq!(batch.form_ids.len(), 2);
    }

    #[test]
    fn exponential_backoff_grows() {
        let batch = TransmissionBatch {
            id: Uuid::now_v7(),
            form_ids: vec![],
            scheduled_at: Utc::now(),
            attempts: 0,
            last_error: None,
            status: BatchStatus::Pending,
        };
        let now = Utc::now();
        let r0 = next_retry_at(&batch, now).unwrap();
        let batch1 = TransmissionBatch { attempts: 1, ..batch.clone() };
        let r1 = next_retry_at(&batch1, now).unwrap();
        assert!(r1 > r0);
    }

    #[test]
    fn no_retry_after_max_attempts() {
        let batch = TransmissionBatch {
            id: Uuid::now_v7(),
            form_ids: vec![],
            scheduled_at: Utc::now(),
            attempts: MAX_ATTEMPTS,
            last_error: None,
            status: BatchStatus::Deferred,
        };
        assert!(next_retry_at(&batch, Utc::now()).is_none());
    }

    #[tokio::test]
    async fn successful_delivery_emits_transmitted_events() {
        let station = Uuid::now_v7();
        let form = make_form(station);
        let (batch, forms) = batch_transmit(vec![form], Utc::now());
        let (updated, envelopes, result) = attempt_delivery(
            batch,
            &forms,
            |_forms| async { Ok(()) },
            Utc::now(),
        )
        .await;
        assert!(result.is_ok());
        assert_eq!(updated.status, BatchStatus::Sent);
        assert_eq!(envelopes.len(), 1);
        assert_eq!(envelopes[0].event_type, events::FORM_TRANSMITTED);
    }

    #[tokio::test]
    async fn failed_delivery_below_max_does_not_defer() {
        let station = Uuid::now_v7();
        let form = make_form(station);
        let (batch, forms) = batch_transmit(vec![form], Utc::now());
        let (_updated, envelopes, _result) = attempt_delivery(
            batch,
            &forms,
            |_| async { Err("network timeout".to_owned()) },
            Utc::now(),
        )
        .await;
        // No deferred envelope yet (attempts=1 < MAX_ATTEMPTS=5).
        assert!(envelopes.is_empty());
    }
}
